use core::sync::atomic::AtomicPtr;

use {spin::Mutex, x86_64::structures::paging::PageTable};

#[cfg(feature = "buddy-alloc")]
pub(self) mod buddy;

#[cfg(feature = "bump-alloc")]
pub(self) mod bump;

pub mod boot_frame;
pub(self) mod heap;
mod helpers;
mod manager;
pub mod pmm;

pub use {heap::LockedHeap, helpers::*, manager::*, pmm::PhysFrameManager};

#[repr(C, packed(4096))]
pub struct AlignedHole([u8; 4096]);

#[no_mangle]
#[link_section = ".extra.bss"]
pub static mut PML4_SPACE: AlignedHole = AlignedHole ([0u8; 4096]);

#[no_mangle]
pub static mut PML4_ADDR: *mut PageTable = unsafe { &mut PML4_SPACE } as *mut AlignedHole as *mut PageTable;

pub type MemoryManagerType = Mutex<MemoryManager<PhysFrameManager>>;

#[no_mangle]
#[link_section = ".extra.bss"]
pub static MEMORY_MANAGER: MemoryManagerType = Mutex::new(MemoryManager::new(unsafe { AtomicPtr::new(PML4_ADDR) }));
