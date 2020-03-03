use core::sync::atomic::{AtomicPtr, Ordering};
use core::fmt::Debug;

use x86_64::{
    structures::paging::{
        mapper::{MapToError, Mapper, MapperFlush},
        FrameAllocator, FrameDeallocator, OffsetPageTable, Page, PageTable, PageTableFlags,
        Size4KiB, UnusedPhysFrame,
    },
    VirtAddr,
};

use crate::sched::StackBounds;

#[derive(Debug)]
pub struct MemoryManager<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> {
    pub pml4_addr: AtomicPtr<PageTable>,
    mapper: Option<OffsetPageTable<'static>>,
    falloc: Option<F>,
}

impl<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> MemoryManager<F> {
    pub const fn new(pml4_addr: AtomicPtr<PageTable>) -> Self {
        Self {
            pml4_addr,
            mapper: None,
            falloc: None,
        }
    }

    pub unsafe fn initialize(&mut self, virt_offset: VirtAddr, falloc: F) {
        self.falloc = Some(falloc);
        self.mapper = Some(OffsetPageTable::new(
            &mut *self.pml4_addr.load(Ordering::SeqCst),
            virt_offset,
        ));
    }

    /// Allocate a new stack to be used by a thread.
    pub fn allocate_thread_stack(
        &mut self,
        size_in_pages: u64,
    ) -> Result<StackBounds, MapToError<Size4KiB>> {
        static STACK_ALLOC_NEXT: AtomicU64 = AtomicU64::new(0x_5555_5555_0000);

        let guard_page_start = STACK_ALLOC_NEXT.fetch_add(
            (size_in_pages + 1) * Page::<Size4KiB>::SIZE,
            Ordering::SeqCst,
        );

        let guard_page = Page::from_start_address(VirtAddr::new(guard_page_start))
            .expect("`STACK_ALLOC_NEXT` not page aligned");

        let stack_start = guard_page + 1;
        let stack_end = stack_start + size_in_pages;

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        sprintln!("{:?}", (&stack_start, &stack_end, &guard_page));

        for page in Page::range(stack_start, stack_end) {
            sprintln!("Mapping {:?}", &page);
            self.map_to(page, flags)?.flush();
        }

        use serial::sprintln;

        sprintln!(
            "Allocated new thread stack {:?}",
            Page::range(stack_start, stack_end)
        );

        Ok(StackBounds::new(
            stack_start.start_address(),
            stack_end.start_address(),
        ))
    }

    pub fn map_to(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let falloc = self.falloc.as_mut().unwrap();
        let frame = falloc.allocate_frame().unwrap();

        self.mapper
            .as_mut()
            .unwrap()
            .map_to(page, frame, flags, falloc)
    }
}

unsafe impl<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> FrameAllocator<Size4KiB>
    for MemoryManager<F>
{
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        self.falloc.as_mut().unwrap().allocate_frame()
    }
}
