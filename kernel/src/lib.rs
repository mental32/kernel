//! A small kernel written in Rust.

#![no_std]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(type_ascription)]
#![feature(llvm_asm)]
#![feature(maybe_uninit_extra)]

use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;

use core::{mem::MaybeUninit, panic};

extern crate alloc;

mod acpi;
mod heap;
mod pci;

#[macros::entry]
unsafe fn kmain() {
    // -- ACPI

    let tables = ::acpi::AcpiTables::search_for_rsdp_bios(self::acpi::AcpiPassthrough)
        .expect("Missing ACPI RSDP...");

    for table in tables.ssdts.iter() {
        log::info!("(ACPI) Parsing AML table {:?}", table);

        let slice = core::slice::from_raw_parts(table.address as *mut _, table.length as usize);

        ::aml::AmlContext::new(
            Box::new(self::acpi::aml::AmlHandler),
            true,
            ::aml::DebugVerbosity::All,
        )
        .parse_table(slice)
        .expect("Failed to parse AML table.");
    }

    #[cfg(false)]  // TODO(mental): Investigate why parsing the DSDT freezes everything...
    if let Some(dsdt) = &tables.dsdt {
        log::info!("(ACPI) Parsing DSDT {:?}", dsdt);

        let slice = core::slice::from_raw_parts(dsdt.address as *mut _, dsdt.length as usize);

        ::aml::AmlContext::new(
            Box::new(self::acpi::aml::AmlHandler),
            tables.revision == 0,
            ::aml::DebugVerbosity::All,
        )
        .parse_table(slice)
        .expect("Failed to parse DSDT");
    }

    let info = tables
        .platform_info()
        .expect("Failed to construct PlatformInfo (FADT/MADT missing?)");

    let pinfo = info
        .processor_info
        .expect("Missing processor information...");

    log::info!("(ACPI) Boot processor is: {:#?}", pinfo.boot_processor);

    for ap in pinfo.application_processors {
        log::info!("    -> {:?}", ap);
    }

    // -- PCI

    log::info!("(PCI Local Bus) Starting enumeration...");

    let ports = pci::PciPorts::new();

    for device in pci::enumerate(&ports) {
        let (vendor_id, device_id) = device.id(&ports);

        let vendor_name = match pci::vendor_name(vendor_id) {
            Some(name) => format!("{:?} ({:#x})", name, vendor_id),
            None => format!("{:#x}", vendor_id),
        };

        let device_name = match pci::device_name(vendor_id, device_id) {
            Some(name) => format!("{:?} ({:#x})", name, device_id),
            None => format!("{:#x}", device_id),
        };

        log::info!(
            "\tVendor: {}\n\tDevice: {}\n\tSupported: {:#010b}\n\tBars: {:#?}",
            vendor_name,
            device_name,
            device.supported_fns(&ports),
            device.bars(&ports).iter().filter(|bar| **bar != 0).count()
        );
    }

    log::info!("(PCI Local Bus) Completed enumeration!");
}
