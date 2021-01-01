//! Kernel memory management utilities.

#![no_std]
#![feature(allocator_api)]
#![feature(min_const_generics)]
#![feature(unchecked_math)]

use chunks::MemoryChunks;
use multiboot2::BootInformation;
use structures::paging;
use x86_64::{PhysAddr, structures::{self, paging::{FrameAllocator, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB, mapper::{MapToError, MapperFlush}}}};

// extern crate alloc;

// mod bitmap;
// mod paging;
// mod bump;

pub mod boot_frame;
pub mod chunks;

/// Used as a buffer to store areas of memory market available.
///
/// Basically a struct with an array of `N` areas `(usize, usize)` (start, end)
/// representing available memory regions.
///
/// I calculated `N` by launching QEMU with as much memory as I could `47GiB`
/// and then counted the amount of available memory regions described by the
/// multiboot memory map tag and doubled it (in my case it was `3` hence `6`)
pub type PhysicalMemory = MemoryChunks<{ 6 }>;

/// A cursor for iterating over physframe chunks.
#[derive(Debug, Default, Clone)]
pub struct PhysFrameCursor {
    /// Which area we're looking in (see `PhysicalMemory`)
    pub chunk_idx: usize,

    /// The offset into the chunk that we're looking at.
    pub section_idx: usize,
}

/// A type used to implement a `FrameAllocator` over some `PhysicalMemory`.
#[derive(Debug)]
pub struct PhysFrameAlloc {
    pub memory: PhysicalMemory,
    pub physframe_cursor: PhysFrameCursor,
}

unsafe impl FrameAllocator<Size4KiB> for PhysFrameAlloc {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame_size = <Size4KiB as PageSize>::SIZE as usize;

        let PhysFrameCursor {
            mut chunk_idx,
            mut section_idx,
        } = self.physframe_cursor;

        chunk_idx %= self.memory.capacity();

        loop {
            let chunk = self
                .memory
                .get(chunk_idx)
                .map(|(a, b)| (a as u64)..(b as u64))
                .expect("Out of bounds chunk read..."); // Should never actually be `None`

            let frame = chunk
                .step_by(frame_size)
                .nth(section_idx)
                .map(PhysAddr::new)
                .map(PhysFrame::containing_address);

            if let Some(frame) = frame {
                section_idx += 1;

                self.physframe_cursor = PhysFrameCursor {
                    chunk_idx,
                    section_idx,
                };

                break Some(frame);
            } else {
                // We're done with this chunk, wrap over to the next one.
                chunk_idx = chunk_idx.wrapping_add(1) % self.memory.capacity();
                section_idx = 0;
            }
        }
    }
}


/// Trait used to abstract over memory managers for different architectures.
pub trait MemoryManager {
    // TODO: make the argument types non-reliant on `x86_64` crate

    fn identity_map(&mut self, address: usize, flags: u64) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>;

    fn map_to(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame,
        flags: u64,
        frame_allocator: &mut PhysFrameAlloc,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>;

    fn map(
        &mut self,
        page: Page<Size4KiB>,
        flags: u64,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>;

    fn unmap(&mut self, page: Page<Size4KiB>);

    fn initialize(&mut self, info: &BootInformation);
}
