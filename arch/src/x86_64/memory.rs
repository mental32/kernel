use core::{intrinsics::transmute, marker::PhantomData, mem::MaybeUninit, ops::Range};

use macros::once;
use mem::{PhysFrameAlloc, PhysFrameCursor, PhysicalMemory, boot_frame::PhysFrameIter, chunks::MemoryChunks};

use multiboot2::BootInformation;
use x86_64::{
    structures::paging::{
        mapper::{MapToError, MapperFlush},
        page_table::FrameError,
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageSize, PageTable,
        PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

/// Used to generate `SIZE` sized and 4KB aligned structures.
#[repr(C, align(4096))]
pub(super) struct AlignedHole<const SIZE: usize>([u8; SIZE]);

/// 4KB of `.bss` memory (aligned) storing our paging PML4.
#[no_mangle]
#[link_section = ".bss.pml4"]
pub(super) static mut PML4_SPACE: AlignedHole<4096> = AlignedHole([0u8; 4096]);

/// 4KB of `.bss` memory (aligned) storing a paging PDPT.
#[no_mangle]
#[link_section = ".bss.pdpt"]
static mut PDPT_SPACE: AlignedHole<4096> = AlignedHole([0u8; 4096]);

/// 4KB of `.bss` memory (aligned) storing a paging PDT.
#[no_mangle]
#[link_section = ".bss.pdt"]
static mut PDT_SPACE: AlignedHole<4096> = AlignedHole([0u8; 4096]);

const STACK_SIZE: usize = 0x4000; // 16KiB

/// Yes... this is the stack, no flash photography please.
#[no_mangle]
#[link_section = ".bss.stack"]
static mut KERNEL_STACK: AlignedHole<STACK_SIZE> = AlignedHole([0u8; STACK_SIZE]);

// -- struct MemoryManager;

/// Generate ELF `MemoryChunks<{ 1 }>` from `multiboot2::BootInformation`.
///
/// This is used to map the range of the kernels ELF sections with the
/// assumption that they are all contiguous in memory, `None` is returned
/// when there is no ELF section tag or the tag contains no entries.
///
/// The multiboot2 memory map tag includes areas in memory that are
/// marked as "available" **__including__** our kernels ELF sections.
///
/// The `MemoryChunks` produced by this function will be used when
/// generating physical frames of a certain size (`0x1000` for instance)
/// allowing us to avoid overwriting the space where our ELF sections are.
pub(super) fn collect_elf_sections_into_chunks(info: &BootInformation) -> Option<(u64, u64)> {
    let mut it = info.elf_sections_tag()?.sections();

    let (start, mut end) = {
        let first = it.next()?;
        let start = first.start_address();
        let end = first.end_address();
        (start, end)
    };

    while let Some(section) = it.next() {
        assert_eq!(
            section.start_address(),
            end,
            "Non-contiguous ELF section detected! {:?} ({:?})",
            section,
            (start, end)
        );

        // Adjacent sections grow eachother.
        end = section.end_address();
    }

    Some((start, end))
}

/// Used to (de)allocate physframes and (un)map pages.
#[derive(Debug, Default)]
pub(super) struct VirtualMemoryManager {
    memory: PhysicalMemory,
    page_table: Option<OffsetPageTable<'static>>,
    physframe_cursor: PhysFrameCursor,
}

impl VirtualMemoryManager {
    pub(super) const fn new() -> Self {
        Self {
            memory: MemoryChunks::Contiguous { start: 0, end: 0 },
            page_table: None,
            physframe_cursor: PhysFrameCursor {
                chunk_idx: 0,
                section_idx: 0,
            },
        }
    }
}

impl ::mem::MemoryManager for VirtualMemoryManager {
    fn identity_map(&mut self, address: usize, flags: u64) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let virt = VirtAddr::new(address as u64);
        let phys = PhysAddr::new(address as u64);

        let page = Page::containing_address(virt);
        let frame = PhysFrame::containing_address(phys);

        let mut frame_allocator = PhysFrameAlloc {
            memory: self.memory.clone(),
            physframe_cursor: PhysFrameCursor::default(),
        };

        unsafe { self.map_to(page, frame, flags, &mut frame_allocator) }
    }
    
    fn map_to(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame,
        flags: u64,
        frame_allocator: &mut PhysFrameAlloc,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let mut table = self.page_table.as_mut().unwrap();

        let flags = unsafe { PageTableFlags::from_bits_unchecked(flags) };

        let result = unsafe { table.map_to(page, frame, flags, frame_allocator) };

        self.physframe_cursor = frame_allocator.physframe_cursor.clone();

        result
    }

    fn unmap(&mut self, page: Page<Size4KiB>) {
        todo!()
    }

    fn map(
        &mut self,
        page: Page<Size4KiB>,
        flags: u64,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let mut frame_allocator = PhysFrameAlloc {
            memory: self.memory.clone(),
            physframe_cursor: PhysFrameCursor::default(),
        };

        let mut frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate a frame...");

        self.map_to(page, frame, flags, &mut frame_allocator)
    }

    #[once]
    fn initialize(&mut self, info: &BootInformation) {
        let mut buf: PhysicalMemory = info
            .memory_map_tag()
            .expect("Memory map tag required.")
            .memory_areas()
            .collect();

        // First megabyte of memory normally contains stuff we don't want to
        // risk immedietly overwriting...

        log::trace!("\t{:?}", buf);

        let hole = buf.poke((0 as usize, 0x100000 as usize));
        log::trace!("1MIB:\t\t{:?}", hole);

        log::trace!("\t{:?}", buf);

        {
            // Also, poke out the areas in memory where our ELF sections have been
            // placed... we don't want to overwrite them (UB)

            let (start, end) =
                collect_elf_sections_into_chunks(&info).expect("No ELF sections found!");

            let hole = buf.poke((start as usize, end as usize));
            log::trace!("ELF:\t\t{:?}", hole);
        }

        log::trace!("\t{:?}", buf);

        // Update the internal buffer.

        self.memory.swap(&mut buf);

        // Reset the underlying page table make sure the PML4 table
        // being used is valid.

        let mut table = unsafe {
            OffsetPageTable::new(
                transmute::<_, &mut PageTable>(&mut PML4_SPACE),
                VirtAddr::new(0x00),
            )
        };

        self.page_table = Some(table);
    }
}
