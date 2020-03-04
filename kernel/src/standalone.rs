use {
    multiboot2::load,
    spin::Mutex,
    x86_64::instructions::{hlt, interrupts},
};

use serial::sprintln;

use crate::{
    mm::LockedHeap,
    sched::{KernelScheduler, Scheduler},
    state::KernelStateObject,
};

/// Global allocator refrence.
#[global_allocator]
pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::new();

/// Global scheduler refrence.
pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

/// Global kernel state refrence.
pub static KERNEL_STATE_OBJECT: Mutex<KernelStateObject> = Mutex::new(KernelStateObject::new());

/// Helper macro to refrence the kernel state object.
#[macro_use]
#[macro_export]
macro_rules! kernel {
    () => {
        $crate::KERNEL_STATE_OBJECT
    };
}

/// Helper macro to refrence the kernel scheduler.
#[macro_use]
#[macro_export]
macro_rules! scheduler {
    () => {
        $crate::SCHEDULER
    };
}

/// Kernel main start point.
#[no_mangle]
pub unsafe extern "C" fn kmain(multiboot_addr: usize) -> ! {
    let boot_info = load(multiboot_addr);

    // Initialize the system logger with a serial writer.

    {
        crate::SYSTEM_LOGGER
            .try_lock()
            .expect("Unable to initialize system logger")
            .add_producer(&serial::GLOBAL_DEFAULT_SERIAL);
    }

    // Setting everything up for regular work.
    //
    // The call to `KernelStateObject::prepare` will:
    //  - Initialize the global allocator and memory manager
    //  - Initialize the GDT & IDT
    //  - Load the appropriate code and tss selectors
    //  - Resize, remap or modify current [kernel] pages and setup a heap.

    {
        let mut state = kernel!().lock();
        state.prepare(&boot_info).unwrap();
    }

    scheduler!()
        .try_lock()
        .expect("Unable to Initialize scheduler!")
        .init();

    interrupts::enable();

    loop {
        hlt()
    }
}

/// Allocation error handler.
#[alloc_error_handler]
pub fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    interrupts::disable();

    sprintln!("{:?}", info);

    loop {
        hlt()
    }
}

/// Marks a function that is used for implementing stack unwinding.
#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}
