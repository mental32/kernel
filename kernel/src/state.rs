use core::{convert::TryInto, mem::size_of};

use alloc::boxed::Box;

use acpi::{search_for_rsdp_bios, AcpiError as InnerAcpiError, InterruptModel, ProcessorState};
use x86_64::{
    instructions::{
        segmentation::set_cs,
        tables::{lidt, load_tss, DescriptorTablePointer},
    },
    structures::{
        gdt::{Descriptor, DescriptorFlags, SegmentSelector},
        idt::{HandlerFunc, InterruptDescriptorTable},
        paging::page_table::PageTableFlags,
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use bit_field::BitField;
use multiboot2::BootInformation;

use crate::dev::{
    apic::{self, Lapic},
    pci::{self, PciEnumeration},
    pic::{CHIP_8259, DEFAULT_PIC_SLAVE_OFFSET},
};
use crate::gdt::ExposedGlobalDescriptorTable;
use crate::mm::{self, LockedHeap, MemoryManagerType, PhysFrameManager, MEMORY_MANAGER};
use crate::result::{AcpiError, KernelException, KernelResult};
use crate::{info, isr, smp, GLOBAL_ALLOCATOR};

struct Selectors {
    code_selector: Option<SegmentSelector>,
    tss_selector: Option<SegmentSelector>,
}

/// A struct that journals the kernels state.
pub struct KernelStateObject {
    // Hardware
    // pci_devices: Option<PciEnumeration>,
    // Structures
    heap_allocator: Option<&'static LockedHeap>,
    memory_manager: Option<&'static MemoryManagerType>,
    selectors: Selectors,
    // Tables
    gdt: ExposedGlobalDescriptorTable,
    idt: InterruptDescriptorTable,
    tss: TaskStateSegment,
}

use acpi::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;

impl AcpiHandler for KernelStateObject {
    fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        _size: usize,
    ) -> PhysicalMapping<T> {
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size_of::<T>(),
            mapped_length: size_of::<T>(),
        }
    }

    fn unmap_physical_region<T>(&mut self, _region: PhysicalMapping<T>) {}
}

impl KernelStateObject {
    pub const fn new() -> Self {
        let idt = InterruptDescriptorTable::new();
        let tss = TaskStateSegment::new();
        let gdt = ExposedGlobalDescriptorTable::new();

        let selectors = Selectors {
            code_selector: None,
            tss_selector: None,
        };

        Self {
            idt,
            tss,
            gdt,

            selectors,

            heap_allocator: None,
            memory_manager: None,
            // pci_devices: None,
        }
    }

    // Initialization related methods

    /// Load the GDT, IDT tables and CS, TSS selectors.
    unsafe fn load_tables(&mut self) {
        // IDT
        isr::map_default_handlers(&mut self.idt);
        self.load_idt();

        // TSS
        self.tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;

            let interrupt_stack = Box::into_raw(Box::new([0; STACK_SIZE]));

            let stack_start = VirtAddr::from_ptr(interrupt_stack);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // GDT
        let tss_descriptor = {
            let ptr = (&self.tss) as *const _ as u64;

            let mut low = DescriptorFlags::PRESENT.bits();
            // base
            low.set_bits(16..40, ptr.get_bits(0..24));
            low.set_bits(56..64, ptr.get_bits(24..32));
            // limit (the `-1` in needed since the bound is inclusive)
            low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
            // type (0b1001 = available 64-bit tss)
            low.set_bits(40..44, 0b1001);

            let mut high = 0;
            high.set_bits(0..32, ptr.get_bits(32..64));

            Descriptor::SystemSegment(low, high)
        };

        let code_selector = self.gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = self.gdt.add_entry(tss_descriptor);
        self.gdt.load();

        // SELECTORS
        self.selectors = Selectors {
            code_selector: Some(code_selector),
            tss_selector: Some(tss_selector),
        };

        set_cs(self.selectors.code_selector.unwrap());
        load_tss(self.selectors.tss_selector.unwrap());
    }

