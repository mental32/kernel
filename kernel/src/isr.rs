//! Default x86 CPU exception and interrupt handlers.

use x86_64::structures::idt::InterruptDescriptorTable;

pub fn map_default_handlers(idt: &mut InterruptDescriptorTable) {
    use handlers::*;

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
}

mod handlers {
    use x86_64::{
        registers::control::Cr2,
        structures::idt::{InterruptStackFrame, PageFaultErrorCode},
    };

    use serial::sprintln;

    pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
        sprintln!("Breakpoint!\n{:#?}", stack_frame);
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
}
