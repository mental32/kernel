use {
    alloc::alloc::{AllocRef, GlobalAlloc, Layout},
    core::ops::Deref,
    core::ptr::{null_mut, NonNull},
};

use x86_64::structures::paging::{Page, PageTableFlags};
use x86_64::VirtAddr;

use spin::Mutex;

use crate::{info, mm};

#[cfg(feature = "buddy-alloc")]
use super::buddy::Heap as RawHeap;

#[cfg(feature = "bump-alloc")]
use super::bump::Heap as RawHeap;

#[cfg(feature = "linked-list-alloc")]
use linked_list_allocator::Heap as RawHeap;

pub const HEAP_START: u64 = 0x4444_4444_0000;
pub const HEAP_SIZE: u64 = 31 * 1024;

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
        let mut allocator = self.0.lock();

        match allocator.alloc(layout) {
            Ok((allocation, _)) => allocation.as_ptr(),
            Err(_) => {
                info!("HEAP ALLOCATION FAILED, ATTEMPTING TO EXTEND!");
                let mut memory_manager = mm!().try_lock().expect("Unable to unlock MM?!");

                assert!(layout.size() < (HEAP_SIZE as usize), "OH FOR FUCKS SAKE");
                let heap_size = HEAP_SIZE;

                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                let allocator_size = allocator.heap_size();
                let mut start = (((HEAP_START as usize) + allocator_size) & !(0x1000 - 1)) + 0x1000;

                info!(
                    "Allocating extended heap space... start=0x{:x?} (heap_size=0x{:x?}) size=0x{:x?}",
                    &start, allocator_size, &heap_size
                );

                while memory_manager
                    .page_is_mapped(Page::containing_address(VirtAddr::new(start as u64)))
                {
                    start += 0x1000;
                }

                for page in mm::page_range_inclusive(start as u64, (start as u64) + heap_size) {
                    info!("  Heap page: {:?}", page);
                    memory_manager.map_to(page, flags).unwrap().flush();
                }

                match allocator.extend_heap(start, heap_size as usize) {
                    Err(_) => null_mut(),
                    Ok(_) => allocator
                        .alloc(layout)
                        .map_or(null_mut(), |(allocation, _)| allocation.as_ptr()),
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(NonNull::new_unchecked(ptr), layout)
    }
}
