#![no_std]
#![forbid(missing_docs)]

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
mod character;
mod writer;
mod result;

pub use {attribute::*, character::*, writer::*, result::*};

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}  // XXX: We should do something better here.
}
