#![feature(str_split_once)]

use std::{
    env,
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use fs::write;

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_file = PathBuf::from(out_dir).join("pci_ids.rs");

    let pci_ids = read_to_string("/usr/share/hwdata/pci.ids")?;
    let mut it = pci_ids.lines().peekable();

    let mut vendor_table = String::from("");
    let mut device_table = String::from("");

    // Skip to the first section.
    while let Some(line) = it.next() {
        if line == "# Vendors, devices and subsystems. Please keep sorted." {
            break;
        }
    }

    while let Some(line) = it.next() {
        if let Some((left, rest)) = line.split_once("  ") {
            if left.chars().all(|c| c.is_ascii_hexdigit()) {
                vendor_table.push_str(&format!("    (0x{}u16, {:?}),\n", left, rest));

                while let Some(peek) = it
                    .peek()
                    .filter(|line| line.starts_with("\t") && !line.starts_with("\t\t"))
                {
                    let peek = it.next().unwrap().trim_start();
                    let (middle, name) = peek.split_once("  ").unwrap();
                    device_table.push_str(&format!(
                        "    (0x{}u16, 0x{}u16, {:?}),\n",
                        left, middle, name
                    ));
                }
            }
        }
    }

    let output = format!(
    "
pub fn vendor_name(vendor_id: u16) -> Option<&'static str> {{
    VENDOR_TABLE
        .iter()
        .filter(|(id, _)| *id == vendor_id)
        .map(|(_, name)| *name)
        .next()
}}

pub fn device_name(vendor_id: u16, device_id: u16) -> Option<&'static str> {{
    DEVICE_TABLE
        .iter()
        .filter(|(vid, did, _)| *vid == vendor_id && *did == device_id)
        .map(|(_, _, name)| *name)
        .next()
}}

pub const VENDOR_TABLE: [(u16, &str); {}] = [\n{}\n];\n
pub const DEVICE_TABLE: [(u16, u16, &str); {}] = [\n{}\n];",
        vendor_table
            .chars()
            .filter(|c| *c == '\n')
            .count(),
        vendor_table,
        device_table
            .chars()
            .filter(|c| *c == '\n')
            .count(),
        device_table
    );

    fs::write(&out_file, &output);

    fs::write("/tmp/pci_ids.rs", output)?;

    Ok(())
}
