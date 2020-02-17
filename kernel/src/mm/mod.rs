use {spin::RwLock, x86_64::structures::paging::PageTable};

#[cfg(feature = "buddy-alloc")]
pub(self) mod buddy;

#[cfg(feature = "bump-alloc")]
pub(self) mod bump;

pub(self) mod heap;

pub use heap::LockedHeap;

#[no_mangle]
pub static PAGE_MAP_LEVEL_4: RwLock<PageTable> = RwLock::new(PageTable::new());
