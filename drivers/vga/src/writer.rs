use {
    crate::{Attribute, Char, Color, FailureReason},
    core::default::Default,
    volatile::Volatile,
};

/// Buffer width constant that is used as the default width.
pub const BUFFER_WIDTH: usize = 80;

/// Buffer height constant that is used as the default height.
pub const BUFFER_HEIGHT: usize = 25;

/// VGA MMIO address constant for most AMD64 systems.
pub const MMIO_VGA_ADDR: *mut Buffer = 0xb8000 as *mut Buffer;

/// Type alias helper that treats VGA MMIO as a two dimensional array.
pub type Buffer = [[Volatile<Char>; BUFFER_WIDTH]; BUFFER_HEIGHT];

enum VGAStatus {
    Normal,
    Escape,
    CSI,
}

/// Struct responsible for writing data into the VGA address space.
pub struct VGAWriter {
    status: VGAStatus,
    cursor: (usize, usize),
    attr: Attribute,
    width: usize,
    height: usize,
    buffer: &'static mut Buffer,
}


impl Default for VGAWriter {
    fn default() -> Self {
        let buffer = unsafe { MMIO_VGA_ADDR.as_mut() }.unwrap();
        Self::new(BUFFER_WIDTH, BUFFER_HEIGHT, buffer)
    }
}

impl VGAWriter {
    /// Create a new writer that operates over a fixed sized
    /// arena of video memory. 
    pub fn new(width: usize, height: usize, buffer: &'static mut Buffer) -> Self {
        Self {
            attr: Attribute::default(),
            status: VGAStatus::Normal,
            cursor: (0, 0),
            width,
            height,
            buffer,
        }
    }

    /// Set a byte on the video memory.
    ///
    /// # Examples
    ///
    /// ```
    /// writer.set_byte(0, 0, 'A');  // Sets the top left character to "A"
    /// ```
    pub fn set_byte(&mut self, x: usize, y: usize, byte: u8) -> crate::Result<()> {
        if y >= self.buffer.len() || x >= self.buffer[y].len() {
            return Err(FailureReason::OutOfBounds((self.width, self.height)));
        }

        self.buffer[y][x].write(Char {
            data: byte,
            attr: self.attr,
        });

        Ok(())
    }

    fn scroll(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char_ = self.buffer[row][col].read();
                self.buffer[row - 1][col].write(char_);
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            data: b' ',
            attr: self.attr,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer[row][col].write(blank);
        }
    }
}
