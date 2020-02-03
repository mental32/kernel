use {
    crate::{gdt::ExposedGlobalDescriptorTable, isr::InterruptHandlers},
    bit_field::BitField,
    core::mem::size_of,
    multiboot2::BootInformation,
    x86_64::{
        instructions::{
            segmentation::set_cs,
            tables::{lidt, load_tss, DescriptorTablePointer},
        },
        structures::{
            gdt::{Descriptor, DescriptorFlags, SegmentSelector},
            idt::InterruptDescriptorTable,
            tss::TaskStateSegment,
        },
        VirtAddr,
    },
};

struct Selectors {
    code_selector: Option<SegmentSelector>,
    tss_selector: Option<SegmentSelector>,
}

/// A struct that journals the kernels state.
pub(crate) struct KernelStateObject {
    pub boot_info: BootInformation,
    idt: InterruptDescriptorTable,
    tss: TaskStateSegment,
    gdt: ExposedGlobalDescriptorTable,
    selectors: Selectors,
}

impl InterruptHandlers for KernelStateObject {}

impl KernelStateObject {
    pub fn new(boot_info: BootInformation) -> Self {
        let idt = InterruptDescriptorTable::new();
        let tss = TaskStateSegment::new();
        let gdt = ExposedGlobalDescriptorTable::new();

        let selectors = Selectors {
            code_selector: None,
            tss_selector: None,
        };

        Self {
            boot_info,
            selectors,
            idt,
            tss,
            gdt,
        }
    }

    pub unsafe fn prepare(&mut self) -> crate::result::Result<()> {
        // TSS
        self.tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // // GDT
        let tss_descriptor = {
            let ptr = (&self.tss) as *const _ as u64;

            let mut low = DescriptorFlags::PRESENT.bits();
            // base
            low.set_bits(16..40, ptr.get_bits(0..24));
            low.set_bits(56..64, ptr.get_bits(24..32));
            // limit (the `-1` in needed since the bound is inclusive)
            low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
            // type (0b1001 = available 64-bit tss)
            low.set_bits(40..44, 0b1001);

            let mut high = 0;
            high.set_bits(0..32, ptr.get_bits(32..64));

            Descriptor::SystemSegment(low, high)
        };

        let code_selector = self.gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = self.gdt.add_entry(tss_descriptor);
        self.gdt.load();

        // SELECTORS
        self.selectors = Selectors {
            code_selector: Some(code_selector),
            tss_selector: Some(tss_selector),
        };

        set_cs(self.selectors.code_selector.unwrap());
        load_tss(self.selectors.tss_selector.unwrap());

        // IDT
        Self::set_isr_handlers(&mut self.idt);
        self.load_idt();

        // ALLOCATOR
        // PAGING
        Ok(())
    }

    pub unsafe fn load_idt(&mut self) {
        let ptr = DescriptorTablePointer {
            base: (&self.idt) as *const _ as u64,
            limit: (size_of::<InterruptDescriptorTable>() - 1) as u16,
        };

        lidt(&ptr);
    }
}
