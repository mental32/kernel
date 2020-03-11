//! PCI(E) device driver.
//!
//! PCI E(xpress) is a high performance, general purpose I/O interconnect
//! defined for a wide variety of future computing and communication platforms.
//!
//! Key PCI attibutes, such as its usage model, load-store architecture, and
//! software interfaces, are maintained, whereas its parallel bus
//! implementation is replaced by a highly scalable, fully serial interface.

pub mod cam;
mod device;
mod error;
// pub mod ecam;

use alloc::{vec, vec::Vec};

pub use device::PciDevice;

#[derive(Debug)]
pub struct PciEnumeration {
    pub devices: Vec<cam::Device>,
}

impl PciEnumeration {
    pub fn new() -> Self {
        Self { devices: vec![] }
    }

    pub fn register(&mut self, device: cam::Device) {
        self.devices.push(device);
    }
}

pub const MAX_FUNCTION: usize = 8;
pub const MAX_DEVICE: usize = 32;
pub const MAX_BUS: usize = 256;
