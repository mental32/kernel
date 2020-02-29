//! Small APIC device driver.

use acpi::interrupt::{Apic, InterruptModel};
use acpi::Acpi;

use x86_64::{structures::paging::{PageTableFlags, Page}, VirtAddr};

use crate::{mm, result::KernelResult};

macro_rules! lapic_write {
    ($madt_controller_addr:expr, $reg:expr, $data:expr) => {{
        let madt: u64 = $madt_controller_addr;
        let reg: u64 = $reg;

        let target = (madt + reg) as *mut u32;

        unsafe {
            (*target) = $data;
        }
    }};
}

macro_rules! lapic_read {
    ($madt_controller_addr:expr, $reg:expr) => {{
        let target = ($madt_controller_addr + $reg) as *const u32;

        unsafe { (*target) }
    }};
}

/// Check if the APIC is supported on this system via CPUID.
#[inline]
pub fn is_apic_supported() -> bool {
    let cpuid = raw_cpuid::CpuId::new();
    *&cpuid.get_feature_info().unwrap().has_apic()
}

#[derive(Debug)]
pub struct LapicEOIPtr(*const u16);

/// Initialize the APIC
pub fn initialize<'a>(acpi: &'a Acpi) -> KernelResult<(&'a Apic, LapicEOIPtr)> {
    let apic = match &acpi.interrupt_model {
        Some(InterruptModel::Apic(apic)) => apic,
        _ => panic!("Attempted to initialize the APIC with a bad ACPI interrupt model"),
    };

    {
        let mut memory_manager = mm!()
            .try_lock()
            .expect("Unable to access the memory manager during APIC initialization.");

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        memory_manager
            .map_to(
                Page::containing_address(VirtAddr::new(apic.local_apic_address)),
                flags,
            )
            .unwrap()
            .flush();
    }

    let lapic_base = apic.local_apic_address; // TODO: Map this since its a MMIO address

    // LAPIC enable
    let word: u32 = lapic_read!(lapic_base, 0xf0);
    lapic_write!(lapic_base, 0xf0, word | 0x1ff);

    let lapic_eoi_ptr = LapicEOIPtr((lapic_base + 0xB0) as *const u16);

    Ok((apic, lapic_eoi_ptr))
}
