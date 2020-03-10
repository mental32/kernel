use super::{PciDevice, PciEnumeration, MAX_BUS, MAX_DEVICE, MAX_FUNCTION};

use x86_64::instructions::port::Port;

#[derive(Debug)]
pub struct CamPorts {
    data: Port<u32>,
    command: Port<u32>,
}

impl CamPorts {
    pub fn inspect(&mut self, bus: u16, device: u8, function: u8) -> Option<PciDevice> {
        use crate::info;
        let ident = 0x1 << 31
            | ((bus as u32) << 16 | (device as u32) << 11 | (function as u32) << 8) as u32
            | 0x00;

        let vendor_id: u16 = self.read_lower_u16(ident, 0x00);

        // info!("{:x?}", vendor_id == 0xFFFF);

        if vendor_id == 0xFFFF {
            None
        } else {
            info!("? {:x?}", vendor_id);
            None
        }
    }

    fn read_lower_u16(&mut self, ident: u32, offset: u32) -> u16 {
        let res = self.read(ident, offset) >> (8 * (offset & 2)) & 0xffff;
        res as u16
    }

    fn read(&mut self, ident: u32, offset: u32) -> u32 {
        unsafe {
            self.command.write(ident & 0xfc);
            self.data.read()
        }
    }
}

pub fn brute_force_enumerate(pci_enumeration: &mut PciEnumeration) {
    let mut ports = CamPorts {
        data: Port::new(0xcfc),
        command: Port::new(0xcf8),
    };

    for bus in 0..MAX_BUS {
        for dev in 0..MAX_DEVICE {
            for fun in 0..MAX_FUNCTION {
                if let Some(device) = ports.inspect(bus as u16, dev as u8, fun as u8) {
                    pci_enumeration.register(device);
                }
            }
        }
    }
}
