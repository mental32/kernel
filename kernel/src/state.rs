use core::mem::size_of;

use {
    bit_field::BitField,
    multiboot2::BootInformation,
    spin::{Mutex, RwLock},
    x86_64::{
        instructions::{
            segmentation::set_cs,
            tables::{lidt, load_tss, DescriptorTablePointer},
        },
        registers::control::{Cr2, Cr3, Cr3Flags},
        structures::{
            gdt::{Descriptor, DescriptorFlags, SegmentSelector},
            idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
            paging::{
                frame::PhysFrame,
                page_table::{PageTable, PageTableEntry, PageTableFlags},
            },
            tss::TaskStateSegment,
        },
        PhysAddr, VirtAddr,
    },
};

use {pic8259::ChainedPics, pit825x::ProgrammableIntervalTimer, serial::sprintln};

use crate::{
    gdt::ExposedGlobalDescriptorTable,
    isr::{
        self,
        pics::{PICS, PIT},
    },
    result::{KernelException, Result as KernelResult},
};

struct Selectors {
    code_selector: Option<SegmentSelector>,
    tss_selector: Option<SegmentSelector>,
}

/// A struct that journals the kernels state.
pub struct KernelStateObject {
    // Hardware
    pic: Option<&'static Mutex<ChainedPics>>,
    pit: Option<&'static Mutex<ProgrammableIntervalTimer>>,
    // Structures
    gdt: ExposedGlobalDescriptorTable,
    idt: InterruptDescriptorTable,
    tss: TaskStateSegment,
    selectors: Selectors,
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
            selectors,
            idt,
            tss,
            gdt,
            pic: None,
            pit: None,
        }
    }

    pub unsafe fn prepare(&mut self, _boot_info: &BootInformation) -> KernelResult<()> {
        if self.pic.is_some() {
            return Err(KernelException::IllegalDoubleCall(
                "Attempted to call KernelStateObject::prepare twice.",
            ));
        }

        // PAGING
        use crate::mm::PAGE_MAP_LEVEL_4;

        {
            let mut pml4 = PAGE_MAP_LEVEL_4.write();
            pml4.zero();

            static mut PML3: PageTable = PageTable::new();

            let mut base = 0x0;
            let offset = 0x200000;

            for entry in PML3.iter_mut() {
                entry.set_addr(
                    PhysAddr::new(base),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
                );
                base += offset;
            }

            for entry in pml4.iter_mut() {
                entry.set_addr(
                    PhysAddr::new(&PML3 as *const PageTable as u64),
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                );
                break;
            }

            let pml4_addr = &*pml4 as *const PageTable as u64;
            let phys_addr = PhysAddr::new(pml4_addr);
            Cr3::write(PhysFrame::containing_address(phys_addr), Cr3Flags::empty());

            sprintln!("{:?}", pml4);
        }

        // ALLOCATOR
        // let mapper: &mut dyn Mapper<Size4KiB> = None;
        // let frame_allocator: &mut dyn FrameAllocator<Size4KiB> = None;

        // let page_range = {
        //     let heap_start = VirtAddr::new(HEAP_START as u64);
        //     let heap_end = heap_start + HEAP_SIZE - 1u64;
        //     let heap_start_page = Page::containing_address(heap_start);
        //     let heap_end_page = Page::containing_address(heap_end);
        //     Page::range_inclusive(heap_start_page, heap_end_page)
        // };

        // for page in page_range {
        //     let frame = frame_allocator
        //         .allocate_frame()
        //         .ok_or(MapToError::FrameAllocationFailed)?;
        //     let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        //     mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        // }

        // ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);

        // TSS
        self.tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&STACK);
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

        self.pic = Some(&(*PICS));

        {
            let mut handle = self.pic.unwrap().lock();
            handle.initialize();
        }

        self.pit = Some(&(*PIT));

        {
            let mut handle = self.pit.unwrap().lock();
            handle.set_frequency(1000);
        }

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
