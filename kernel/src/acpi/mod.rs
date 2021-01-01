use core::{mem::size_of, ptr::NonNull};

use acpi::PhysicalMapping;

pub mod aml;

#[derive(Copy, Clone, Debug)]
pub(crate) struct AcpiPassthrough;

impl acpi::AcpiHandler for AcpiPassthrough {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        log::trace!(
            "(ACPI) Mapping physical_address {:?} ({:?} size)",
            physical_address,
            size
        );

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(physical_address as *mut _).unwrap(),
            region_length: size_of::<T>(),
            mapped_length: (1024 * 1024) * 2,
            handler: Self,
        }
    }

    fn unmap_physical_region<T>(&self, region: &acpi::PhysicalMapping<Self, T>) {
        log::trace!("(ACPI) Unapping region {:#x}", region.physical_start);
    }
}
