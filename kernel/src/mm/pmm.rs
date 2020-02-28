use smallvec::{smallvec, SmallVec};

use x86_64::{
    structures::paging::{
        PhysFrame, Size4KiB, UnusedPhysFrame, FrameAllocator, FrameDeallocator
    },
    PhysAddr,
};

use super::boot_frame::PhysFrameIter;

pub struct PhysFrameManager {
    frame_map: SmallVec<[u64; 1]>,
    frames: PhysFrameIter,
}

impl PhysFrameManager {
    pub fn new(frame_iter: PhysFrameIter, initial_frame_map: u64) -> Self {
        Self {
            frame_map: smallvec![initial_frame_map],
            frames: frame_iter,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysFrameManager {
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        self.frames.next().and_then(|range| unsafe {
            Some(UnusedPhysFrame::new(
                PhysFrame::from_start_address(PhysAddr::new(range.start.start_address().as_u64()))
                    .unwrap(),
            ))
        })
    }
}

impl FrameDeallocator<Size4KiB> for PhysFrameManager {
    fn deallocate_frame(&mut self, _frame: UnusedPhysFrame<Size4KiB>) {
    }
}

