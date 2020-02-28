use {spin::Mutex, x86_64::structures::paging::PageTable};

#[cfg(feature = "buddy-alloc")]
pub(self) mod buddy;

#[cfg(feature = "bump-alloc")]
pub(self) mod bump;

pub mod boot_frame;
pub(self) mod heap;
mod helpers;
mod manager;
mod pmm;

pub use {heap::LockedHeap, helpers::*, manager::*, pmm::PhysFrameManager};

use core::mem::{self, MaybeUninit};

pub static mut PML4_RAW: *mut [u8; 4096] = (&mut [0u8; 4096]) as *mut [u8; 4096];

use core::sync::atomic::AtomicPtr;

#[no_mangle]
pub static mut PML4_ADDR: AtomicPtr<PageTable> = unsafe { AtomicPtr::new(PML4_RAW as *mut PageTable) };

pub type MemoryManagerType = Mutex<MemoryManager<PhysFrameManager>>;

pub static MEMORY_MANAGER: MemoryManagerType = Mutex::new(MemoryManager::new(unsafe { &mut PML4_ADDR }));
