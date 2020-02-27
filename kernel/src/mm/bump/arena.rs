use core::ptr::NonNull;

use alloc::alloc::{AllocErr, Layout};

#[derive(Debug)]
pub struct Arena {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
    pub neighbour: Option<&'static mut Arena>,
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

impl Arena {
    /// Creates a new empty bump allocator.
    pub const fn empty() -> Self {
        Self {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
            neighbour: None,
        }
    }

    pub fn contains(&self, addr: usize) -> bool {
        ((self.heap_start)..(self.heap_end)).contains(&addr)
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
        serial::sprintln!("{:x?}", self);
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
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
            Ok(NonNull::new(alloc_start as *mut u8).unwrap())
        }
    }

    pub unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
        self.allocations -= 1;
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }
}
