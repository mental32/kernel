/// Log an information message.
#[macro_export]
#[macro_use]
macro_rules! info {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().info($message)
    };
}

/// Log a warning.
#[macro_export]
#[macro_use]
macro_rules! warn {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().warn($message)
    };
}

/// Log an error.
#[macro_export]
#[macro_use]
macro_rules! error {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().error($message)
    };
}

/// Log a fatal error.
#[macro_export]
#[macro_use]
macro_rules! fatal {
    ($message:expr) => {
        $crate::SYSTEM_LOGGER.lock().fatal($message)
    };
}
