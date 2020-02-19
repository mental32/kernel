use core::ptr::NonNull;

use alloc::alloc::{Alloc, AllocErr, Layout};

use super::arena::Arena;

use serial::sprintln;

#[derive(Debug)]
pub struct Heap {
    head: Arena,
    count: usize,
}

impl Heap {
    pub const fn empty() -> Self {
        Self {
            head: Arena::empty(),
            count: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.head.init(heap_start, heap_size);
        self.count = 1;
    }
}

unsafe impl Alloc for Heap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let mut head = &mut self.head;

        if let Ok(addr) = head.alloc(layout) {
            return Ok(addr);
        }

        let mut neighbour = head.neighbour.as_mut();
        for _ in (0..(self.count)) {
            let mut arena = match neighbour {
                None => break,
                v => v.unwrap(),
            };

            let res = arena.alloc(layout);
            serial::sprintln!("=> {:?}", res);

            if let Ok(addr) = res {
                return Ok(addr);
            }

            neighbour = arena.neighbour.as_mut();
        }

        Err(AllocErr)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let mut head = &mut self.head;

        if head.contains(ptr.as_ptr() as usize) {
            head.dealloc(ptr, layout);
            return;
        }

        let mut neighbour = head.neighbour.as_mut();
        for _ in (0..(self.count)) {
            let mut arena = match neighbour {
                None => break,
                v => v.unwrap(),
            };

            if arena.contains(ptr.as_ptr() as usize) {
                arena.dealloc(ptr, layout);
                return;
            }

            neighbour = arena.neighbour.as_mut();
        }

        panic!("Unable to deallocate {:?}", ptr);
    }
}
