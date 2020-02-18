use crate::RawChar;

/// A trait that allows different concrete implementations of a VGA buffer.
pub trait VGABuffer {
    /// Report the height of the buffer.
    fn height(&self) -> usize;

    /// Report the width of the buffer.
    fn width(&self) -> usize;

    /// Write a character at position x, y.
    fn write(&mut self, x: usize, y: usize, ch: RawChar) -> Result<(), (usize, usize)>;

    /// Read a character at position x, y.
    fn read(&self, x: usize, y: usize) -> Result<RawChar, (usize, usize)>;
}
