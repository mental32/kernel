//! Default x86 CPU exception/interrupt handlers.

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::{VirtAddr, registers::control::Cr2, structures::idt::{InterruptStackFrame, PageFaultErrorCode}, structures::{gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector}, idt::InterruptDescriptorTable, tss::TaskStateSegment}};

mod cpu_reserved;
mod index;

/// Each pic interrupt must be met with an end of interrupt.
macro_rules! eoi {
    ($base:expr, $pics:expr, $irq:expr) => {
        unsafe { $pics.notify_end_of_interrupt($base + ($irq as u8)) }
    };
}

pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

lazy_static! {
    pub(crate) static ref INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        use cpu_reserved::*;

        // Set CPU routines.
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        idt
    };

    pub static ref TSS: TaskStateSegment = {
            let mut tss = TaskStateSegment::new();
            tss.interrupt_stack_table[0] = {
                const STACK_SIZE: usize = 4096 * 5;
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

                let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
                let stack_end = stack_start + STACK_SIZE;
                stack_end
            };
            tss
        };

    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_selector,
                tss_selector,
            },
        )
    };

}
