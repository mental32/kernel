#[derive(Debug)]
pub enum KernelException {
    IllegalDoubleCall(&'static str),
}

pub type KernelResult<T> = core::result::Result<T, KernelException>;
