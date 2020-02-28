use x86_64::{
    structures::paging::{
        page::{PageRange, PageRangeInclusive},
        Page,
    },
    VirtAddr,
};


pub fn raw_page_range(start: u64, stop: u64) -> (Page, Page) {
    let start = VirtAddr::new(start);
    let end = VirtAddr::new(stop);
    let start_page = Page::containing_address(start);
    let end_page = Page::containing_address(end);
    (start_page, end_page)
}

pub fn page_range_exclusive(start: u64, stop: u64) -> PageRange {
    let (start_page, end_page) = raw_page_range(start, stop);
    Page::range(start_page, end_page)
}

pub fn page_range_inclusive(start: u64, stop: u64) -> PageRangeInclusive {
    let (start_page, end_page) = raw_page_range(start, stop);
    Page::range_inclusive(start_page, end_page)
}
