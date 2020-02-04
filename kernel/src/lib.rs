#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

mod gdt;
mod isr;
mod pic;
mod result;
mod state;

use {
    core::fmt::Write,
    lazy_static::lazy_static,
    multiboot2::load,
    spin::Mutex,
    state::KernelStateObject,
    vga::{vprint, DefaultBuffer, DefaultWriter},
    x86_64::{
        instructions::{hlt, interrupts},
        structures::paging::PageTable,
    },
};

lazy_static! {
    pub(crate) static ref KERNEL_STATE_OBJECT: Mutex<Option<KernelStateObject>> =
        { Mutex::new(None) };
}

extern "C" {
    static PML4: *mut PageTable;
}

/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) -> ! {
    let boot_info = load(multiboot_addr);
    let mut state = KernelStateObject::new();

    // Setting everything up for regular work.
    // The call to ``KernelStateObject::prepare`` will:
    //  - Initialize the GDT & IDT
    //  - Load the appropriate code and tss selectors
    //  - Initialize the global allocator and memory manager
    //  - Resize, reallocate or modify current pages and setup a heap.

    state.prepare(&boot_info).unwrap();

    {
        let mut handle = KERNEL_STATE_OBJECT.lock();
        *handle = Some(state);
    }

    interrupts::enable();

    serial::sprintln!("{}", "Hello, World!");

    loop {
        hlt()
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    interrupts::disable();

    let mut writer = DefaultWriter::new(DefaultBuffer::refrence());
    writer.print_fill_char(' ').unwrap();
    vprint!(writer, "{}", info);

    loop {
        hlt()
    }
}

/// OOM handler.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
