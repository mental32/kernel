use {spin::RwLock, x86_64::structures::paging::PageTable};

#[cfg(feature = "buddy-alloc")]
pub(self) mod buddy;

#[cfg(feature = "bump-alloc")]
pub(self) mod bump;

pub mod boot_frame;
pub(self) mod heap;
mod pmm;
mod vmm;

pub use heap::LockedHeap;

#[no_mangle]
pub static mut PAGE_MAP_LEVEL_4: PageTable = PageTable::new();
