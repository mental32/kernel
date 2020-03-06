//! Kernel result type and error enums.

/// A kernel exception blanket enum.
#[derive(Debug)]
pub enum KernelException {
    /// Used when an initializer function was called more than once.
    AlreadyInitialized(&'static str),

    /// Used when an `AcpiError` as occured.
    Acpi(AcpiError),

    /// Used as an exception to check if a page is already mapped.
    PageAlreadyMapped,
}

/// A ACPI error enum.
#[derive(Debug)]
pub enum AcpiError {
    /// The APIC is not supported.
    ApicNotSupported,
}

/// The kernels result type alias.
pub type KernelResult<T> = core::result::Result<T, KernelException>;
