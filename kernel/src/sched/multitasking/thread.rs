use alloc::boxed::Box;

use x86_64::{
    structures::paging::{mapper, Size4KiB},
    VirtAddr,
};

use crate::mm;

use super::{Stack, StackBounds};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadIdent(u64);

impl ThreadIdent {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    fn new() -> Self {
        use core::sync::atomic::{AtomicU64, Ordering};
        static NEXT_THREAD_ID: AtomicU64 = AtomicU64::new(1);
        ThreadIdent(NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug)]
pub struct Thread {
    id: ThreadIdent,
    stack_pointer: Option<VirtAddr>,
    stack_bounds: Option<StackBounds>,
}

impl Thread {
    pub fn create(
        entry_point: fn() -> !,
        stack_size: u64,
    ) -> Result<Self, mapper::MapToError<Size4KiB>> {
        let stack_bounds = {
            let mut memory_manager = mm!().lock();
            memory_manager.allocate_thread_stack(stack_size)
        }?;

        let mut stack = unsafe { Stack::new(stack_bounds.end()) };
        unsafe { stack.set_up_for_entry_point(entry_point) };

        Ok(Self::new(stack.get_stack_pointer(), stack_bounds))
    }

    pub fn create_from_closure<F>(
        closure: F,
        stack_size: u64,
    ) -> Result<Self, mapper::MapToError<Size4KiB>>
    where
        F: FnOnce() -> ! + 'static + Send + Sync,
    {
        let stack_bounds = {
            let mut memory_manager = mm!().lock();
            memory_manager.allocate_thread_stack(stack_size)
        }?;

        let mut stack = unsafe { Stack::new(stack_bounds.end()) };
        unsafe { stack.set_up_for_closure(Box::new(closure)) };
        Ok(Self::new(stack.get_stack_pointer(), stack_bounds))
    }

    pub fn new(stack_pointer: VirtAddr, stack_bounds: StackBounds) -> Self {
        Thread {
            id: ThreadIdent::new(),
            stack_pointer: Some(stack_pointer),
            stack_bounds: Some(stack_bounds),
        }
    }

    pub fn create_root_thread() -> Self {
        Thread {
            id: ThreadIdent(0),
            stack_pointer: None,
            stack_bounds: None,
        }
    }

    pub fn id(&self) -> ThreadIdent {
        self.id
    }

    pub fn stack_pointer(&mut self) -> &mut Option<VirtAddr> {
        &mut self.stack_pointer
    }
}
