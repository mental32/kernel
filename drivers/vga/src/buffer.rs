use {super::Char, volatile::Volatile};

/// A trait that allows different concrete implementations of a VGA buffer.
pub trait VGABuffer {
    /// Report the height of the buffer.
    fn height(&self) -> usize;

    /// Report the width of the buffer.
    fn width(&self) -> usize;

    /// Write a character at position x, y.
    fn write(&mut self, x: usize, y: usize, ch: Char);

    /// Read a character at position x, y.
    fn read(&self, x: usize, y: usize) -> Char;
}

/// Buffer width constant that is used as the default width.
pub const BUFFER_WIDTH: usize = 80;

/// Buffer height constant that is used as the default height.
pub const BUFFER_HEIGHT: usize = 25;

/// Default buffer implementation that targets that hardware provided mmio.
#[repr(transparent)]
pub struct DefaultBuffer {
    data: [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// VGA MMIO address constant for most AMD64 systems.
pub const MMIO_VGA_ADDR: *mut DefaultBuffer = 0xb8000 as *mut DefaultBuffer;

impl DefaultBuffer {
    /// Get a refrence to the default buffer.
    pub fn refrence() -> &'static mut Self {
        unsafe { MMIO_VGA_ADDR.as_mut() }.unwrap()
    }
}

impl VGABuffer for DefaultBuffer {
    fn height(&self) -> usize {
        BUFFER_HEIGHT
    }

    fn width(&self) -> usize {
        BUFFER_WIDTH
    }

    fn write(&mut self, x: usize, y: usize, ch: Char) {
        self.data[y][x].write(ch);
    }

    fn read(&self, x: usize, y: usize) -> Char {
        self.data[y][x].read()
    }
}
