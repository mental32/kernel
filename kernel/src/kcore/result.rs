#[derive(Debug)]
pub enum KernelException {
    IllegalDoubleCall(&'static str),
}

pub type Result<T> = core::result::Result<T, KernelException>;
