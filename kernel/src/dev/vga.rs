use core::mem::size_of;

use {multiboot2::FramebufferTag, volatile::Volatile};

use vga::{RawChar, VGABuffer};

#[derive(Debug)]
pub struct VGAFramebuffer<'a> {
    tag: FramebufferTag<'a>,
}

impl<'a> VGAFramebuffer<'a> {
    pub fn new(tag: FramebufferTag<'a>) -> Self {
        Self { tag }
    }
}

impl VGABuffer for VGAFramebuffer<'_> {
    fn height(&self) -> usize {
        self.tag.height as usize
    }

    fn width(&self) -> usize {
        self.tag.width as usize
    }

    fn write(&mut self, x: usize, y: usize, ch: RawChar) -> Result<(), (usize, usize)> {
        if y >= self.height() || x >= self.width() {
            return Err((self.width(), self.height()));
        }

        unsafe {
            let ptr = (self.tag.address
                + ((size_of::<RawChar>() * x) as u64)
                + ((size_of::<RawChar>() * self.width() * y) as u64))
                as *mut Volatile<RawChar>;

            (&mut *ptr).write(ch);
        };

        Ok(())
    }

    fn read(&self, x: usize, y: usize) -> Result<RawChar, (usize, usize)> {
        if y >= self.height() || x >= self.width() {
            return Err((self.width(), self.height()));
        }

        let ch = unsafe {
            let ptr = (self.tag.address
                + ((size_of::<RawChar>() * x) as u64)
                + ((size_of::<RawChar>() * self.width() * y) as u64))
                as *mut Volatile<RawChar>;

            (&mut *ptr).read()
        };

        Ok(ch)
    }
}
