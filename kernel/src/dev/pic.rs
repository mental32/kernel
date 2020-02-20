use {
    lazy_static::lazy_static,
    pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1},
    spin::Mutex,
    x86_64::{instructions::port::Port, structures::idt::InterruptStackFrame},
};

use {
    pic8259::{eoi, ChainedPics, InterruptIndex},
    pit825x::ProgrammableIntervalTimer,
    serial::sprintln,
};

use crate::state::KernelStateObject;

/// IRQ offset for the slave pic.
const DEFAULT_PIC_SLAVE_OFFSET: u8 = 32;

/// IRQ offset for the master pic.
const DEFAULT_PIC_MASTER_OFFSET: u8 = DEFAULT_PIC_SLAVE_OFFSET + 8;

pub static CHIP_8259: Chip8259 =
    Chip8259::new(0x1000, DEFAULT_PIC_SLAVE_OFFSET, DEFAULT_PIC_MASTER_OFFSET);

pub struct Chip8259 {
    pub pic: Mutex<ChainedPics>,
    pub pit: Mutex<ProgrammableIntervalTimer>,
}

impl Chip8259 {
    pub const fn new(pit_freq: usize, slave: u8, master: u8) -> Self {
        Self {
            pit: Mutex::new(ProgrammableIntervalTimer::new(pit_freq)),
            pic: Mutex::new(unsafe { ChainedPics::new(slave, master) }),
        }
    }

    pub unsafe fn init(&self, kernel: &mut KernelStateObject) {
        use controller_interrupt_handlers::*;

        kernel.set_idt_entry(
            DEFAULT_PIC_SLAVE_OFFSET + (InterruptIndex::Timer as u8),
            timer_interrupt_handler,
        );

        kernel.set_idt_entry(
            DEFAULT_PIC_SLAVE_OFFSET + (InterruptIndex::PS2Keyboard as u8),
            keyboard_interrupt_handler,
        );

        let mut pic = self.pic.lock();
        pic.initialize();

        let mut pit = self.pit.lock();
        pit.reconfigure();
    }
}

mod controller_interrupt_handlers {
    use super::*;

    pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
        eoi!(
            DEFAULT_PIC_SLAVE_OFFSET,
            CHIP_8259.pic.lock(),
            InterruptIndex::Timer
        );
    }

    pub extern "x86-interrupt" fn keyboard_interrupt_handler(
        _stack_frame: &mut InterruptStackFrame,
    ) {
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

        eoi!(
            DEFAULT_PIC_SLAVE_OFFSET,
            CHIP_8259.pic.lock(),
            InterruptIndex::PS2Keyboard
        );
    }
}
