//! Mechanisms used to identify "boot frames"
//!
//! Boot frames are physical frames of a certain size that are available to be
//! used by the operating system.

use core::{iter::Filter, marker::PhantomData};
use core::ops::Range;

use multiboot2::{BootInformation, MemoryArea, MemoryAreaIter, MemoryAreaType};

use x86_64::{PhysAddr, structures::paging::{Page, PageSize, PhysFrame, Size4KiB, page::{PageRange, PageRangeInclusive}}};
use x86_64::VirtAddr;

use crate::chunks::MemoryChunks;

/// Used to iterate over holes of a certain `SIZE` from chunks specified in a multiboot2 memory area tag.
#[derive(Debug)]
pub struct PhysFrameIter<'a, S: PageSize = Size4KiB> {
    memory_area_iter: MemoryAreaIter<'a>,
    memory_area: Option<&'a MemoryArea>,
    section_index: usize,
    _phantom: PhantomData<S>,
}

impl<'a, S: PageSize> PhysFrameIter<'a, S> {
    /// Create a new physical frame iterator from the memory map tag of some `multiboot2::BootInformation`.
    ///
    /// The iterator does not cycle, its a single pass over the memory areas and it skips over ELF sections.
    pub fn new(
        bootinfo: &'a BootInformation,
    ) -> Option<Self> {
        let memory_area_iter = bootinfo.memory_map_tag()?.all_memory_areas();

        let iter = Self {
            memory_area_iter,
            memory_area: None,
            section_index: 0,
            _phantom: PhantomData,
        };

        Some(iter)
    }

    /// Get the current `memory_area`, `None` if there aren't any more.
    #[inline]
    fn memory_area(&mut self) -> Option<&'a MemoryArea> {
        if self.memory_area.is_none() {
            let area = loop {
                let area = self.memory_area_iter.next()?;

                if area.typ() == MemoryAreaType::Available {
                    break area;
                }
            };

            self.memory_area.replace(area);
        }

        self.memory_area.clone()
    }

    #[inline]
    pub fn next(&mut self) -> Option<PhysFrame> {
        let mut area;
        let size = <S as PageSize>::SIZE as usize;

        // Find the first chunk in an available memory area
        // that fits `SIZE` requirements and **is not** also
        // an ELF section.
        loop {
            // Get the current target memory area, if `None`
            // source it from the `memory_area_iter`.
            area = self.memory_area()?;
            let range = (area.start_address()..area.end_address());

            if let Some(section) = range.step_by(size).nth(self.section_index) {
                self.section_index += 1;

                let frame = PhysFrame::containing_address(PhysAddr::new(section));

                break Some(frame);
            } else {
                let _ = self.memory_area.take();
                area = self.memory_area()?;
                self.section_index = 0;
            }
        }
    }

    /// Advance the iterator and produce the next valid `PageRange` of `SIZE` that is approved by `f(range)`.
    #[inline]
    pub fn filter_next<F>(&mut self, f: Option<F>) -> Option<PhysFrame> where F: Fn(&Range<u64>) -> bool {
        let size = <S as PageSize>::SIZE;

        loop {
            let frame = self.next()?;

            let section = frame.start_address().as_u64();
            let section_range = section..(section + size);

            if f.as_ref().map(|f| f(&section_range)).unwrap_or(false) {
                continue;
            } else {
                break Some(frame);
            }
        }
    }
}
