use x86_64::VirtAddr;

pub(crate) type ThreadIdent = u64;

#[derive(Debug, PartialEq)]
#[repr(C)]
pub(crate) struct ThreadControlBlock {
    stack: Option<VirtAddr>,
    ident: ThreadIdent,
}

impl ThreadControlBlock {
    pub fn new(ident: ThreadIdent) -> Self {
        Self { ident, stack: None }
    }
}
