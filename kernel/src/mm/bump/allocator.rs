use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};

use alloc::alloc::{AllocErr, AllocRef, Layout};

use super::arena::Arena;
use crate::KernelResult;

#[derive(Debug)]
pub struct Heap {
    head: Arena,
    count: usize,
}

#[derive(Debug)]
pub struct ArenaIter {
    cursor: Option<AtomicPtr<Arena>>,
}

// TODO: This is the hackiest most unsafe code I've ever had to write. May the
// lord forgive my unpure soul as I have never done so many "on a whim"
// shennanigans in one function.
//
// Also Investigate if this can be much cleaner.
impl<'a> Iterator for ArenaIter {
    type Item = *mut Arena;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.as_ref() {
            None => None,
            Some(atomic_ptr) => {
                let arena_ptr = atomic_ptr.load(Ordering::SeqCst);

                if arena_ptr.is_null() {
                    self.cursor = None;
                    None
                } else {
                    let next_ptr = {
                        let a = unsafe { (&*arena_ptr) };
                        a.neighbour.load(Ordering::SeqCst)
                    };

                    self.cursor = Some(AtomicPtr::new(next_ptr));
                    Some(arena_ptr)
                }
            }
        }
    }
}

impl Heap {
    pub const fn empty() -> Self {
        Self {
            head: Arena::empty(),
            count: 0,
        }
    }

    pub fn arenas(&mut self) -> ArenaIter {
        ArenaIter {
            cursor: Some(AtomicPtr::new(&mut self.head as *mut _)),
        }
    }

    pub fn heap_size(&mut self) -> usize {
        self.arenas().map(|arena| unsafe { &*arena }.size()).sum()
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.head.init(heap_start, heap_size);
        self.count = 1;
    }

    pub unsafe fn extend_heap(&mut self, start: usize, size: usize) -> KernelResult<()> {
        for arena_ptr in self.arenas() {
            let mut arena = &mut *arena_ptr;

            if arena.neighbour.get_mut().is_null() {
                let sp = start as *mut Arena;
                sp.write(Arena::empty());
                let mut new_arena = &mut *sp;
                new_arena.init(start, size);
                arena.neighbour.store(sp, Ordering::SeqCst);
                return Ok(());
            }
        }

        panic!("OOF");
    }
}

unsafe impl AllocRef for Heap {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<(NonNull<u8>, usize), AllocErr> {
        for arena in self.arenas() {
            match (&mut *arena).alloc(layout) {
                Err(_) => continue,
                Ok(ptr) => return Ok((ptr, layout.size())),
            }
        }

        Err(AllocErr)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        for arena in self.arenas() {
            if (&mut *arena).contains(ptr.as_ptr() as usize) {
                (&mut *arena).dealloc(ptr, layout);
                return;
            }
        }

        panic!("Unable to deallocate {:?}", ptr);
    }
}
