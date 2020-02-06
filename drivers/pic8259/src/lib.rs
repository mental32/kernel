#![no_std]
#![forbid(missing_docs)]

//! A 8259 PIC driver.
//!
//! This crate uses the pic8259_simple crate under the hood and attempts
//! to expand upon it in a portable manner.
//!
//! # Background
//!
//! The 8259 Programmable Interrupt Controller (PIC) is one of the most
//! important chips making up the x86 architecture. Without it, the x86
//! architecture would not be an interrupt driven architecture. The
//! function of the 8259A is to manage hardware interrupts and send them
//! to the appropriate system interrupt. This allows the system to respond to
//! devices needs without loss of time (from polling the device, for instance).
//!
//! It is important to note that APIC has replaced the 8259 PIC in more modern
//! systems, especially those with multiple cores/processors.
//!
//! ## What does the 8259 PIC do?
//!
//! The 8259 PIC controls the CPU's interrupt mechanism, by accepting several
//! interrupt requests and feeding them to the processor in order. For instance,
//! when a keyboard registers a keyhit, it sends a pulse along its interrupt
//! line (IRQ 1) to the PIC chip, which then translates the IRQ into a system
//! interrupt, and sends a message to interrupt the CPU from whatever it is
//! doing. Part of the kernel's job is to either handle these IRQs and perform
//! the necessary procedures (poll the keyboard for the scancode) or alert a
//! userspace program to the interrupt (send a message to the keyboard driver).
//!
//! Without a PIC, you would have to poll all the devices in the system to see
//! if they want to do anything (signal an event), but with a PIC, your system
//! can run along nicely until such time that a device wants to signal an event,
//! which means you don't waste time going to the devices, you let the devices
//! come to you when they are ready.

mod index;

pub use {index::*, pic8259_simple::ChainedPics};

/// Each pic interrupt must be met with an end of interrupt.
#[macro_export]
macro_rules! eoi {
    ($base:expr, $pics:expr, $irq:expr) => {
        unsafe { $pics.notify_end_of_interrupt($base + ($irq as u8)) }
    };
}
