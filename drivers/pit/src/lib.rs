#![no_std]
#![forbid(missing_docs)]

//! A PIT driver.
//!
//! The Programmable Interval Timer (PIT) chip (also called an 8253/8254 chip)
//! basically consists of an oscillator, a prescaler and 3 independent frequency
//! dividers. Each frequency divider has an output, which is used to allow the
//! timer to control external circuitry (for example, IRQ 0).
//!
//! | I/O port | Usage                                                  |
//! +----------+--------------------------------------------------------|
//! | 0x40     | Channel 0 data port (read/write)                       |
//! | 0x41     | Channel 1 data port (read/write)                       |
//! | 0x42     | Channel 2 data port (read/write)                       |
//! | 0x43     | Mode/Command register (write only, a read is ignored)  |
//!

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CHAN_0_DATA: Mutex<UnsafePort<u16>> = {};
    pub static ref CHAN_1_DATA: Mutex<UnsafePort<u16>> = {};
    pub static ref CHAN_2_DATA: Mutex<UnsafePort<u16>> = {};
    pub static ref MDE_CMD_REG: Mutex<UnsafePort<u16>> = {};
}
