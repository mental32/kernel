use {lazy_static::lazy_static, pic8259_simple::ChainedPics, spin::Mutex};

pub(crate) const PIC_1_OFFSET: u8 = 32;
pub(crate) const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

lazy_static! {
    pub(crate) static ref PICS: Mutex<ChainedPics> =
        { Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }) };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    PS2Keyboard,
    // Unused
    Cascade,
    COM2,
    COM1,
    LPT2,
    FloppyDisk,
    LPT1,
    CMOS,
    NIC1,
    NIC2,
    NIC3,
    PS2Mouse,
    // Unused
    FPUCoprocessor,
    PrimaryATA,
    SecondaryATA,
}
