use core::fmt::Write;

use {
    lazy_static::lazy_static,
    pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1},
    x86_64::instructions::port::Port,
};

use {pic::PICS, vga::vprint};

macro_rules! eoi {
    ($pics:ident, $irq:expr) => {
        unsafe { $pics.lock().notify_end_of_interrupt($irq as u8) }
    };
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    eoi!(PICS, InterruptIndex::Timer);
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
                    vprint!("{}", character);
                }
                DecodedKey::RawKey(key) => {
                    vprint!("{:?}", key);
                }
            }
        }
    }

    eoi!(PICS, InterruptIndex::Keyboard);
}
