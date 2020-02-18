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
mod buffer;
mod character;
mod color;
mod cursor;
mod writer;

pub use {attribute::*, buffer::*, character::*, color::*, cursor::*, writer::*};
