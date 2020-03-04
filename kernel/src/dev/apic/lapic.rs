//! Small APIC device driver.

use acpi::interrupt::{Apic as ApicInner, InterruptModel};
use acpi::Acpi;

use crate::{mm, AcpiError, KernelException, KernelResult};

use super::SPURRIOUS_INTERRUPT_VECTOR;

use x86_64::{
    structures::paging::{Page, PageTableFlags},
    VirtAddr,
};

pub struct Lapic<'a> {
    pub inner: &'a ApicInner,
}

impl<'a> Lapic<'a> {
    /// Check if the APIC is supported on this system via CPUID.
    #[inline]
    pub fn is_supported() -> bool {
        let cpuid = raw_cpuid::CpuId::new();
        *&cpuid.get_feature_info().unwrap().has_apic()
    }

    pub fn new(acpi: &'a Acpi) -> KernelResult<Lapic> {
        let apic = match &acpi.interrupt_model {
            Some(InterruptModel::Apic(apic)) => apic,
            _ => return Err(KernelException::Acpi(AcpiError::ApicNotSupported)),
        };

        Ok(Self { inner: apic })
    }

    pub fn init(&mut self) -> KernelResult<()> {
        let mut memory_manager = mm!()
            .try_lock()
            .expect("Unable to access the memory manager during APIC initialization.");

        let lapic_base = self.inner.local_apic_address;
        let apic_register_page = Page::containing_address(VirtAddr::new(lapic_base));

        if memory_manager.page_is_mapped(apic_register_page) {
            return Err(KernelException::PageAlreadyMapped);
        }

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        memory_manager
            .map_to(apic_register_page, flags)
            .expect("Unable to map APIC register page!")
            .flush();

        SPURRIOUS_INTERRUPT_VECTOR.write(
            apic_register_page,
            SPURRIOUS_INTERRUPT_VECTOR.read(apic_register_page)? | 0x1ff,
        )?;

        Ok(())
    }
}
