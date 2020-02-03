use {
    core::mem::size_of,
    x86_64::{
        instructions::tables::lgdt,
        structures::{
            gdt::{Descriptor, SegmentSelector},
            DescriptorTablePointer,
        },
        PrivilegeLevel,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct ExposedGlobalDescriptorTable {
    pub table: [u64; 8],
    pub next_free: usize,
}

impl ExposedGlobalDescriptorTable {
    /// Creates an empty GDT.
    pub const fn new() -> Self {
        Self {
            table: [0; 8],
            next_free: 1,
        }
    }

    /// Adds the given segment descriptor to the GDT, returning the segment selector.
    ///
    /// Panics if the GDT has no free entries left.
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    /// Loads the GDT in the CPU using the `lgdt` instruction. This does **not** alter any of the
    /// segment registers; you **must** (re)load them yourself using [the appropriate
    /// functions](x86_64::instructions::segmentation):
    /// [load_ss](x86_64::instructions::segmentation::load_ss),
    /// [set_cs](x86_64::instructions::segmentation::set_cs).
    pub unsafe fn load(&self) {
        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: (self.table.len() * size_of::<u64>() - 1) as u16,
        };

        lgdt(&ptr);
    }

    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full");
        }
    }
}
