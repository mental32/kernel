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

use {lazy_static::lazy_static, spin::Mutex};

/// A writer type alias helper that uses the DefaultBuffer.
pub type DefaultWriter = VGAWriter<'static, DefaultBuffer>;

/// A thread safe instance of the DefaultWriter.
pub type GlobalDefaultWriter = Mutex<DefaultWriter>;

lazy_static! {
    /// A lazy static to a GlobalDefaultWriter instance.
    pub static ref GLOBAL_WRITER: GlobalDefaultWriter =
        { Mutex::new(VGAWriter::new(buffer::DefaultBuffer::refrence()),) };
}

/// Like print!
#[macro_export]
macro_rules! vprint {
    ($writer:ident, $($arg:tt)*) => {
        x86_64::instructions::interrupts::without_interrupts(|| {
            $writer.write_fmt(format_args!($($arg)*)).unwrap();
        })
    };

    ($($arg:tt)*) => {
        {
            let mut writer = $crate::GLOBAL_WRITER.lock();
            $crate::vprint!(writer, $($arg)*);
        }
    }
}
