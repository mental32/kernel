// // use core::ptr::NonNull;
// // use core::sync::atomic::{AtomicPtr, Ordering};

// // use alloc::alloc::{AllocError, Allocator, Layout};

// // use super::arena::Arena;
// // use crate::KernelResult;

// // #[derive(Debug)]
// // pub struct Heap {
// //     head: Arena,
// //     count: usize,
// // }

// // #[derive(Debug)]
// // pub struct ArenaIter {
// //     cursor: Option<AtomicPtr<Arena>>,
// // }

// // // TODO: This is the hackiest most unsafe code I've ever had to write. May the
// // // lord forgive my unpure soul as I have never done so many "on a whim"
// // // shennanigans in one function.
// // //
// // // Also Investigate if this can be much cleaner.
// // impl<'a> Iterator for ArenaIter {
// //     type Item = *mut Arena;

// //     fn next(&mut self) -> Option<Self::Item> {
// //         match self.cursor.as_ref() {
// //             None => None,
// //             Some(atomic_ptr) => {
// //                 let arena_ptr = atomic_ptr.load(Ordering::SeqCst);

// //                 if arena_ptr.is_null() {
// //                     self.cursor = None;
// //                     None
// //                 } else {
// //                     let next_ptr = {
// //                         let a = unsafe { (&*arena_ptr) };
// //                         a.neighbour.load(Ordering::SeqCst)
// //                     };

// //                     self.cursor = Some(AtomicPtr::new(next_ptr));
// //                     Some(arena_ptr)
// //                 }
// //             }
// //         }
// //     }
// // }

// // impl Heap {
// //     pub const fn empty() -> Self {
// //         Self {
// //             head: Arena::empty(),
// //             count: 0,
// //         }
// //     }

// //     pub fn arenas(&self) -> ArenaIter {
// //         ArenaIter {
// //             cursor: Some(AtomicPtr::new(&self.head as *const _ as *mut _)),
// //         }
// //     }

// //     pub fn heap_size(&mut self) -> usize {
// //         self.arenas().map(|arena| unsafe { &*arena }.size()).sum()
// //     }

// //     pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
// //         self.head.init(heap_start, heap_size);
// //         self.count = 1;
// //     }

// //     pub unsafe fn extend_heap(&mut self, start: usize, size: usize) -> KernelResult<()> {
// //         for arena_ptr in self.arenas() {
// //             let mut arena = &mut *arena_ptr;

// //             if arena.neighbour.get_mut().is_null() {
// //                 let sp = start as *mut Arena;
// //                 sp.write(Arena::empty());
// //                 let mut new_arena = &mut *sp;
// //                 new_arena.init(start, size);
// //                 arena.neighbour.store(sp, Ordering::SeqCst);
// //                 return Ok(());
// //             }
// //         }

// //         panic!("OOF");
// //     }
// // }

// // unsafe impl Allocator for Heap {
// //     fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
// //         for arena in self.arenas() {
// //             match unsafe { (&mut *arena).alloc(layout) } {
// //                 Err(_) => continue,
// //                 Ok(ptr) => return Ok(ptr),
// //             }
// //         }

// //         Err(AllocError)
// //     }

// //     unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
// //         for arena in self.arenas() {
// //             if (&mut *arena).contains(ptr.as_ptr() as usize) {
// //                 (&mut *arena).dealloc(ptr, layout);
// //                 return;
// //             }
// //         }

// //         panic!("Unable to deallocate {:?}", ptr);
// //     }
// // }

// use core::mem::size_of;
// use core::ptr::{null_mut, NonNull};
// use core::sync::atomic::{AtomicPtr, Ordering};

// use alloc::alloc::{AllocError, Layout};

// #[derive(Debug)]
// pub struct Arena {
//     heap_start: usize,
//     heap_end: usize,
//     next: usize,
//     allocations: usize,
//     pub neighbour: AtomicPtr<Arena>,
// }

// fn align_up(addr: usize, align: usize) -> usize {
//     (addr + align - 1) & !(align - 1)
// }

// impl Arena {
//     /// Creates a new empty bump allocator.
//     pub const fn empty() -> Self {
//         Self {
//             heap_start: 0,
//             heap_end: 0,
//             next: 0,
//             allocations: 0,
//             neighbour: AtomicPtr::new(null_mut()),
//         }
//     }

//     pub fn contains(&self, addr: usize) -> bool {
//         ((self.heap_start)..(self.heap_end)).contains(&addr)
//     }

//     #[inline]
//     pub fn size(&self) -> usize {
//         self.heap_end - self.heap_start
//     }

//     /// Initializes the bump allocator with the given heap bounds.
//     ///
//     /// This method is unsafe because the caller must ensure that the given
//     /// memory range is unused. Also, this method must be called only once.
//     pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
//         self.heap_start = heap_start;
//         self.heap_end = heap_start + heap_size;
//         self.next = heap_start + size_of::<Self>();
//     }

//     pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//         let alloc_start = align_up(self.next, layout.align());
//         let alloc_end = match alloc_start.checked_add(layout.size()) {
//             Some(end) => end,
//             None => return Err(AllocError),
//         };

//         if alloc_end > self.heap_end {
//             Err(AllocError)
//         } else {
//             self.next = alloc_end;
//             self.allocations += 1;
//             let sl = unsafe { core::slice::from_raw_parts(alloc_start as *mut u8, layout.size()) };
//             let ptr = NonNull::from(sl);
//             Ok(ptr)
//         }
//     }

//     pub unsafe fn dealloc(&mut self, _ptr: NonNull<u8>, _layout: Layout) {
//         self.allocations -= 1;
//         if self.allocations == 0 {
//             self.next = self.heap_start + size_of::<Self>();
//         }
//     }
// }

