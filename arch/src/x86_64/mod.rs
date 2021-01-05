#![cfg(feature = "x86_64")]

mod interrupts;
mod serial_logger;

pub type AllocatorT = usize;

mod memory;

pub mod prelude {
    use super::*;

    use macros::once;
    use mem::{boot_frame::PhysFrameIter, chunks::MemoryChunks};

    use bit_field::BitField;
    use log::LevelFilter;
    use multiboot2::BootInformation;

    use x86_64::{
        instructions::{
            segmentation::set_cs,
            tables::{lgdt, lidt, load_tss},
        },
        structures::{
            gdt::{Descriptor, DescriptorFlags, GlobalDescriptorTable},
            idt::InterruptDescriptorTable,
            paging::{OffsetPageTable, PageTable, PageTableIndex, RecursivePageTable, Size4KiB},
            tss::TaskStateSegment,
            DescriptorTablePointer,
        },
        VirtAddr,
    };

    /// Setup a logger and register it with `log::set_logger`.
    pub fn install_logger(level: LevelFilter) {
        let logger = serial_logger::SerialLogger::global_ref()
            .expect("Failed to get a reference to the serial logger.");

        log::set_logger(logger).expect("Failed to set logger.");
        log::set_max_level(level);
    }

    /// Panic handler stub for x86-64.
    pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
        x86_64::instructions::interrupts::disable();

        log::error!("{:#?}\n", info);

        use vga::colors::Color16;
        use vga::writers::{Graphics640x480x16, GraphicsWriter};
        let mode = Graphics640x480x16::new();
        mode.set_mode();
        mode.clear_screen(Color16::Red);

        loop {
            x86_64::instructions::hlt()
        }
    }

    pub unsafe fn memory_manager_ref() -> &'static mut impl mem::MemoryManager {
        static mut MEMORY_MANAGER: self::memory::VirtualMemoryManager = self::memory::VirtualMemoryManager::new();
        &mut MEMORY_MANAGER
    }

    /// Boot routine for x86_64 bit systems.
    #[once]
    pub fn boot(info: BootInformation) {
        log::debug!("Running boot procedure...");

        unsafe {
            use mem::MemoryManager;
            memory_manager_ref().initialize(&info);
        }

        // IDT
        let ptr = DescriptorTablePointer {
            base: VirtAddr::new(&interrupts::INTERRUPT_DESCRIPTOR_TABLE as *const _ as u64),
            limit: (core::mem::size_of::<InterruptDescriptorTable>() - 1) as u16,
        };

        unsafe { lidt(&ptr) }

        // GDT & CS/TSS selectors.
        let (gdt, selectors) = &*interrupts::GDT;

        gdt.load();

        unsafe {
            set_cs(selectors.code_selector);
            load_tss(selectors.tss_selector);
        }

        log::debug!("Boot procedure completed!");
    }
}
