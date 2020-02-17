#![no_std]
#![forbid(missing_docs)]

//! A Simple PIT driver.
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
//! # The Oscillator
//!
//! The oscillator used by the PIT chip runs at (roughly) 1.193182 MHz. The
//! reason for this requires a trip back into history (to the later half of the 1970's)...
//!
//! The original PC used a single "base oscillator" to generate a frequency of
//! 14.31818 MHz because this frequency was commonly used in television
//! circuitry at the time. This base frequency was divided by 3 to give a
//! frequency of 4.77272666 MHz that was used by the CPU, and divided by 4
//! to give a frequency of 3.579545 MHz that was used by the CGA video
//! controller. By logically ANDing these signals together a frequency
//! equivalent to the base frequency divided by 12 was created. This frequency
//! is 1.1931816666 MHz (where the 6666 part is recurring). At the time it was
//! a brilliant method of reducing costs, as the 14.31818 MHz oscillator was
//! cheap due to mass production and it was cheaper to derive the other
//! frequencies from this than to have several oscillators. In modern
//! computers, where the cost of electronics is much less, and the CPU and
//! video run at much higher frequencies the PIT lives on as a reminder of
//! "the good ole' days".

use cpuio::Port;

const CHAN_0_DATA: u16 = 0x40;
const MDE_CMD_REG: u16 = 0x43;

macro_rules! pt {
    ($pt:expr) => {
        unsafe { Port::<u8>::new($pt) }
    };
}

/// The frequency (in hertz) of the (8253/8254) PIT Oscillator.
pub const PIT_OSC_FREQ: usize = 1193180;

/// A PIT struct.
pub struct ProgrammableIntervalTimer {
    freq: usize,
}

impl ProgrammableIntervalTimer {
    #[inline]
    fn io_wait(&self) {
        pt!(0x80).write(0x00);
    }

    /// Get the frequency of the PIT.
    pub fn freq(&self) -> usize {
        self.freq
    }

    /// Modify the PITs frequency to a set value.
    pub fn set_frequency(&mut self, freq: usize) {
        let divisor = PIT_OSC_FREQ / freq;

        // 0x36 is the command byte.
        pt!(MDE_CMD_REG).write(0x36);

        // Divisor has to be sent byte-wise, so split here into upper/lower bytes.
        let l = (divisor & 0xFF) as u8;
        let h = ((divisor >> 8) & 0xFF) as u8;

        let mut chan_0_data = pt!(CHAN_0_DATA);

        // Send the frequency divisor.
        chan_0_data.write(l);
        self.io_wait();
        chan_0_data.write(h);
        self.io_wait();

        self.freq = freq;
    }

    /// Estimate the end time from a base uptime after some milis.
    pub fn time_after_sleep_milis(&self, uptime: usize, delta: usize) -> usize {
        uptime + (delta * (self.freq / 1000))
    }
}

impl From<usize> for ProgrammableIntervalTimer {
    fn from(other: usize) -> Self {
        Self { freq: other }
    }
}
