use super::context_switch::call_closure_entry;

use alloc::boxed::Box;
use core::mem;
use core::raw::TraitObject;
use x86_64::VirtAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct StackBounds {
    start: VirtAddr,
    end: VirtAddr,
}

impl StackBounds {
    pub const fn new(start: VirtAddr, end: VirtAddr) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> VirtAddr {
        self.start
    }

    pub fn end(&self) -> VirtAddr {
        self.end
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct Stack {
    pointer: VirtAddr,
}

impl Stack {
    pub unsafe fn new(stack_pointer: VirtAddr) -> Self {
        Stack {
            pointer: stack_pointer,
        }
    }

    pub fn get_stack_pointer(self) -> VirtAddr {
        self.pointer
    }

    pub unsafe fn set_up_for_closure(&mut self, closure: Box<dyn FnOnce() -> !>) {
        let trait_object: TraitObject = mem::transmute(closure);
        self.push(trait_object.data);
        self.push(trait_object.vtable);
        self.set_up_for_entry_point(call_closure_entry);
    }

    pub unsafe fn set_up_for_entry_point(&mut self, entry_point: fn() -> !) {
        self.push(entry_point);
        let rflags: u64 = 0x200;
        self.push::<u64>(0x200); // rflags
    }

    unsafe fn push<T>(&mut self, value: T) {
        self.pointer -= core::mem::size_of::<T>();
        let ptr: *mut T = self.pointer.as_mut_ptr();
        ptr.write(value);
    }
}
