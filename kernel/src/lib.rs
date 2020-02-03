#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

mod driver;
mod gdt;
mod isr;
mod result;
mod state;

use {
    core::fmt::Write,
    multiboot2::load,
    state::KernelStateObject,
    vga::{vprintln, VGAWriter},
    x86_64::instructions::interrupts,
};

/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) -> ! {
    let boot_info = load(multiboot_addr);
    let mut state = KernelStateObject::new(boot_info);

    // Setting everything up for regular work.
    // The call to ``KernelStateObject::prepare`` will:
    //  - Initialize the GDT & IDT
    //  - Load the appropriate code and tss selectors
    //  - Initialize the global allocator and memory manager
    //  - Resize, reallocate or modify current pages and setup a heap.

    state.prepare().unwrap();

    vprintln!("{:?}", state.boot_info);

    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut writer = VGAWriter::default();
    writer.print_fill_char(' ').unwrap();
    vprintln!(writer, "{}", info);
    loop {}
}

/// OOM handler.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
