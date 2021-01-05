use core::fmt::Write;

use uart_16550::SerialPort;
use log::{Level, LevelFilter, Log, Metadata, Record};
use once_cell::unsync::OnceCell;
use spin::Mutex;

// -- SerialLogger

/// A newtype implementing `log::Log` for logging to a UART serial line.
#[derive(Default)]
pub(crate) struct SerialLogger(Mutex<OnceCell<SerialPort>>);

impl SerialLogger {
    /// Attempt to get a static reference to the global logger instance.
    pub fn global_ref() -> Option<&'static impl Log> {
        static UART_LOGGER: SerialLogger = SerialLogger(Mutex::new(OnceCell::new()));

        let mut cell = UART_LOGGER.0.try_lock()?;
        let default = || {
            let mut serial_port = unsafe { SerialPort::new(0x3F8) };
            serial_port.init();
            serial_port
        };

        cell.get_or_init(default);

        Some(&UART_LOGGER)
    }
}

impl Log for SerialLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if let Some(mut cell) = self.0.try_lock() {
            if let Some(port) = cell.get_mut() {
                let _ = port.write_fmt(format_args!("[{}] {}\n", record.level(), record.args()));
            }
        }
    }

    fn flush(&self) {}
}
