use {
    crate::{gdt::ExposedGlobalDescriptorTable, isr::InterruptHandlers, pic::PICS},
    bit_field::BitField,
    core::mem::size_of,
    multiboot2::BootInformation,
    pic8259_simple::ChainedPics,
    spin::Mutex,
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
    pic: Option<&'static Mutex<ChainedPics>>,
    gdt: ExposedGlobalDescriptorTable,
    idt: InterruptDescriptorTable,
    tss: TaskStateSegment,
    selectors: Selectors,
}

impl KernelStateObject {
    pub fn new() -> Self {
        let idt = InterruptDescriptorTable::new();
        let tss = TaskStateSegment::new();
        let gdt = ExposedGlobalDescriptorTable::new();
        let pic = None;

        let selectors = Selectors {
            code_selector: None,
            tss_selector: None,
        };

        Self {
            selectors,
            idt,
            tss,
            gdt,
            pic,
        }
    }

    pub unsafe fn prepare(&mut self, _boot_info: &BootInformation) -> crate::result::Result<()> {
        // TSS
        self.tss.interrupt_stack_table[0] = {
            const STACK_SIZE: usize = 4096;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };

        // GDT
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
        ChainedPics::set_isr_handlers(&mut self.idt);
        self.load_idt();

        self.pic = Some(&(*PICS));

        {
            let mut handle = self.pic.unwrap().lock();
            handle.initialize();
        }

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
