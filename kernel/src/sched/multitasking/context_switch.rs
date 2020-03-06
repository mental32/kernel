use super::thread::ThreadIdent;

use crate::{
    sched::{KernelScheduler, SwitchReason},
    scheduler,
};

use serial::sprintln;

use alloc::boxed::Box;
use core::mem;
use core::raw::TraitObject;
use x86_64::VirtAddr;

use crate::sched::Scheduler;

pub unsafe fn context_switch_to(
    new_stack_pointer: VirtAddr,
    prev_thread_id: ThreadIdent,
    switch_reason: SwitchReason,
) {
    asm!(
        "call asm_context_switch"
        :
        : "{rdi}"(new_stack_pointer), "{rsi}"(prev_thread_id), "{rdx}"(switch_reason as u64)
        : "rax", "rbx", "rdx", "rcx", "rsi", "rdi", "rbp", "r8", "r9", "r10", "r11", "r12", "r13", "r14", "r15", "rflags", "memory"
        : "intel", "volatile"
    );
}

global_asm!(
    "
    .intel_syntax noprefix

    // asm_context_switch(stack_pointer: u64, thread_id: u64)
    asm_context_switch:
        pushfq

        mov rax, rsp
        mov rsp, rdi

        mov rdi, rax
        call add_paused_thread

        popfq
        ret
"
);

#[no_mangle]
pub extern "C" fn add_paused_thread(
    paused_stack_pointer: VirtAddr,
    paused_thread_id: ThreadIdent,
    switch_reason: SwitchReason,
) {
    scheduler!()
        .try_lock()
        .expect("Unable to lock scheduler while adding paused thread!")
        .add_paused_thread(paused_stack_pointer, paused_thread_id, switch_reason);
}

#[naked]
pub fn call_closure_entry() -> ! {
    unsafe {
        asm!("
        pop rsi
        pop rdi
        call call_closure
    " ::: "mem" : "intel", "volatile")
    };
    unreachable!();
}

// no_mangle required because of https://github.com/rust-lang/rust/issues/68136
#[no_mangle]
extern "C" fn call_closure(data: *mut (), vtable: *mut ()) -> ! {
    let trait_object = TraitObject { data, vtable };
    let f: Box<dyn FnOnce() -> !> = unsafe { mem::transmute(trait_object) };
    f()
}
