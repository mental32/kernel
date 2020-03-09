mod lapic;
mod registers;

pub use {lapic::Lapic, registers::*};

/// Check if the APIC is supported on this system via CPUID.
#[inline]
pub fn is_supported() -> bool {
    let cpuid = raw_cpuid::CpuId::new();
    *&cpuid.get_feature_info().unwrap().has_apic()
}

