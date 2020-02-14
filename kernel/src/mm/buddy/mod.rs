mod allocator;
mod frame;
mod linked_list;

pub use {allocator::*, frame::*, linked_list::*};

pub fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}
