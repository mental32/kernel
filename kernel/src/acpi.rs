use core::{mem::size_of, ptr::NonNull};

use acpi::PhysicalMapping;


pub struct AmlHandler;

impl aml::Handler for AmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        todo!()
    }

    fn read_u16(&self, address: usize) -> u16 {
        todo!()
    }

    fn read_u32(&self, address: usize) -> u32 {
        todo!()
    }

    fn read_u64(&self, address: usize) -> u64 {
        todo!()
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        todo!()
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        todo!()
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        todo!()
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        todo!()
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        todo!()
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        todo!()
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        todo!()
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        todo!()
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        todo!()
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        todo!()
    }

    fn read_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, segment: u16, bus: u8, device: u8, function: u8, offset: u16, value: u32) {
        todo!()
    }
}

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
