use pic8259_simple::ChainedPics;
use spin::Mutex;

macro_rules! pt {
    ($pt:expr) => {
        unsafe { ::cpuio::Port::<u8>::new($pt) }
    };
}

/// IRQ offset for the slave pic.
pub const DEFAULT_PIC_SLAVE_OFFSET: u8 = 32;

/// IRQ offset for the master pic.
pub const DEFAULT_PIC_MASTER_OFFSET: u8 = DEFAULT_PIC_SLAVE_OFFSET + 8;

// -- Chip8259

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

    pub unsafe fn remap(&self, slave: u8, master: u8) {
        let mut handle = self.pic.lock();
        *handle = ChainedPics::new(slave, master);
    }

    pub unsafe fn mask_all(&self) {
        pt!(0xA1).write(0xFF);
        pt!(0x21).write(0xFF);
    }

    pub unsafe fn setup(&self, pic_slave_offset: u8) {
        let mut pic = self.pic.lock();
        pic.initialize();

        let mut pit = self.pit.lock();
        pit.reconfigure();
    }
}

// -- ProgrammableIntervalTimer (PIT)

const CHAN_0_DATA: u16 = 0x40;
const MDE_CMD_REG: u16 = 0x43;

/// The frequency (in hertz) of the (8253/8254) PIT Oscillator.
pub const PIT_OSC_FREQ: usize = 1193180;

/// A (P)rogrammable (I)nterval (Timer) for the 825x oscillator.
pub struct ProgrammableIntervalTimer {
    /// The frequency (in hertz) that the PIT should interrupt at.
    pub freq: usize,
}

impl ProgrammableIntervalTimer {
    #[inline]
    unsafe fn io_wait(&self) {
        pt!(0x80).write(0x00);
    }

    /// Construct a new PIT.
    pub const fn new(freq: usize) -> Self {
        Self { freq }
    }

    /// Reconfigure the PIT with our frequency.
    pub unsafe fn reconfigure(&mut self) {
        let divisor = PIT_OSC_FREQ / self.freq;

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
