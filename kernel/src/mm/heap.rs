mod buddy;

use {
    alloc::alloc::{GlobalAlloc, Layout},
    core::ptr::null_mut,
};

/// The virtual address of where the heap will be mapped.
pub const HEAP_START: usize = 0x_4444_4444_0000;

/// The amount of memory the heap will be grown by in typical cases.
pub const HEAP_STEP: usize = 4 * 1024; // 4 KiB

#[global_allocator]
static ALLOCATOR: buddy::Heap = buddy::Heap::new();

// #[global_allocator]
// static DUMMY: Dummy = Dummy {};

// pub struct Dummy;

// unsafe impl GlobalAlloc for Dummy {
//     unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
//         null_mut()
//     }

//     unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
//         panic!("dealloc should be never called")
//     }
// }

#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
