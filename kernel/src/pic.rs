use {
    crate::isr::InterruptHandlers,
    core::fmt::Write,
    lazy_static::lazy_static,
    pic8259_simple::ChainedPics,
    spin::Mutex,
    vga::vprint,
    x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

pub(crate) const PIC_1_OFFSET: u8 = 32;
pub(crate) const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    pub(crate) static ref PICS: Mutex<ChainedPics> =
        { Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }) };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptHandlers for ChainedPics {
    fn set_isr_handlers(idt: &mut InterruptDescriptorTable) {
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut handle = PICS.lock();

    unsafe {
        handle.notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Uk105Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = (*KEYBOARD).lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    vprint!("{}", character);
                }
                DecodedKey::RawKey(key) => {
                    vprint!("{:?}", key);
                }
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}
