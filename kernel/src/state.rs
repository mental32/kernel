use core::{convert::TryInto, mem::size_of};

use alloc::boxed::Box;

use {bit_field::BitField, multiboot2::BootInformation, smallvec::SmallVec, spin::Mutex};

use x86_64::{
    instructions::{
        segmentation::set_cs,
        tables::{lidt, load_tss, DescriptorTablePointer},
    },
    registers::control::{Cr3, Cr3Flags},
    structures::{
        gdt::{Descriptor, DescriptorFlags, SegmentSelector},
        idt::{HandlerFunc, InterruptDescriptorTable},
        paging::{
            frame::PhysFrame,
            page::{PageRange, PageRangeInclusive},
            page_table::{PageTable, PageTableFlags},
            Page,
        },
        tss::TaskStateSegment,
    },
    PhysAddr, VirtAddr,
};

use {pic8259::ChainedPics, pit825x::ProgrammableIntervalTimer, serial::sprintln};

use crate::{
    dev::{apic, pic::CHIP_8259, vga::VGAFramebuffer},
    gdt::ExposedGlobalDescriptorTable,
    isr,
    mm::{self, LockedHeap, PAGE_MAP_LEVEL_4},
    result::{KernelException, KernelResult},
    GLOBAL_ALLOCATOR,
};

const TWO_MIB: usize = 0x200000;

struct Selectors {
    code_selector: Option<SegmentSelector>,
    tss_selector: Option<SegmentSelector>,
}

/// A struct that journals the kernels state.
pub struct KernelStateObject {
    // Hardware
    devices: Option<()>,
    // Structures
    heap: Option<&'static LockedHeap>,
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
        size: usize,
    ) -> PhysicalMapping<T> {
        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut T).unwrap(),
            region_length: size_of::<T>(),
            mapped_length: size_of::<T>(),
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {}
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
            heap: None,

            devices: None,
        }
    }

    // Initialization related methods
    unsafe fn load_tables(&mut self) {
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

        // IDT
        isr::map_default_handlers(&mut self.idt);
        self.load_idt();
    }

    unsafe fn load_device_drivers(&mut self) {
        // ACPI
        use acpi::{search_for_rsdp_bios, AcpiError};

        let maybe_acpi = {
            let acpi = match search_for_rsdp_bios(self) {
                Ok(acpi) => Some(acpi),
                Err(acpi_error) => match acpi_error {
                    AcpiError::NoValidRsdp => None,
                    err => panic!("{:?}", err),
                },
            };

            acpi
        };

        let apic_supported = apic::is_apic_supported();
        let mut legacy_pics_supported = true;

        if let Some(acpi) = maybe_acpi {
            // APIC/LAPIC initialization and setup
            if apic_supported {
                sprintln!("APIC support detected, proceeding to remap and mask PIT8259");

                if let Ok((apic, lapic_eoi_ptr)) = apic::initialize(&acpi) {
                    if apic.also_has_legacy_pics {
                        CHIP_8259.remap(0xA0, 0xA8);
                        CHIP_8259.mask_all();
                    } else {
                        legacy_pics_supported = false;
                    }

                    sprintln!("Finished initializing the APIC.");
                }
            } else {
                sprintln!("NO APIC support detected!");
            }

            // AML interpreter instance
            // TODO: Finish wrapping lai (https://github.com/mental32/lai-rs)
        }

        if legacy_pics_supported {
            CHIP_8259.setup(self);
        }
    }

    pub unsafe fn prepare(&mut self, boot_info: &BootInformation) -> KernelResult<()> {
        if self.heap.is_some() {
            return Err(KernelException::IllegalDoubleCall(
                "Attempted to call KernelStateObject::prepare twice.",
            ));
        }

        self.heap = Some(&GLOBAL_ALLOCATOR);

        let map = boot_info.memory_map_tag().unwrap();
        let last_addr = map
            .memory_areas()
            .into_iter()
            .map(|area| area.end_address())
            .max()
            .unwrap();

        // PAGING
        self.initial_pml3_map(last_addr);

        // ALLOCATOR
        let (heap_start, heap_end) =
            mm::boot_frame::find_first_non_overlapping_free_area(0x400 * 100, boot_info);

        self.heap.unwrap().lock().init(
            heap_start.try_into().unwrap(),
            (heap_end - heap_start).try_into().unwrap(),
        );

        // Load tables
        // self.load_tables();

        // Device drivers
        // self.load_device_drivers();

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