    /// Detect and use ACPI to load device drivers and initialize the APIC and
    /// AML interpreter.
    unsafe fn load_device_drivers(&mut self) -> KernelResult<()> {
        let maybe_acpi = {
            let acpi = match search_for_rsdp_bios(self) {
                Ok(acpi) => Some(acpi),
                Err(acpi_error) => match acpi_error {
                    InnerAcpiError::NoValidRsdp => None,
                    err => panic!("{:?}", err),
                },
            };

            acpi
        };

        let mut legacy_pics_supported = true;

        if let Some(acpi) = maybe_acpi {
            let bsp = acpi.boot_processor.expect("No boot processor detected?!");

            info!("ACPI says BSP is: {:?}", bsp);
            assert!(
                !bsp.is_ap,
                "Bootstrap processor marked as application processor!"
            );

            let model = acpi.interrupt_model.expect("No interrupt model detected!");

            // APIC/LAPIC initialization and setup
            match model {
                InterruptModel::Apic(apic) => {
                    info!("APIC detected!");

                    if !apic.also_has_legacy_pics {
                        info!("APIC says legacy PICS are NOT supported!");
                        legacy_pics_supported = false;
                    } else {
                        info!("Proceeding to remap and mask legacy pics");
                        CHIP_8259.remap(0xA0, 0xA8);
                        CHIP_8259.mask_all();
                    }

                    info!("APIC initialization for BSP");
                    apic::init_processor(bsp, &apic);

                    for ap in acpi.application_processors {
                        info!(
                            "Processing AP => (processor_uid: {}, lapic_id: {})",
                            ap.processor_uid, ap.local_apic_id
                        );

                        match ap.state {
                            ProcessorState::Disabled => {
                                info!("  AP marked disabled! skipping... ({:?})", ap)
                            }

                            ProcessorState::Running => {
                                panic!("Processor was running during SMP initialization?! {:?}", ap)
                            }

                            ProcessorState::WaitingForSipi => {
                                info!("  [1/2] Attempting to apic init AP");
                                apic::init_processor(ap, &apic)
                                    .expect("Unable to apic init application processor");

                                info!("  [2/2] Attempting SIPI to AP");
                                smp::sipi(ap).expect("Unable to complete SIPI for an AP");
                            }
                        }
                    }
                }

                InterruptModel::Pic => info!("PIC detected! (Are we on a legacy system?)"),
                _ => panic!("Unknown interrupt model was reported!"),
            }

            // AML interpreter instance
            // TODO: Finish wrapping lai (https://github.com/mental32/lai-rs)
            info!("AML support not implemented yet!");

            if let Some(pci_config_regions) = acpi.pci_config_regions {
                info!("PCI-E configuration regions detected.");

                let mut pci_enumeration = PciEnumeration::new();

                info!("Attempting a PCI device enumeration.");
                pci::cam::brute_force_enumerate(&mut pci_enumeration);

                for device in &pci_enumeration.devices {
                    info!("  PCI DEVICE => {:?}", &device);
                }

                info!(
                    "Completed PCI device enumeration with {:?} device(s)",
                    &pci_enumeration.devices.len()
                );

            // self.pci_devices = Some(pci_enumeration);
            } else {
                info!("No PCI-E configuration regions detected.");
            }
        } else {
            info!("No ACPI support detected!");
        }

        if legacy_pics_supported {
            CHIP_8259.setup(DEFAULT_PIC_SLAVE_OFFSET, self);
        }

        Ok(())
    }

    /// Prepare the kernel and host machine state.
    pub unsafe fn prepare(&mut self, boot_info: &BootInformation) -> KernelResult<()> {
        if self.heap_allocator.is_some() {
            return Err(KernelException::AlreadyInitialized(
                "Attempted to call KernelStateObject::prepare twice.",
            ));
        }

        self.heap_allocator = Some(&GLOBAL_ALLOCATOR);
        self.memory_manager = Some(&MEMORY_MANAGER);

        // Initialize the memory manager.
        {
            let mut memory_manager = self
                .memory_manager
                .unwrap()
                .try_lock()
                .expect("Unable to unlock the memory manager during kernel prepare.");

            // Use the boot info memory map to find all non overlapping
            // 4KiB sized holes in available memory.
            let holes = mm::boot_frame::find_holes(0x1000, boot_info);

            use crate::mm::pmm::{BitMap, INITIAL_PHYSFRAME_BITMAP, INITIAL_PHYSFRAME_BITMAP_SIZE};
            use core::sync::atomic::AtomicPtr;

            let initial_bitmap = BitMap::new(
                AtomicPtr::new(&mut INITIAL_PHYSFRAME_BITMAP as *mut _ as *mut u8),
                INITIAL_PHYSFRAME_BITMAP_SIZE.try_into().unwrap(),
            );

            let virtual_offset = VirtAddr::new(0x00);
            memory_manager.initialize(virtual_offset, PhysFrameManager::new(initial_bitmap, holes));

            info!("Initialized VMM & PMM");

            // Allocate and map the heap.
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            for page in mm::page_range_inclusive(HEAP_START, HEAP_START + HEAP_SIZE) {
                memory_manager.map_to(page, flags).unwrap().flush();
            }

            self.heap_allocator
                .unwrap()
                .try_lock()
                .expect("Unable to lock the heap allocator during kernel prepare.")
                .init(
                    HEAP_START.try_into().unwrap(),
                    HEAP_SIZE.try_into().unwrap(),
                );
        }

        // Load tables
        self.load_tables();

        // Device drivers
        self.load_device_drivers()
            .expect("Failed to load device drivers.");

        Ok(())
    }

    pub unsafe fn set_idt_entry(&mut self, index: u8, handler: HandlerFunc) {
        self.idt[index as usize].set_handler_fn(handler);
    }

    pub unsafe fn load_idt(&mut self) {
        let ptr = DescriptorTablePointer {
            base: (&self.idt) as *const _ as u64,
            limit: (size_of::<InterruptDescriptorTable>() - 1) as u16,
        };

        lidt(&ptr);
    }
}
