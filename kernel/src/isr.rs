use x86_64::structures::idt::InterruptDescriptorTable;

pub(crate) trait InterruptHandlers {
    fn set_isr_handlers(idt: &mut InterruptDescriptorTable);
}
