/// Log an information message.
#[macro_export]
#[macro_use]
macro_rules! info {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().info($message)
    };

    ($($arg:tt)*) => {
        $crate::SYSTEM_LOGGER.lock().fmt_info(format_args!($($arg)*))
    }
}

/// Log a warning.
#[macro_export]
#[macro_use]
macro_rules! warn {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().warn($message)
    };

    ($($arg:tt)*) => {
        $crate::SYSTEM_LOGGER.lock().fmt_warn(format_args!($($arg)*))
    }
}

/// Log an error.
#[macro_export]
#[macro_use]
macro_rules! error {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().error($message)
    };

    ($($arg:tt)*) => {
        $crate::SYSTEM_LOGGER.lock().fmt_error(format_args!($($arg)*))
    }
}

/// Log a fatal error.
#[macro_export]
#[macro_use]
macro_rules! fatal {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().fatal($message)
    };

    ($($arg:tt)*) => {
        $crate::SYSTEM_LOGGER.lock().fmt_fatal(format_args!($($arg)*))
    }
}
