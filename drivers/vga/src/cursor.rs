use super::Attribute;

pub struct VGACursor {
    x: usize,
    y: usize,
    attrs: Attribute
}

impl VGACursor {
    pub fn blank() -> Self {
        Self {x: 0, y: 0, attrs: Attribute::default()}
    }
}
