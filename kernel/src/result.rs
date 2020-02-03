#[derive(Debug)]
pub(crate) enum KernelException {}

pub(crate) type Result<T> = core::result::Result<T, KernelException>;
