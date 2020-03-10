mod lapic;
mod registers;

use acpi::{interrupt::Apic, Processor};

use crate::{KernelException, KernelResult};

pub use {lapic::Lapic, registers::*};

/// Check if the APIC is supported on this system via CPUID.
#[inline]
pub fn is_supported() -> bool {
    let cpuid = raw_cpuid::CpuId::new();
    *&cpuid.get_feature_info().unwrap().has_apic()
}

pub fn init_processor(processor: Processor, apic: &Apic) -> KernelResult<()> {
    Ok(())
}
