#[derive(Debug, Copy, Clone)]
pub struct PciDevice {
    vendor_id: u16,
    device_id: u16,
}

impl PciDevice {
    pub fn new(device_id: u16, vendor_id: u16) -> Self {
        Self {
            device_id,
            vendor_id,
        }
    }
}
