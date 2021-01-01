use buddy_system_allocator::LockedHeapWithRescue;
use x86_64::{
    structures::paging::{Page, PageTableFlags},
    VirtAddr,
};

use mem::MemoryManager;

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeapWithRescue = LockedHeapWithRescue::new(|heap| {
    static mut HEAP_BASE_PTR: usize = 0x6666_0000_0000;
    const EXTENSION_AMOUNT: usize = 0x1000;

    // SAFETY: It's not.
    unsafe {
        let mapper = arch::prelude::memory_manager_ref();

        let page_range = {
            let heap_start = VirtAddr::new(HEAP_BASE_PTR as u64);
            let heap_end = heap_start + EXTENSION_AMOUNT - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            mapper
                .map(
                    page,
                    (PageTableFlags::PRESENT | PageTableFlags::WRITABLE).bits(),
                )
                .expect("Failed to map.")
                .flush()
        }

        HEAP_BASE_PTR += EXTENSION_AMOUNT;

        let (start, end) = (HEAP_BASE_PTR - EXTENSION_AMOUNT, HEAP_BASE_PTR);

        log::debug!(
            "(GLOBAL_ALLOCATOR) Mapping heap space {:#x}...{:#x} ({:?} bytes)",
            start,
            end,
            EXTENSION_AMOUNT,
        );

        heap.add_to_heap(start, end);
    }
});
