/// A VGA cursor.
#[derive(Debug)]
pub struct VGACursor {
    /// The x co-ord of the cursor.
    pub x: usize,

    /// The y co-ord of the cursor.
    pub y: usize,

    enabled: bool,
}

impl VGACursor {
    /// A new cursor.
    pub fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            enabled: false,
        }
    }

    /// Enable the cursor by writing to some ports.
    pub fn enable(&mut self) {
        // TODO
        self.enabled = true;
    }

    /// Disable the cursor.
    pub fn disable(&mut self) {
        // TODO
        self.enabled = false;
    }

    /// Seek the cursor to pos x, y.
    pub fn seek(&mut self, width: usize, x: usize, y: usize) -> crate::Result<()> {
        let pos = y * width + x;
        Ok(())
    }

    /// Get the cursors current position.
    pub fn tell(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
