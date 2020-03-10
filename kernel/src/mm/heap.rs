use {
    alloc::alloc::{AllocRef, GlobalAlloc, Layout},
    core::ops::Deref,
    core::ptr::{null_mut, NonNull},
};

use spin::Mutex;

#[cfg(feature = "buddy-alloc")]
use super::buddy::Heap as RawHeap;

#[cfg(feature = "bump-alloc")]
use super::bump::Heap as RawHeap;

#[cfg(feature = "linked-list-alloc")]
use linked_list_allocator::Heap as RawHeap;

#[derive(Debug)]
pub struct LockedHeap(Mutex<RawHeap>);

impl LockedHeap {
    /// Creates an empty heap
    pub const fn new() -> LockedHeap {
        LockedHeap(Mutex::new(RawHeap::empty()))
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
            .map_or(null_mut(), |(allocation, _)| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}
