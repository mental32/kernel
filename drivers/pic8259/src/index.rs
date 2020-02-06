/// InterruptIndex enum that is used for mapping out pic interrupt vectors.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    /// Programmable Interrupt Timer (PIT) Interrupt vector,
    Timer = 0,

    /// Keyboard Interrupt vector.
    PS2Keyboard,

    /// Cascade (used internally by the two PICs. never raised).
    Cascade,

    /// COM2 (if enabled.)
    COM2,

    /// COM1 (if enabled.)
    COM1,

    /// LPT2 (if enabled.)
    LPT2,

    /// Floppy Disk.
    FloppyDisk,

    /// LPT1 / Unreliable "spurious" interrupt (usually).
    LPT1,

    /// CMOS real-time clock (if enabled).
    CMOS,

    /// Free for peripherals / legacy SCSI / NIC
    NIC1,

    /// Free for peripherals / SCSI / NIC
    NIC2,

    /// Free for peripherals / SCSI / NIC
    NIC3,

    /// PS2 Mouse.
    PS2Mouse,

    /// FPU / Coprocessor / Inter-processor.
    FPUCoprocessor,

    /// Primary ATA Hard Disk.
    PrimaryATA,

    /// Secondary ATA Hard Disk.
    SecondaryATA,
}
