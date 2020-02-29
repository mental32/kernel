use core::sync::atomic::{AtomicPtr, Ordering};
use core::fmt::Debug;

use spin::Mutex;

use x86_64::{
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        mapper::{MapToError, Mapper, MapperFlush},
        FrameAllocator, FrameDeallocator, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, UnusedPhysFrame,
    },
    PhysAddr, VirtAddr,
};

#[derive(Debug)]
pub struct MemoryManager<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> {
    pub pml4_addr: AtomicPtr<PageTable>,
    mapper: Option<OffsetPageTable<'static>>,
    falloc: Option<F>,
}

impl<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> MemoryManager<F> {
    pub const fn new(pml4_addr: AtomicPtr<PageTable>) -> Self {
        Self {
            pml4_addr,
            mapper: None,
            falloc: None,
        }
    }

    #[inline]
    pub fn is_ready(&self) -> bool {
        self.mapper.is_some()
    }

    pub unsafe fn initialize(&mut self, virt_offset: VirtAddr, falloc: F) {
        self.falloc = Some(falloc);
        self.mapper = Some(OffsetPageTable::new(&mut *self.pml4_addr.load(Ordering::SeqCst), virt_offset));
    }

    pub fn map_to(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>> {
        let mut falloc = self.falloc.as_mut().unwrap();
        let mut frame = falloc.allocate_frame().unwrap();

        self.mapper
            .as_mut()
            .unwrap()
            .map_to(page, frame, flags, falloc)
    }

    pub unsafe fn reload_paging_table(&self) {
        let phys_addr = PhysAddr::new(self.pml4_addr.load(Ordering::SeqCst) as u64);
        Cr3::write(PhysFrame::containing_address(phys_addr), Cr3Flags::empty());
    }
}

unsafe impl<F: FrameAllocator<Size4KiB> + FrameDeallocator<Size4KiB> + Debug> FrameAllocator<Size4KiB>
    for MemoryManager<F>
{
    fn allocate_frame(&mut self) -> Option<UnusedPhysFrame<Size4KiB>> {
        self.falloc.as_mut().unwrap().allocate_frame()
    }
}
