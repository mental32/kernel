use {lazy_static::lazy_static, pic8259::ChainedPics, spin::Mutex};

/// IRQ offset for the slave pic.
pub const PIC_SLAVE_OFFSET: u8 = 32;

/// IRQ offset for the master pic.
pub const PIC_MASTER_OFFSET: u8 = PIC_SLAVE_OFFSET + 8;

lazy_static! {
    /// Static refrence to ``Mutex<ChainedPics>``.
    pub static ref PICS: Mutex<ChainedPics> =
        { Mutex::new(unsafe { ChainedPics::new(PIC_SLAVE_OFFSET, PIC_MASTER_OFFSET) }) };
}
