#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_fn)]
#![feature(const_mut_refs)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("This kernel only supports the (AMD) x86_64 architecture.");

extern crate alloc;

mod log;
mod dev;
mod gdt;
mod isr;
mod mm;
mod result;
mod sched;
mod state;
mod vfs;

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
    () => ($crate::mm::MEMORY_MANAGER)
}

#[cfg(feature = "standalone")]
mod standalone;

#[cfg(feature = "standalone")]
pub use standalone::*;
