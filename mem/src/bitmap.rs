//! Implementation of a bitmap used to track physical page frames and whether they're used or free.

use core::convert::TryInto;
use core::sync::atomic::{AtomicPtr, Ordering};

use bit_field::BitField;

// -- BitMap

/// 
#[derive(Debug, Default)]
pub(crate) struct BitMap {
    head: AtomicPtr<u8>,
    tail: AtomicPtr<u8>,
}

impl BitMap {
        /// Create a new bitmap from a range.
        ///
        /// # Safety
        ///
        /// The caller must ensure that `head` and `tail` are valid, accessable
        /// aligned pointers and that `tail` > `head`.
        pub(crate) const unsafe fn new(head: AtomicPtr<u8>, tail: AtomicPtr<u8>) -> Self {
            Self { head, tail }
        }
    
        /// Get an address into the bitmap containing the `index` bit.
        ///
        /// Returns an `(ptr, bit_idx)` tuple, `ptr` is a `*mut u8` containing the
        /// byte with the index and `bit` describes which bit of that byte is the slot.
        ///
        /// # Safety
        ///
        /// The supplied index is not bounds checked and an out of bounds address
        /// could be returned. It is up to the caller to enusure
        /// that `index + self.tail < self.tail` is true.
        #[inline]
        pub unsafe fn address_of_unchecked(
            &self,
            index: isize,
            ordering: Ordering,
        ) -> (*mut u8, isize) {
            let (byte, bit) = ((index / 8), (index % 8));
            let ptr = self.head.load(ordering).offset(byte);
            (ptr, bit)
        }   

    /// Set the bit at `index` to `value`.
    ///
    /// This is a helper used to reduce boilerplat for `set` and `clear`.
    #[inline]
    unsafe fn modify_unchecked(&mut self, index: usize, value: bool, ordering: Ordering) -> bool {
        let index = index.try_into().unwrap();
        let (ptr, bit) = self.address_of_unchecked(index, ordering);
        let bit = bit.try_into().unwrap();

        unsafe {
            let prev = (*ptr).get_bit(bit);
            (*ptr).set_bit(bit, value);
            prev
        }
    }

    /// Enable the bit at `index` to `true`.
    #[inline]
    pub unsafe fn set_unchecked(&mut self, index: usize, ordering: Ordering) -> bool {
        self.modify_unchecked(index, true, ordering)
    }

    /// Disable the bit at `index` to `false`.
    #[inline]
    pub unsafe fn clear_uncecked(&mut self, index: usize, ordering: Ordering) -> bool {
        self.modify_unchecked(index, false, ordering)
    }
}


impl BitMap {
    /// Get an address into the bitmap containing the `index` bit.
    ///
    /// Returns an `(ptr, bit_idx)` tuple, `ptr` is a `*mut u8` containing the
    /// byte with the index and `bit` describes which bit of that byte is the slot.
    ///
    /// This function will performa a bounds check to ensure that `index` is within
    /// ptr bounds.
    #[inline]
    pub fn address_of(&self, index: isize, ordering: Ordering) -> Option<(*mut u8, isize)> {
        let start = self.head.load(ordering) as usize;
        let end = self.tail.load(ordering) as usize;

        let idx: usize = index.try_into().ok()?;

        if idx + start >= end {
            return None;
        }

        // SAFETY: We just checked that `index` is within ptr bounds.
        Some(unsafe { self.address_of_unchecked(index, ordering) })
    }

    /// Set the bit at `index` to `value`.
    ///
    /// This is a helper used to reduce boilerplat for `set` and `clear`.
    #[inline]
    fn modify(&mut self, index: usize, value: bool, ordering: Ordering) -> Option<bool> {
        let index = index.try_into().ok()?;
        let (ptr, bit) = self.address_of(index, ordering)?;
        let bit = bit.try_into().ok()?;

        unsafe {
            let prev = (*ptr).get_bit(bit);
            (*ptr).set_bit(bit, value);
            return Some(prev);
        }
    }

    /// Enable the bit at `index` to `true`.
    #[inline]
    pub fn set(&mut self, index: usize, ordering: Ordering) -> Option<bool> {
        self.modify(index, true, ordering)
    }

    /// Disable the bit at `index` to `false`.
    #[inline]
    pub fn clear(&mut self, index: usize, ordering: Ordering) -> Option<bool> {
        self.modify(index, false, ordering)
    }

    // pub fn bits(&mut self) -> impl Iterator<Item = bool> {
    //     Bits {
    //         start: self.start.load(Ordering::SeqCst) as usize,
    //         stop: self.end.load(Ordering::SeqCst) as usize,
    //         cursor: 0,
    //     }
    // }
}
