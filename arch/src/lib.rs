#![no_std]
#![feature(global_asm)]
#![feature(asm)]
#![feature(abi_x86_interrupt)]

cfg_if::cfg_if! {
    if #[cfg(feature = "x86-64")] {
        mod x86_64;
        pub use self::x86_64::*;
    } else {
        compile_error!("Unsupported architecture!");
    }
}
