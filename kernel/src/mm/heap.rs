use {
    alloc::alloc::{AllocRef, GlobalAlloc, Layout},
    core::ops::Deref,
    core::ptr::{null_mut, NonNull},
};

use spin::Mutex;

use super::buddy::Heap as RawHeap;
// use super::bump::Heap as RawHeap;
// use linked_list_allocator::Heap as RawHeap;

/// The virtual address of where the heap will be mapped.
pub const HEAP_START: usize = 0x_4444_4444_0000;

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
        let res = self.0.lock().alloc(layout);

        use serial::sprintln;

        sprintln!("{:?}", res);

        res.ok()
            .map_or(null_mut(), |allocation| allocation.as_ptr())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}
