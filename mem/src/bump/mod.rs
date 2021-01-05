use core::cell::Cell;
use core::ptr::NonNull;

use alloc::alloc::{AllocError, Allocator, Layout};

use no_panic::no_panic;

/// An area of memory that is used for bump allocation.
#[derive(Debug)]
pub struct BumpArena<const N: usize> {
    start: *const u8,
    count: Cell<usize>,
    ptr: Cell<*mut u8>,
}

impl<const N: usize> BumpArena<N> {
    /// Make a new (const sized) bump-allocating arena.
    ///
    /// # Safety
    ///
    /// * The `arena_start` ptr must be non-null, readable, writeable, and correctly aligned.
    #[inline]
    pub unsafe fn new(arena_start: NonNull<u8>) -> Self {
        let start = arena_start.as_ptr() as *const _;

        assert!(
            (start as usize).checked_add(N).is_some(),
            "The arena start address ({:p}) must not overflow with the arena size ({:?} bytes)",
            start,
            N
        );

        let ptr = Cell::new(start as *mut u8);
        let count = Cell::new(0);

        Self { start, count, ptr }
    }

    /// Check whether this arena contains some `ptr`.
    #[inline]
    #[no_panic]
    pub fn contains(&self, ptr: NonNull<u8>) -> bool {
        let ptr = ptr.as_ptr() as usize;

        let start = self.start as usize;

        // SAFETY: We've checked that this wont fail in the constructor.
        let end = unsafe { start.unchecked_add(N) };

        (start..end).contains(&ptr)
    }

    /// Bump-allocate space for `layout`.
    #[inline]
    #[no_panic]
    pub unsafe fn alloc(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: We've checked that this wont fail in the constructor.
        let end = unsafe { (self.start as usize).unchecked_add(N) };

        let size = layout.size();
        let align = layout.align();

        let aligned_ptr = {
            let ptr = self.ptr.get() as usize;

            let value = ptr.checked_add(align - 1).unwrap_or(0);
            let aligned = value & !(align - 1);
            let new_ptr = aligned.checked_add(size).ok_or(AllocError)?;

            // Does the arena have enough space remaining?
            if new_ptr > end {
                return Err(AllocError);
            }

            new_ptr as *mut u8
        };

        // Bump "up"
        self.ptr.replace(aligned_ptr as *mut u8);

        // SAFETY: It's unlikely that we'll ever reach `usize::MAX` allocations.
        let count = unsafe { self.count.get().unchecked_add(1) };

        // incr "live" alloc count
        self.count.replace(count);

        // Form the ptr-slice and return it
        let slice_ptr = core::slice::from_raw_parts_mut(aligned_ptr, size) as *mut _;

        Ok(NonNull::new_unchecked(slice_ptr))
    }

    /// Decrement the internal allocation counter and reset the bump ptr to the start if there are no more live allocations.
    #[inline]
    #[no_panic]
    pub unsafe fn dealloc(&self) {
        let amount = self.count.get().saturating_sub(1);

        if amount == 0 {
            self.ptr.replace(self.start as *mut u8);
        }

        self.count.replace(amount);
    }
}

unsafe impl<const N: usize> Allocator for BumpArena<N> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.alloc(layout) }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        if self.contains(ptr) {
            self.dealloc()
        }
    }
}
