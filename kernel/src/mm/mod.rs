use {spin::RwLock, x86_64::structures::paging::PageTable};

mod heap;

#[no_mangle]
pub static PAGE_MAP_LEVEL_4: RwLock<PageTable> = RwLock::new(PageTable::new());
