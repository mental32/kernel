use {
    alloc::alloc::{GlobalAlloc, Layout},
    core::ops::Deref,
    core::ptr::{null_mut, NonNull},
};

use spin::Mutex;

use super::buddy::Heap as RawHeap;

/// The virtual address of where the heap will be mapped.
pub const HEAP_START: usize = 0x_4444_4444_0000;

pub struct LockedHeap(Mutex<RawHeap>);

impl LockedHeap {
    /// Creates an empty heap
    pub const fn new() -> LockedHeap {
        LockedHeap(Mutex::new(RawHeap::new()))
    }
}

impl Deref for LockedHeap {
    type Target = Mutex<RawHeap>;

    fn deref(&self) -> &Mutex<RawHeap> {
        &self.0
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0
            .lock()
            .alloc(layout)
            .ok()
            .map_or(null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}
