#![no_std]
#![forbid(missing_docs)]

//! Primitive serial IO driver.
//!
//! Serial ports are a legacy communications port common on IBM-PC compatible
//! computers. Use of serial ports for connecting peripherals has largely been
//! deprecated in favor of USB and other modern peripheral interfaces, however
//! it is still commonly used in certain industries for interfacing with
//! industrial hardware such as CNC machines or commercial devices such as POS
//! terminals. Historically it was common for many dial-up modems to be
//! connected via a computer's serial port, and the design of the underlying
//! UART hardware itself reflects this.
//!
//! Serial ports are typically controlled by UART hardware. This is the
//! hardware chip responsible for encoding and decoding the data sent over the
//! serial interface. Modern serial ports typically implement the RS-232 standard,
//! and can use a variety of different connector interfaces. The DE-9 interface is
//! the one most commonly used connector for serial ports in modern systems.
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
    port.write_fmt(format_args!("\n"));
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
