use {
    crate::state::KernelStateObject,
    core::fmt::Write,
    vga::{vprint, GLOBAL_WRITER},
    x86_64::{
        registers::control::Cr2,
        structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
    },
};

pub(crate) trait InterruptHandlers {
    fn set_isr_handlers(idt: &mut InterruptDescriptorTable);
}

impl InterruptHandlers for KernelStateObject {
    fn set_isr_handlers(idt: &mut InterruptDescriptorTable) {
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
    }
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    let mut writer = GLOBAL_WRITER.lock();
    writer.print_fill_char(' ').unwrap();
    vprint!("Breakpoint!\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double fault!\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "Page fault!\nAccessed Address: {:?}\nError Code: {:?}, {:#?}",
        Cr2::read(),
        error_code,
        stack_frame
    );
}
