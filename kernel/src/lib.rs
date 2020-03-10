#![no_std]
#![forbid(missing_docs)]
#![allow(unused_attributes)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(const_mut_refs)]
#![feature(never_type)]
#![feature(asm)]
#![feature(global_asm)]
#![feature(raw)]
#![feature(naked_functions)]
#![feature(option_expect_none)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("This kernel only supports the (AMD) x86_64 architecture.");

extern crate alloc;

mod dev;
mod gdt;
mod isr;
mod log;
mod mm;
mod net;
pub mod result;
mod sched;
mod smp;
mod state;
mod vifs;

use spin::Mutex;

use log::SystemLogger;

/// A static system logger for the kernel.
pub static SYSTEM_LOGGER: Mutex<SystemLogger> = Mutex::new(SystemLogger::new());

pub use result::*;

/// A basic eum of access permissions: read only, write only and read/write.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum AccessPermissions {
    /// The item is read only.
    ReadOnly = 0,

    /// The item is write only.
    WriteOnly,

    /// The item can be read from and written to.
    ReadWrite,
}

/// A macro used to quicky construct handles to ports.
#[macro_use]
#[macro_export]
macro_rules! pt {
    ($ln:expr) => { Port::<u8>::new($ln) };
    ($ln:expr, $size:ty) => { Port::<$size>::($ln) }
}

/// A helper macro to access the memory manager.
#[macro_use]
#[macro_export]
macro_rules! mm {
    () => {
        $crate::mm::MEMORY_MANAGER
    };
}

#[cfg(feature = "standalone")]
mod standalone;

#[cfg(feature = "standalone")]
pub use standalone::*;
