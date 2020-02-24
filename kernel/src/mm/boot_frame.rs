//! Raw Boot info frame allocator.

use {multiboot2::BootInformation, smallvec::SmallVec};

use x86_64::{
    instructions::{
        segmentation::set_cs,
        tables::{lidt, load_tss, DescriptorTablePointer},
    },
    registers::control::{Cr3, Cr3Flags},
    structures::{
        gdt::{Descriptor, DescriptorFlags, SegmentSelector},
        idt::InterruptDescriptorTable,
        paging::{
            frame::PhysFrame,
            page::{PageRange, PageRangeInclusive},
            page_table::{PageTable, PageTableFlags},
            Page,
        },
        tss::TaskStateSegment,
    },
    PhysAddr, VirtAddr,
};

fn raw_page_range(start: u64, stop: u64) -> (Page, Page) {
    let start = VirtAddr::new(start);
    let end = VirtAddr::new(stop);
    let start_page = Page::containing_address(start);
    let end_page = Page::containing_address(end);
    (start_page, end_page)
}

fn page_range_exclusive(start: u64, stop: u64) -> PageRange {
    let (start_page, end_page) = raw_page_range(start, stop);
    Page::range(start_page, end_page)
}

fn page_range_inclusive(start: u64, stop: u64) -> PageRangeInclusive {
    let (start_page, end_page) = raw_page_range(start, stop);
    Page::range_inclusive(start_page, end_page)
}

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

pub fn find_first_non_overlapping_free_area(
    size: usize,
    boot_info: &BootInformation,
) -> (u64, u64) {
    // Heap page range
    let section_page_ranges = elf_areas(boot_info);

    let memory_map = boot_info.memory_map_tag().unwrap();
    let heap_range = {
        let mut heap_range = None;

        // The memory_areas method name is a little deceptive.
        //
        // It'll only yield areas that are listed as available
        // By the multiboot tag, not all areas that get listed.
        for area in memory_map.memory_areas() {
            let area_start: usize = area.start_address() as usize;
            let area_end: usize = area.end_address() as usize;

            for section_start in (area_start..area_end).step_by(size) {
                let section_end = section_start + size;
                let section = page_range_exclusive(section_start as u64, section_end as u64);

                if section_page_ranges.iter().all(|(start, stop)| {
                    ((section_start as u64) <= *stop) && (*start >= (section_end as u64))
                }) {
                    heap_range = Some(section);
                    break;
                }
            }

            if heap_range.is_some() {
                break;
            }
        }

        match heap_range {
            None => panic!("Not enough memory to allocate a heap!"),
            Some(range) => range,
        }
    };

    let mut heap_start = heap_range.start.start_address().as_u64();

    // When an allocation call returns 0x00 as a pointer
    // It's treated as an allocation failure, even though
    // In some cases an area starting at addr 0x00 is
    // Possible.
    if heap_start == 0 {
        heap_start += 8; // 8 byte alignment
    }

    let heap_end = heap_range.end.start_address().as_u64();

    (heap_start, heap_end)
}
