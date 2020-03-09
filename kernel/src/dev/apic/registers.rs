use x86_64::structures::paging::Page;

use crate::{AccessPermissions, KernelResult};

macro_rules! lapic_reg_write {
    ($madt_controller_addr:expr, $reg:expr, $data:expr) => {{
        let madt: u64 = $madt_controller_addr;
        let reg: u64 = $reg;

        let target = (madt + reg) as *mut u32;
        (*target) = $data;
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
    ($offset:literal, $perm:ident, $name:ident) => {
        pub const $name: ApicRegister = ApicRegister::new($offset, AccessPermissions::$perm);
    };
}

reg!(0x20, ReadWrite, LAPIC_ID);
reg!(0x30, ReadOnly, LAPIC_VERSION);
// Registers [0x40..0x70] are marked reserved.
reg!(0x80, ReadWrite, TASK_PRIORITY);
reg!(0x90, ReadOnly, ARBITRATION_PRIORITY);
reg!(0xA0, ReadOnly, PROCESSOR_PRIORITY);
reg!(0xB0, WriteOnly, EOI);
reg!(0xC0, ReadOnly, REMOTE_READ);
reg!(0xD0, ReadWrite, LOGICAL_DESTINATION);
reg!(0xE0, ReadWrite, DEST_FMT);
reg!(0xF0, ReadWrite, SPURRIOUS_INTERRUPT_VECTOR);
// Bunch of in service registers (ISRs) and trigger mode registers (TMRs)
// TODO: Investigate bits.
reg!(0x280, ReadOnly, ERROR_STATUS);
reg!(0x2F0, ReadWrite, LVT_CMCI);
reg!(0x320, ReadWrite, LVT_TIMER);
reg!(0x330, ReadWrite, LVT_THERMAL_SENSOR);
reg!(0x340, ReadWrite, LVT_PERF_MONITORING_COUNTERS);
reg!(0x350, ReadWrite, LVT_LINT0);
reg!(0x360, ReadWrite, LVT_LINT1);
reg!(0x370, ReadWrite, LVT_ERROR);
reg!(0x380, ReadWrite, INITAL_COUNTER);
reg!(0x390, ReadOnly, CURRENT_COUNT);
reg!(0x3E0, ReadWrite, DIVIDE_CONFIGURE);
