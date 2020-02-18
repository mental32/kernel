use crate::Attribute;

/// A character that VGA Text mode frambuffers will understand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct RawChar {
    /// The ASCII encoded byte that will be rendered.
    pub data: u8,

    /// The attribute to render with.
    pub attr: Attribute,
}

impl RawChar {
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
