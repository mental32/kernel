//! PCI(E) device driver.
//!
//! PCI E(xpress) is a high performance, general purpose I/O interconnect
//! defined for a wide variety of future computing and communication platforms.
//!
//! Key PCI attibutes, such as its usage model, load-store architecture, and
//! software interfaces, are maintained, whereas its parallel bus
//! implementation is replaced by a highly scalable, fully serial interface.

mod device;
mod error;

pub use device::PciDevice;

use core::convert::TryInto;

use alloc::{vec, vec::Vec};

use acpi::PciConfigRegions;
use x86_64::{
    structures::paging::{Page, PageTableFlags},
    VirtAddr,
};

use crate::mm;

const MAX_FUNCTION: usize = 8;
const MAX_DEVICE: usize = 32;
const MAX_BUS: usize = 256;

#[derive(Debug)]
pub struct PciDeviceIter<'a> {
    bus: usize,
    dev: usize,
    fun: usize,
    regions: &'a PciConfigRegions,
}

impl PciDeviceIter<'_> {
    fn check_device(
        &self,
        segment: usize,
        bus: usize,
        dev: usize,
        func: usize,
    ) -> Option<PciDevice> {
        let physical_address = self.regions.physical_address(
            segment.try_into().unwrap(),
            bus.try_into().unwrap(),
            dev.try_into().unwrap(),
            func.try_into().unwrap(),
        )?;

        let mut memory_manager = mm!().lock();

        let device_cas_page = Page::containing_address(VirtAddr::new(physical_address));

        if !memory_manager.page_is_mapped(device_cas_page) {
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            memory_manager
                .map_to(device_cas_page, flags)
                .expect("Unable to map PCI-E device configuration access space page!")
                .flush();
        }

        let cas_ptr = (physical_address & 0xfffc) as *mut u32;

        use crate::info;

        let id_reg: u32 = unsafe { *cas_ptr };

        info!(">> {:x?} @ {:?}", id_reg, device_cas_page);

        memory_manager
            .unmap(device_cas_page)
            .expect("Unable to unmap PCI-E device configuration access space!")
            .flush();

        return None;

        if id_reg == 0xffff {
            None
        } else {
            let device_id: u16 = (id_reg >> 16).try_into().unwrap();
            let vendor_id: u16 = (id_reg & 0xFFFF).try_into().unwrap();
            Some(PciDevice::new(device_id, vendor_id))
        }
    }
}

impl<'a> Iterator for PciDeviceIter<'a> {
    type Item = PciDevice;

    fn next(&mut self) -> Option<Self::Item> {
        for bus in (0..MAX_BUS).skip(self.bus) {
            for dev in (0..MAX_DEVICE).skip(self.dev) {
                for fun in (0..MAX_FUNCTION).skip(self.fun) {
                    if let Some(device) = self.check_device(0, bus, dev, fun) {
                        self.dev = dev;
                        self.bus = bus;
                        self.fun = fun;

                        return Some(device);
                    }
                }
                self.fun = 0
            }
            self.dev = 0
        }

        None
    }
}

pub fn brute_force_enumerate<'a>(regions: &'a PciConfigRegions) -> PciDeviceIter<'a> {
    PciDeviceIter {
        bus: 0,
        dev: 0,
        fun: 0,
        regions,
    }
}

#[derive(Debug)]
pub struct PciEnumeration {
    devices: Vec<PciDevice>,
}

impl PciEnumeration {
    pub fn new() -> Self {
        Self { devices: vec![] }
    }

    pub fn register(&mut self, device: PciDevice) {
        // self.devices.push(device);
    }
}
