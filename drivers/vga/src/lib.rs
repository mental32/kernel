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

pub use {
    attribute::*, buffer::*, character::*, core::fmt::Write, cursor::*, result::*, writer::*,
};

/// Like println!
#[macro_export]
macro_rules! vprintln {
    ($writer:ident, $($arg:tt)*) => {
        let enabled = x86_64::instructions::interrupts::are_enabled();

        if enabled {
            x86_64::instructions::interrupts::disable();
        }

        let res = $writer.write_fmt(format_args!($($arg)*));

        if enabled {
            x86_64::instructions::interrupts::enable();
        }

        res.unwrap();
    };

    ($($arg:tt)*) => {
        let mut writer = VGAWriter::default();
        vprintln!(writer, $($arg)*);
    }
}
