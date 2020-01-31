#![no_std]
#![forbid(missing_docs)]
#![allow(unused_macros)]

//! VGA terminal driver.
//!
//! This crate provides an implementation of a VGA mode text driver
//! to be used by the kernel.
//!
//! # Examples
//!
//! Let's see hello world:
//! ```
//! use drivers::vga::{VGAWriter, vprintln};
//!
//! fn routine() {
//!     let mut writer = VGAWriter::default();
//!     vprintln!(writer, "Hello, World!");
//! }
//! ```

mod attribute;
mod buffer;
mod character;
mod cursor;
mod result;
mod writer;

pub use {attribute::*, buffer::*, character::*, cursor::*, result::*, writer::*};

macro_rules! vprintln {
    ($writer:expr, $($arg:tt)*) => {
        #[cfg(target_arch = "x86_64")]
        use x86_64::instructions::interrupts;

        interupts::without_interrupts(|| {
            $writer.write_fmt(format_args!($($arg)*))
        });
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {} // XXX: We should do something better here.
}
