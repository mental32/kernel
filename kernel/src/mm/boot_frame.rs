//! Raw Boot info frame allocator.

use core::cmp::{max, min};

use {multiboot2::BootInformation, smallvec::SmallVec};

use x86_64::{
    structures::paging::{
        page::{PageRange, PageRangeInclusive},
        Page,
    },
    VirtAddr,
};

use super::{page_range_exclusive, page_range_inclusive};

pub fn elf_areas(boot_info: &BootInformation) -> SmallVec<[(u64, u64); 16]> {
    let elf_tag = boot_info.elf_sections_tag().unwrap();

    let mut raw_section_page_ranges = elf_tag
        .sections()
        .map(|section| {
            Some(page_range_inclusive(
                section.start_address(),
                section.end_address(),
            ))
        })
        .collect::<SmallVec<[Option<PageRangeInclusive>; 16]>>();

    // Fuse contiguous ranges.
    for index in 0..(raw_section_page_ranges.len() - 1) {
        if raw_section_page_ranges[index].is_none() {
            continue;
        }

        let left = raw_section_page_ranges[index].unwrap();
        let right = raw_section_page_ranges[index + 1].unwrap();

        if left.end == right.start {
            raw_section_page_ranges[index + 1] = Some(Page::range_inclusive(left.start, right.end));
            raw_section_page_ranges[index] = None;
        }
    }

    raw_section_page_ranges
        .iter()
        .filter(|range| range.is_some())
        .map(|range| range.unwrap())
        .map(|range| {
            (
                range.start.start_address().as_u64(),
                range.end.start_address().as_u64() + range.end.size(),
            )
        })
        .collect::<SmallVec<[(u64, u64); 16]>>()
}

pub fn find_holes(hole_size: usize, boot_info: &BootInformation) -> PhysFrameIter {
    let memory_areas = boot_info
        .memory_map_tag()
        .unwrap()
        .memory_areas()
        .map(|area| (area.start_address(), area.end_address()))
        .collect::<SmallVec<[(u64, u64); 32]>>();

    PhysFrameIter {
        elf_areas: elf_areas(boot_info),
        hole_size,
        memory_areas,
        count: 0,
        area_index: 0,
        section_index: 0,
    }
}

#[derive(Debug)]
pub struct PhysFrameIter {
    hole_size: usize,
    area_index: usize,
    section_index: usize,
    count: usize,
    memory_areas: SmallVec<[(u64, u64); 32]>,
    elf_areas: SmallVec<[(u64, u64); 16]>,
}

impl PhysFrameIter {
    pub fn nth_area(&mut self, index: usize) -> Option<PageRange> {
        let (preserved_area_index, preserved_section_index, preserved_counter) =
            (self.area_index, self.section_index, self.count);

        self.area_index = 0;
        self.section_index = 0;
        self.count = 0;

        let range = self.nth(index - 1).and_then(|(_, range)| Some(range));

        self.count = preserved_counter;
        self.area_index = preserved_area_index;
        self.section_index = preserved_section_index;

        range
    }

    pub fn index_of_range_that_starts_at_address(&mut self, base: VirtAddr) -> Option<usize> {
        let (preserved_area_index, preserved_section_index, preserved_counter) =
            (self.area_index, self.section_index, self.count);

        self.area_index = 0;
        self.section_index = 0;
        self.count = 0;

        let mut res = None;
        loop {
            match self.next() {
                None => break,
                Some((index, range)) => {
                    if range.start.start_address() == base {
                        res = Some(index);
                        break;
                    }
                }
            }
        }

        self.count = preserved_counter;
        self.area_index = preserved_area_index;
        self.section_index = preserved_section_index;

        res
    }
}

impl Iterator for PhysFrameIter {
    type Item = (usize, PageRange);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.area_index >= self.memory_areas.len() {
                return None;
            }

            let (area_start, area_end) = self.memory_areas[self.area_index];

            let (section_start, section_end) = match (area_start..area_end)
                .step_by(self.hole_size)
                .nth(self.section_index)
            {
                None => {
                    self.area_index += 1;
                    self.section_index = 0;
                    continue;
                }

                Some(section_start) => {
                    self.section_index += 1;
                    (
                        (section_start as u64),
                        (section_start as u64) + (self.hole_size as u64),
                    )
                }
            };

            let section = page_range_exclusive(section_start as u64, section_end as u64);

            if self
                .elf_areas
                .iter()
                .map(|area| area.clone())
                .any(|(start, stop)| max(section_start, start) <= min(section_end, stop))
            {
                continue;
            }

            self.count += 1;
            return Some((self.count, section));
        }
    }
}
