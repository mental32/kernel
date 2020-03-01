use core::{ptr, sync::atomic::{AtomicPtr, Ordering}};

use spin::RwLock;
use bit_field::{BitField, BitArray};

use smallvec::{smallvec, SmallVec};

use x86_64::{
    structures::paging::{
        page::PageRange, FrameAllocator, FrameDeallocator, PageSize, PhysFrame, Size4KiB,
        UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

use super::boot_frame::PhysFrameIter;

fn page_range_to_unused_physical_frame<S: PageSize>(
    range: PageRange,
) -> Result<UnusedPhysFrame<S>, ()> {
    let frame = PhysFrame::from_start_address(PhysAddr::new(range.start.start_address().as_u64()))?;

    Ok(unsafe { UnusedPhysFrame::new(frame) })
}

#[derive(Debug)]
pub struct BitMap<T: Sized + BitOrAssign + BitAndAssign + From<u8>> {
    pub start: AtomicPtr<T>,
    pub end: AtomicPtr<T>,
}

use core::ops::{BitOrAssign, BitAndAssign};

impl<T: Sized + BitOrAssign + BitAndAssign + From<u8> + BitField> BitMap<T> {
    pub fn new(start: AtomicPtr<T>, size: isize) -> Self {
        Self {
            end: AtomicPtr::new(unsafe { start.load(Ordering::SeqCst).offset(size) }),
            start,
        }
    }

    pub const fn null() -> Self {
        Self {
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn access<F>(&mut self, index: usize, f: F) where F: FnOnce(*mut T, usize) {
        if (index + (self.start.load(Ordering::SeqCst) as usize)) >= (self.end.load(Ordering::SeqCst) as usize) {
            panic!("Attempted out of bounds write.")
        }

        let (byte, bit) = ((index / 8), (index % 8));

        use core::convert::TryInto;

        unsafe {
            let ptr = self.start.load(Ordering::SeqCst).offset(byte.try_into().unwrap());
            f(ptr, bit);
        }
    }

    pub fn set(&mut self, index: usize) {
        self.access(index, |ptr, bit| {
            unsafe { (*ptr).set_bit(bit, true); }
        })
    }

    pub fn clear(&mut self, index: usize) {
        self.access(index, |ptr, bit| {
            unsafe { (*ptr).set_bit(bit, false); }
        })   
    }

    pub fn bits(&mut self) -> impl Iterator<Item = bool> {
        Bits {
            start: self.start.load(Ordering::SeqCst) as usize,
            stop: self.end.load(Ordering::SeqCst) as usize,
            cursor: 0,
        }
    }
}

pub struct Bits {
    start: usize,
    stop: usize,
    cursor: usize,
}

impl Bits {
    pub fn new(start: usize, stop: usize) -> Self {
        Self {
            start,
            stop,
            cursor: 0,
        }
    }
}

impl Iterator for Bits {
    type Item = bool;


    fn next(&mut self) -> Option<Self::Item> {
        let res = if (self.cursor / 8) >= self.stop {
            None
        } else {
            let ptr = ((self.cursor / 8) + self.start) as *mut u8;
            let value: u8 = unsafe { *ptr };
            Some(value.get_bit(self.cursor % 8))
        };

        if res.is_some() {
            self.cursor += 1;
        }

        res
    }
}

pub const INITIAL_PHYSFRAME_BITMAP_SIZE: usize = 128 * 1024;
pub static mut INITIAL_PHYSFRAME_BITMAP: [u8; INITIAL_PHYSFRAME_BITMAP_SIZE] = [0u8; INITIAL_PHYSFRAME_BITMAP_SIZE];

#[derive(Debug)]
pub struct PhysFrameManager {
    frames: PhysFrameIter,
    reusable_count: usize,
    frame_map: BitMap<u8>,
}

impl PhysFrameManager {
    pub fn new(frame_map: BitMap<u8>, frame_iter: PhysFrameIter) -> Self {
        Self {
            reusable_count: 0,
            frames: frame_iter,
            frame_map,
        }
    }

    fn first_reusable_frame(&mut self) -> Option<(usize, UnusedPhysFrame<Size4KiB>)> {
        if self.reusable_count > 0 {
            let base: u64 = 2;

            for (index, bit) in self.frame_map.bits().enumerate().filter(|(_, bit)| *bit) {
                let frame = match self.frames.nth_area(index) {
                    None => panic!("Frame marked free in bitmap but not available in physframe iterator."),
                    Some(range) => page_range_to_unused_physical_frame(range)
                        .expect("Unable to cast page range to unused phys frame."),
                };

                return Some((index, frame));
            }
        }

        None
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysFrameManager {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        let (index, frame) = match self.first_reusable_frame() {
            Some(reused) => reused,
            None => {
                let (index, range) = self.frames.next().expect("Ran out of physical frames!");

                unsafe { (index, page_range_to_unused_physical_frame(range).unwrap()) }
            }
        };

        self.frame_map.clear(index);

        Some(frame)
    }
}

impl FrameDeallocator<Size4KiB> for PhysFrameManager {
    fn deallocate_frame(&mut self, frame: UnusedPhysFrame<Size4KiB>) {
        match self
            .frames
            .index_of_range_that_starts_at_address(VirtAddr::new(frame.start_address().as_u64()))
        {
            Some(index) => {
                self.frame_map.set(index);
                self.reusable_count += 1
            }

            None => panic!("Unable to find the range of a frame to dealloc."),
        }
    }
}
