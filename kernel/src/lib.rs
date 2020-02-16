#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_fn)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

#[cfg(not(target_arch = "x86_64"))]
compile_error!("This kernel only supports the (AMD) x86_64 architecture.");

extern crate alloc;

mod gdt;
mod isr;
mod mm;
mod result;
mod state;
// mod vfs;

use core::fmt::Write;

use {
    lazy_static::lazy_static,
    multiboot2::load,
    spin::Mutex,
    x86_64::instructions::{hlt, interrupts},
};

use {
    mm::LockedHeap,
    state::KernelStateObject,
    vga::{vprint, DefaultBuffer, DefaultWriter},
};

pub(crate) static KERNEL_STATE_OBJECT: Mutex<KernelStateObject> =
    Mutex::new(KernelStateObject::new());

/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) -> ! {
    let boot_info = load(multiboot_addr);

    vprint!("{:?}", &boot_info);

    // Setting everything up for regular work.
    // The call to ``KernelStateObject::prepare`` will:
    //  - Initialize the GDT & IDT
    //  - Load the appropriate code and tss selectors
    //  - Initialize the global allocator and memory manager
    //  - Resize, remap or modify current [kernel] pages and setup a heap.

    {
        let mut state = KERNEL_STATE_OBJECT.lock();
        state.prepare(&boot_info).unwrap();
    }

    interrupts::enable();

    loop {
        hlt()
    }
}

/// Global allocator refrence.
#[global_allocator]
pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::new();

/// Allocation error handler.
#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
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

/// Marks a function that is used for implementing stack unwinding.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
