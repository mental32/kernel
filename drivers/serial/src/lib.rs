#![no_std]
#![forbid(missing_docs)]

//! Primitive serial IO driver.
//!
//! # Examples
//!
//! ```
//! use serial::sprintln;
//! sprintln!("{:?}", "Hello, World!");
//! ```

use {core::fmt::Write, lazy_static::lazy_static, spin::Mutex, uart_16550::SerialPort};

/// Default serial io port for x86 systems.
pub const DEAFULT_SERIAL_IO_PORT: u16 = 0x3F8;

lazy_static! {
    /// Global static refrence to a ``Mutex<SerialPort>`` using the ``DEAFULT_SERIAL_IO_PORT`` port.
    pub static ref GLOBAL_DEFAULT_SERIAL: Mutex<SerialPort> = {
        let mut port = unsafe { SerialPort::new(DEAFULT_SERIAL_IO_PORT) };
        port.init();
        Mutex::new(port)
    };
}

/// Write formatted arguments into a serial port and panic on errors.
#[inline]
pub fn serial_println(port: &mut SerialPort, args: core::fmt::Arguments) {
    port.write_fmt(args).unwrap();
}

/// Helper macros that deal with serial IO.
#[macro_export]
macro_rules! sprintln {
    ($($arg:tt)*) => {
        let mut handle = (*$crate::GLOBAL_DEFAULT_SERIAL).lock();
        $crate::serial_println(&mut handle, format_args!($($arg)*));
        core::mem::drop(handle);
    };

    ($handle:ident, $($arg:tt)*) => {
        $crate::serial_println(&mut $handle, format_args!($($arg)*));
    }
}
