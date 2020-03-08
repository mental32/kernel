use x86_64::structures::paging::Page;

use crate::{AccessPermissions, KernelResult};

macro_rules! lapic_reg_write {
    ($madt_controller_addr:expr, $reg:expr, $data:expr) => {{
        let madt: u64 = $madt_controller_addr;
        let reg: u64 = $reg;

        let target = (madt + reg) as *mut u32;

        unsafe {
            (*target) = $data;
        }
    }};
}

macro_rules! lapic_reg_read {
    ($madt_controller_addr:expr, $reg:expr) => {{
        let target = ($madt_controller_addr + $reg) as *const u32;

        (*target)
    }};
}

#[derive(Debug, Copy, Clone)]
pub struct ApicRegister {
    offset: u64,
    perms: AccessPermissions,
}

impl ApicRegister {
    pub const fn new(offset: u64, perms: AccessPermissions) -> Self {
        Self { offset, perms }
    }

    /// Attempt to read into the register.
    pub unsafe fn read(&self, lapic_address: Page) -> KernelResult<u32> {
        Ok(lapic_reg_read!(
            lapic_address.start_address().as_u64(),
            self.offset
        ))
    }

    /// Attempt to write into the register.
    pub unsafe fn write(&self, lapic_address: Page, data: u32) -> KernelResult<()> {
        Ok(lapic_reg_write!(
            lapic_address.start_address().as_u64(),
            self.offset,
            data
        ))
    }
}

// These offsets were source from the IA programmers manual
// Volume 4, section 10, table 10-1

macro_rules! reg {
    ($name:ident, $offset:literal, $perm:ident) => {
        pub const $name: ApicRegister = ApicRegister::new($offset, AccessPermissions::$perm);
    };
}

reg!(LAPIC_ID, 0x20, ReadWrite);
reg!(LAPIC_VERSION, 0x30, ReadOnly);
// Registers [0x40..0x70] are marked reserved.
reg!(TASK_PRIORITY, 0x80, ReadWrite);
reg!(ARBITRATION_PRIORITY, 0x90, ReadOnly);
reg!(PROCESSOR_PRIORITY, 0xA0, ReadOnly);
reg!(EOI, 0xB0, WriteOnly);
reg!(REMOTE_READ, 0xC0, ReadOnly);
reg!(LOGICAL_DESTINATION, 0xD0, ReadWrite);
reg!(DEST_FMT, 0xE0, ReadWrite);
reg!(SPURRIOUS_INTERRUPT_VECTOR, 0xF0, ReadWrite);
// Bunch of in service registers (ISRs) and trigger mode registers (TMRs)
// TODO: Investigate bits.
reg!(ERROR_STATUS, 0x280, ReadOnly);
reg!(LVT_CMCI, 0x2F0, ReadWrite);
reg!(LVT_TIMER, 0x320, ReadWrite);
reg!(LVT_THERMAL_SENSOR, 0x330, ReadWrite);
reg!(LVT_PERF_MONITORING_COUNTERS, 0x340, ReadWrite);
reg!(LVT_LINT0, 0x350, ReadWrite);
reg!(LVT_LINT1, 0x360, ReadWrite);
reg!(LVT_ERROR, 0x370, ReadWrite);
reg!(INITAL_COUNTER, 0x380, ReadWrite);
reg!(CURRENT_COUNT, 0x390, ReadOnly);
reg!(DIVIDE_CONFIGURE, 0x3E0, ReadWrite);
