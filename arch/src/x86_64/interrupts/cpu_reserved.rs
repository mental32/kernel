use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
    structures::{idt::InterruptDescriptorTable, tss::TaskStateSegment},
};

// CPU reserved routines.

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    log::error!("Breakpoint!\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("Double fault!\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn page_fault_handler(
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
