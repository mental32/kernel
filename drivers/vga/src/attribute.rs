use crate::Color;

/// A character attribute describes the foreground and background colors to use
/// When rendering the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Attribute(u8);

impl Attribute {
    /// Create a new Attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// let attr = Attribute::new(Color::White, Color::Black);
    /// ```
    pub fn new(foreground: Color, background: Color) -> Self {
        Attribute((background as u8) << 4 | (foreground as u8))
    }

    /// Set the foreground color.
    pub fn set_foreground(&mut self, foreground: Color) {
        self.0 |= foreground as u8;
    }

    /// Set the background color.
    pub fn set_background(&mut self, background: Color) {
        self.0 = (background as u8) << 4 | ((self.0 & 0x0F) as u8)
    }

    /// Helper to create a white foreground on a black background.
    pub fn default() -> Self {
        Attribute((Color::Black as u8) << 4 | (Color::White as u8))
    }

    /// Helper to copy another attribute.
    pub fn same(color: Color) -> Self {
        Self::new(color, color)
    }
}
