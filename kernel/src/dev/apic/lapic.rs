//! Small APIC device driver.

use acpi::interrupt::{Apic as ApicInner, InterruptModel};
use acpi::Acpi;

use crate::{mm, AcpiError, KernelException, KernelResult};

use super::SPURRIOUS_INTERRUPT_VECTOR;

use x86_64::{
    structures::paging::{Page, PageTableFlags},
    VirtAddr,
};

pub struct Lapic {
    register_page: Page,
}

impl Lapic {
    pub fn new(local_apic_address: u64) -> Lapic {
        let register_page = Page::containing_address(VirtAddr::new(local_apic_address));
        Self { register_page }
    }

    pub unsafe fn init(&mut self) -> KernelResult<()> {
        let mut memory_manager = mm!()
            .try_lock()
            .expect("Unable to access the memory manager during APIC initialization.");

        if memory_manager.page_is_mapped(self.register_page) {
            return Err(KernelException::PageAlreadyMapped);
        }

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        memory_manager
            .map_to(self.register_page, flags)
            .expect("Unable to map APIC register page!")
            .flush();

        SPURRIOUS_INTERRUPT_VECTOR.write(
            self.register_page,
            SPURRIOUS_INTERRUPT_VECTOR.read(self.register_page)? | 0x1ff,
        )?;

        Ok(())
    }
}
