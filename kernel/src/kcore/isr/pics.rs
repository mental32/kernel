use {
    lazy_static::lazy_static,
    pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1},
    spin::Mutex,
    x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame},
};

use {
    pic::{eoi, ChainedPics, InterruptIndex},
    serial::sprintln,
};

/// IRQ offset for the slave pic.
pub const PIC_1_OFFSET: u8 = 32;

/// IRQ offset for the master pic.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    /// Static refrence to ``Mutex<ChainedPics>``.
    pub static ref PICS: Mutex<ChainedPics> =
        { Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }) };
}

// Interrupt Handlers.

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    eoi!(PIC_1_OFFSET, PICS.lock(), InterruptIndex::Timer);
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
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
                    sprintln!("{}", character);
                }
                DecodedKey::RawKey(key) => {
                    sprintln!("{:?}", key);
                }
            }
        }
    }

    eoi!(PIC_1_OFFSET, PICS.lock(), InterruptIndex::PS2Keyboard);
}
