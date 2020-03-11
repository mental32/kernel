use super::{PciDevice, PciEnumeration, MAX_BUS, MAX_DEVICE, MAX_FUNCTION};

use crate::info;

use x86_64::instructions::port::Port;

#[derive(Debug)]
pub struct CamPorts {
    data: Port<u32>,
    command: Port<u32>,
}

use alloc::vec::Vec;

/// Struct represents a single PCI device
#[derive(Debug, Clone)]
pub struct Device {
    pub bus: u16,
    pub device: u16,
    pub function: u16,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u16,
    pub subclass_id: u16,
    pub interface_id: u8,
    pub revision: u8,
    pub interrupt: u16,
    pub port_base: Option<u32>,
}

#[derive(Debug)]
pub struct BaseAddrReg {
    addr: u32,
    size: u32,
    reg_type: DeviceType,
    prefetch: bool,
}

#[derive(Debug)]
pub enum DeviceType {
    MemoryMapping = 0,
    InputOutput = 1,
}

impl Device {
    /// Method takes a bus, device and function int and tries to get information about them, if
    /// such a device exists it returns a new instance of Self, otherwise returns None.
    pub fn from(bus: u16, device: u16, function: u16, ports: &mut CamPorts) -> Option<Self> {
        let mut device = Self {
            bus,
            device,
            function,
            ..Default::default()
        };

        device.fill_headers(ports);

        if device.vendor_id <= 0x0004 || device.vendor_id == 0xffff {
            return None;
        }

        for i in 0..6 {
            if let Some(x) = device.get_base_addr_reg(ports, i) {
                match x.reg_type {
                    DeviceType::InputOutput => {
                        device.port_base = Some(x.addr as u32);
                    }
                    _ => {}
                }
            }
        }

        Some(device)
    }

    fn fill_headers(&mut self, ports: &mut CamPorts) {
        self.vendor_id = self.read(ports, 0x00) as u16;
        self.device_id = self.read(ports, 0x02) as u16;

        self.class_id = self.read(ports, 0x0b) >> 8;
        self.subclass_id = self.read(ports, 0x0a) & 0xff;
        self.interface_id = self.read(ports, 0x09) as u8;

        self.revision = self.read(ports, 0x08) as u8;
        self.interrupt = self.read(ports, 0x3c) & 0x00ff;
    }

    fn get_base_addr_reg(&mut self, ports: &mut CamPorts, bar: u16) -> Option<BaseAddrReg> {
        let hdr_type = self.read(ports, 0x0e) & 0x7f;

        if bar >= 6 - (4 * hdr_type) {
            return None;
        }

        let bar_val = self.read32(ports, (0x10 + 4 * bar).into());

        let dev_type = if (bar_val & 0x1) == 1 {
            DeviceType::InputOutput
        } else {
            DeviceType::MemoryMapping
        };

        match dev_type {
            DeviceType::InputOutput => Some(BaseAddrReg {
                addr: (bar_val & 0xfffc) as u32,
                size: 0,
                reg_type: dev_type,
                prefetch: false,
            }),
            _ => None,
        }
    }

    pub fn set_mastering(&mut self, ports: &mut CamPorts) {
        let original_conf = self.read32(ports, 0x04);
        let next_conf = original_conf | 0x04;

        unsafe {
            ports.command.write(self.get_id(0x04));
            ports.data.write(next_conf);
        }

        info!(
            "pci: Done setting bitmastering for {:x}:{:x}",
            self.vendor_id, self.device_id
        );
    }

    pub fn set_enable_int(&mut self, ports: &mut CamPorts) {
        let original_conf = self.read32(ports, 0x04);
        let next_conf = original_conf & !(1 << 10);

        unsafe {
            ports.command.write(self.get_id(0x04));
            ports.data.write(next_conf);
        }
    }

    pub fn set_disable_int(&mut self, ports: &mut CamPorts) {
        let original_conf = self.read32(ports, 0x04);
        let next_conf = original_conf | (1 << 10);

        unsafe {
            ports.command.write(self.get_id(0x04));
            ports.data.write(next_conf);
        }
    }

    pub fn set_interrupt(&mut self, ports: &mut CamPorts, int: u32) {
        unsafe {
            ports.command.write(self.get_id(0x3c));
            ports.data.write(int);
        }

        info!(
            "pci: Done setting interrupt to {} for {:x}:{:x}",
            int, self.vendor_id, self.device_id
        );
    }

    fn read(&mut self, ports: &mut CamPorts, offset: u32) -> u16 {
        unsafe {
            ports.command.write(self.get_id(offset & 0xfc));
            (ports.data.read() >> (8 * (offset & 2)) & 0xffff) as u16
        }
    }

    fn read32(&mut self, ports: &mut CamPorts, offset: u32) -> u32 {
        unsafe {
            ports.command.write(self.get_id(offset & 0xfc));
            ports.data.read()
        }
    }

    fn get_id(&self, offset: u32) -> u32 {
        0x1 << 31
            | (self.bus as u32) << 16
            | (self.device as u32) << 11
            | (self.function as u32) << 8
            | offset
    }
}

impl Default for Device {
    fn default() -> Self {
        Self {
            bus: 0,
            device: 0,
            function: 0,
            vendor_id: 0,
            device_id: 0,
            class_id: 0,
            subclass_id: 0,
            interface_id: 0,
            revision: 0,
            interrupt: 0,
            port_base: None,
        }
    }
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
                if let Some(device) = Device::from(bus as u16, dev as u16, fun as u16, &mut ports) {
                    pci_enumeration.register(device);
                }
            }
        }
    }
}
