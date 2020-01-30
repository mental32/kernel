use super::Attribute;

/// A character that VGA MMIO will understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct Char {
    /// The ASCII encoded byte that will be rendered.
    pub data: u8,

    /// The attribute to render with.
    pub attr: Attribute,
}

impl Char {
    /// Create a new character to render.
    ///
    /// # Examples
    /// ```
    /// let char_ = Char::new('A', Attribute::default());
    /// ```
    pub fn new(data: u8, attr: Attribute) -> Self {
        Self { data, attr }
    }
}
