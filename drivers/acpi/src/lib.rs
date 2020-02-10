#![no_std]
#![forbid(missing_docs)]

//! A portable Advanced Configuration and Power Interface (ACPI) driver.
//!
//! At the time of writing the driver supports v6.3 of the ACPI spec:
//! https://uefi.org/sites/default/files/resources/ACPI_6_3_final_Jan30.pdf
//!
//! # ACPI Overview
//!
//! ## History of ACPI
//!
//! ACPI was developed through collaboration between Intel, Microsoft*,
//! Toshiba*, HP*, and Phoenix* in the mid-1990s. Before the development of
//! ACPI, operating systems (OS) primarily used BIOS
//! (Basic Input/Output System) interfaces for power management and device
//! discovery and configuration. This power management approach used the OS’s
//! ability to call the system BIOS natively for power management. The BIOS
//! was also used to discover system devices and load drivers based on probing
//! input/output (I/O) and attempting to match the correct driver to the
//! correct device (plug and play). The location of devices could also be hard
//! coded within the BIOS because the platform itself was non-enumerable.
//!
//! These solutions were problematic in three key ways. First, the behavior of
//! OS applications could be negatively affected by the BIOS-configured power
//! management settings, causing systems to go to sleep during presentations
//! or other inconvenient times. Second, the power management interface was
//! proprietary on each system. This required developers to learn how to
//! configure power management for each individual system. Finally, the default
//! settings for various devices could also conflict with each other, causing
//! devices to crash, behave erratically, or become undiscoverable.
//!
//! ACPI was developed to solve these problems and others.
//!
//! ## What is ACPI?
//!
//! ACPI can first be understood as an architecture-independent power
//! management and configuration framework that forms a subsystem within the
//! host OS. This framework establishes a hardware register set to define power
//! states (sleep, hibernate, wake, etc). The hardware register set can
//! accommodate operations on dedicated hardware and general purpose hardware.
//!
//! ACPI can first be understood as an architecture-independent power
//! management and configuration framework that forms a subsystem within the
//! host OS. This framework establishes a hardware register set to define power
//! states (sleep, hibernate, wake, etc). The hardware register set can
//! accommodate operations on dedicated hardware and general purpose hardware/.
