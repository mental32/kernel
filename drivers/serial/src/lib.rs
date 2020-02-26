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

use core::fmt::{self, Write};

use {spin::Mutex, uart_16550::SerialPort};

#[cfg(feature = "default")]
use lazy_static::lazy_static;

/// Default serial io port for x86 systems.
pub const DEAFULT_SERIAL_IO_PORT: u16 = 0x3F8;


/// A newtype that represents a locked serial port.
pub struct SerialIO(Mutex<SerialPort>);

impl SerialIO {
    /// Get a new initialized SerialIO port.
    pub fn new() -> Self {
        let mut port = unsafe { SerialPort::new(DEAFULT_SERIAL_IO_PORT) };
        port.init();
        Self(Mutex::new(port))
    }

    /// Write formatted arguments into a serial port and panic on errors.
    #[inline]
    pub fn print_args(&self, args: fmt::Arguments) {
        self.0.lock().write_fmt(args).unwrap();
    }
}

#[cfg(feature = "default")]
lazy_static! {
    /// Global static refrence to a ``Mutex<SerialPort>`` using the ``DEAFULT_SERIAL_IO_PORT`` port.
    pub static ref GLOBAL_DEFAULT_SERIAL: SerialIO = {
        SerialIO::new()
    };
}


/// Emit some data without a newline.
#[macro_export]
macro_rules! sprint {
    ($($arg:tt)*) => ({

        #[cfg(not(feature = "default"))]
        compile_error!("Unable to infer handle from macro args or the `global-serial` feature is not enabled.");

        #[cfg(feature = "default")]
        {
            (*$crate::GLOBAL_DEFAULT_SERIAL).print_args(format_args!($($arg)*));
        }

    });

    ($handle:ident, $($arg:tt)*) => ({
        let mut handle: SerialIO = $handle;
        handle.print_args($($arg)*);
    })
}


/// Emit some data with a newline.
#[macro_export]
macro_rules! sprintln {
    ($($arg:tt)*) => ({
        $crate::sprint!($($arg)*);
        $crate::sprint!("\n");
    });

    ($handle:ident, $($arg:tt)*) => ({
        $crate::sprint!($handle, $($arg:tt)*);
        $crate::sprint!($handle, "\n");
    })
}
