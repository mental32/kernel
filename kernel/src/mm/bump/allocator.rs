use core::{
    cmp::{max, min},
    fmt,
    mem::size_of,
    ptr,
    ptr::NonNull,
};

use alloc::alloc::{Alloc, AllocErr, GlobalAlloc, Layout};

#[derive(Debug)]
pub struct Heap {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

use serial::sprintln;

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

impl Heap {
    /// Creates a new empty bump allocator.
    pub const fn empty() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
        sprintln!("{:?}", self);
    }
}

unsafe impl Alloc for Heap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let alloc_start = align_up(self.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return Err(AllocErr),
        };

        if alloc_end > self.heap_end {
            Err(AllocErr)
        } else {
            self.next = alloc_end;
            self.allocations += 1;
            sprintln!("{:?}", alloc_start as *mut u8);
            Ok(NonNull::new(alloc_start as *mut u8).unwrap())
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.allocations -= 1;
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }
}
