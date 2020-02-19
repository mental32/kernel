use core::{convert::TryInto, mem::size_of};

use {bit_field::BitField, multiboot2::BootInformation, smallvec::SmallVec, spin::Mutex};

use x86_64::{
    instructions::{
        segmentation::set_cs,
        tables::{lidt, load_tss, DescriptorTablePointer},
    },
    registers::control::{Cr3, Cr3Flags},
    structures::{
        gdt::{Descriptor, DescriptorFlags, SegmentSelector},
        idt::InterruptDescriptorTable,
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
    dev::{pic8259::PICS, pit825x::PIT},
    gdt::ExposedGlobalDescriptorTable,
    isr,
    mm::{self, LockedHeap, PAGE_MAP_LEVEL_4},
    result::{KernelException, Result as KernelResult},
    GLOBAL_ALLOCATOR,
};

const TWO_MIB: usize = 0x200000;

struct Selectors {
    code_selector: Option<SegmentSelector>,
    tss_selector: Option<SegmentSelector>,
}

use alloc::{boxed::Box, vec::Vec};

trait Device {}

/// A struct that journals the kernels state.
pub struct KernelStateObject {
    // Hardware
    devices: Option<Vec<u8>>,
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
            mapped_length: x86_64::align_up(size_of::<T>() as u64, 64) as usize,
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

    unsafe fn initial_pml3_map(&mut self, last_addr: u64) {
        // Identity map all possible physical memory up to 2GiB
        static mut IDENT_MAP_PML3: PageTable = PageTable::new();
        assert!(IDENT_MAP_PML3.iter().all(|entry| entry.is_unused()));

        for (index, entry) in IDENT_MAP_PML3.iter_mut().enumerate() {
            let addr = (TWO_MIB * index).try_into().unwrap();

            if addr >= last_addr {
                sprintln!(
                    "Stopping identity mapping at PML3 index={:?}, addr=0x{:x?}, last_addr=0x{:x?}",
                    index,
                    addr,
                    last_addr
                );
                break;
            }

            entry.set_addr(
                PhysAddr::new(addr),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
            );
        }

        // Map the identity PML3 to the new PML4 and updated CR3
        let mut pml4 = PAGE_MAP_LEVEL_4.write();
        pml4.zero();

        pml4[0].set_addr(
            PhysAddr::new(&IDENT_MAP_PML3 as *const PageTable as u64),
            PageTableFlags::PRESENT,
        );

        let pml4_addr = &*pml4 as *const PageTable as u64;
        let phys_addr = PhysAddr::new(pml4_addr);
        Cr3::write(PhysFrame::containing_address(phys_addr), Cr3Flags::empty());
    }

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
        sprintln!("{:?}", acpi::search_for_rsdp_bios(self));
        // self.pic = Some(&(*PICS));

        // {
        //     let mut handle = self.pic.unwrap().lock();
        //     handle.initialize();
        // }

        // self.pit = Some(&PIT);

        // {
        //     let mut handle = self.pit.unwrap().lock();
        //     handle.set_frequency(100);
        // }
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

        sprintln!("{:x?}", boot_info.framebuffer_tag().unwrap().address);

        // PAGING
        self.initial_pml3_map(last_addr);

        // ALLOCATOR

        let (heap_start, heap_end) = mm::boot_frame::find_non_overlapping(boot_info);

        self.heap.unwrap().lock().init(
            heap_start.try_into().unwrap(),
            (heap_end - heap_start).try_into().unwrap(),
        );

        // Load tables
        self.load_tables();

        // Device drivers
        self.load_device_drivers();

        Ok(())
    }

    pub unsafe fn load_idt(&mut self) {
        let ptr = DescriptorTablePointer {
            base: (&self.idt) as *const _ as u64,
            limit: (size_of::<InterruptDescriptorTable>() - 1) as u16,
        };

        lidt(&ptr);
    }
}
