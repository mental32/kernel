#[derive(Debug, Copy, Clone)]
pub struct PciDevice {
    pub vendor_id: u16,
    pub device_id: u16,

    // pub bus: u16,
    // pub device: u16,
    // pub function: u16,
    // pub class_id: u16,
    // pub subclass_id: u16,
    // pub interface_id: u8,
    // pub revision: u8,
    // pub interrupt: u16,
    // pub port_base: Option<u32>,
}

impl PciDevice {
    pub fn new(device_id: u16, vendor_id: u16) -> Self {
        Self {
            device_id,
            vendor_id,
        }
    }
}
