#![no_std]
#![forbid(missing_docs)]
#![feature(lang_items)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

//! A muggle blood kernel written in Rust, C and Haskell, with an embedded
//! WASM runtime.

mod gdt;
mod isr;
mod kcore;
mod mm;
mod sched;
mod vfs;

use core::fmt::Write;

use {
    lazy_static::lazy_static,
    multiboot2::load,
    spin::Mutex,
    x86_64::{
        instructions::{hlt, interrupts},
        structures::paging::PageTable,
    },
};

use {
    kcore::state::KernelStateObject,
    sched::{KernelScheduler, RoundRobin},
    vga::{vprint, DefaultBuffer, DefaultWriter},
};

lazy_static! {
    pub(crate) static ref KERNEL_STATE_OBJECT: Mutex<KernelStateObject> =
        { Mutex::new(KernelStateObject::new()) };
}

extern crate alloc;

/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) -> ! {
    let boot_info = load(multiboot_addr);

    // Setting everything up for regular work.
    // The call to ``KernelStateObject::prepare`` will:
    //  - Initialize the GDT & IDT
    //  - Load the appropriate code and tss selectors
    //  - Initialize the global allocator and memory manager
    //  - Resize, reallocate or modify current pages and setup a heap.

    {
        let mut state = (*KERNEL_STATE_OBJECT).lock();
        state.prepare(&boot_info).unwrap();
    }

    interrupts::enable();

    let mut executor = RoundRobin::new(&(*KERNEL_STATE_OBJECT));

    executor.spawn(vfs::launch).unwrap();
    executor.run_forever().unwrap();

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
