use core::{convert::TryInto, mem::size_of};

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
                page::{PageRangeInclusive, Size4KiB},
                page_table::{PageTable, PageTableEntry, PageTableFlags},
                Page,
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
    mm::{LockedHeap, HEAP_START, PAGE_MAP_LEVEL_4},
    result::{KernelException, Result as KernelResult},
    GLOBAL_ALLOCATOR,
};

const TWO_MIB: usize = 0x200000;
const HEAP_SIZE: usize = 100 * 1024;

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
    heap: Option<&'static LockedHeap>,
    selectors: Selectors,
    // Tables
    gdt: ExposedGlobalDescriptorTable,
    idt: InterruptDescriptorTable,
    tss: TaskStateSegment,
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

            pic: None,
            pit: None,
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

        // Identity map all physical memory up to 2GiB
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
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );

        let pml4_addr = &*pml4 as *const PageTable as u64;
        let phys_addr = PhysAddr::new(pml4_addr);
        Cr3::write(PhysFrame::containing_address(phys_addr), Cr3Flags::empty());

        sprintln!("{:?}", pml4);

        // ALLOCATOR

        // Heap page range
        let page_range: PageRangeInclusive<Size4KiB> = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        // let mapper: &mut dyn Mapper<Size4KiB> = None;
        // let frame_allocator: &mut dyn FrameAllocator<Size4KiB> = None;

        // for page in page_range {
        //     let frame = frame_allocator
        //         .allocate_frame()
        //         .ok_or(MapToError::FrameAllocationFailed)?;
        //     let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        //     mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        // }

        // GLOBAL_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); // Statically size the heap to 100Kib

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
