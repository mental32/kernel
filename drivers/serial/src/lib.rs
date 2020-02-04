#![no_std]
// #![forbid(missing_docs)]

use {core::fmt::Write, lazy_static::lazy_static, spin::Mutex, uart_16550::SerialPort};

pub const SERIAL_IO_PORT: u16 = 0x3F8;

lazy_static! {
    pub static ref DEFAULT_SERIAL: Mutex<SerialPort> = {
        let mut port = unsafe { SerialPort::new(SERIAL_IO_PORT) };
        port.init();
        Mutex::new(port)
    };
}

#[inline]
pub fn serial_println(port: &mut SerialPort, args: core::fmt::Arguments) {
    port.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! sprintln {
    ($($arg:tt)*) => {
        let mut handle = (*$crate::DEFAULT_SERIAL).lock();
        $crate::serial_println(&mut handle, format_args!($($arg)*));
        core::mem::drop(handle);
    }
}
