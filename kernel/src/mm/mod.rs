use {spin::RwLock, x86_64::structures::paging::PageTable};

pub(self) mod buddy;
pub(self) mod bump;
pub(self) mod heap;

pub use heap::{LockedHeap, HEAP_START};

#[no_mangle]
pub static PAGE_MAP_LEVEL_4: RwLock<PageTable> = RwLock::new(PageTable::new());
