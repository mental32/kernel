use {
    crate::{Attribute, Char, Color, DefaultBuffer, VGABuffer, VGACursor},
    core::default::Default,
};

#[derive(Debug, PartialEq)]
enum VGAStatus {
    Normal,
    Escape,
    CSI,
}

/// Struct responsible for writing data into the VGA address space.
pub struct VGAWriter<'a> {
    /// The VGA cursor that gets used.
    pub cursor: VGACursor,

    attr: Attribute,
    buffer: &'a mut dyn VGABuffer,
    csi_param: Option<usize>,
    status: VGAStatus,
}

impl Default for VGAWriter<'static> {
    fn default() -> Self {
        Self::new(DefaultBuffer::refrence())
    }
}

impl VGAWriter<'static> {
    /// Create a new writer that operates over a fixed sized
    /// arena of video memory.
    pub fn new(buffer: &'static mut (dyn VGABuffer + 'static)) -> Self {
        Self {
            attr: Attribute::default(),
            cursor: VGACursor::new(buffer.width(), buffer.height()),
            status: VGAStatus::Normal,
            csi_param: None,
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
    fn set_byte(&mut self, x: usize, y: usize, byte: u8) {
        self.buffer.write(
            x,
            y,
            Char {
                data: byte,
                attr: self.attr,
            },
        );
    }

    fn scroll(&mut self) {
        for row in 1..self.buffer.height() {
            for col in 0..self.buffer.width() {
                let char_ = self.buffer.read(row, col);
                self.buffer.write(col, row - 1, char_);
            }
        }
    }

    fn clear_row(&mut self, row: usize) {
        let blank = Char {
            data: b' ',
            attr: self.attr,
        };

        for col in 0..self.buffer.width() {
            self.buffer.write(col, row, blank);
        }
    }
}
