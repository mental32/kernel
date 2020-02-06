pub mod cpu;
pub mod pics;

use x86_64::structures::idt::InterruptDescriptorTable;

use pic8259::InterruptIndex;

use {cpu::*, pics::*};

macro_rules! set_pic_handler {
    ($idt:ident, $index:expr, $func:ident) => {
        $idt[(PIC_1_OFFSET + ($index as u8)) as usize].set_handler_fn($func);
    };
}

pub fn map_default_handlers(idt: &mut InterruptDescriptorTable) {
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    set_pic_handler!(idt, InterruptIndex::Timer, timer_interrupt_handler);
    set_pic_handler!(idt, InterruptIndex::PS2Keyboard, keyboard_interrupt_handler);
}
