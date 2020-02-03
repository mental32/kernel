#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

mod state;

use {
    multiboot2::load,
    state::KernelStateObject,
    spin::RwLock,
    vga::{VGAWriter, vprintln},
    lazy_static::lazy_static,
};

lazy_static! {
    /// global static refrence to the kernel state.
    pub static ref KERNEL_STATE_OBJECT: Option<RwLock<KernelStateObject>> = { None };
}


/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) {
    let boot_info = load(multiboot_addr);
    let mut writer = VGAWriter::default();
    vprintln!(writer, "{:?}", boot_info);
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
pub extern fn eh_personality() {}
