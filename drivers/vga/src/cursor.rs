use cpuio::Port;

macro_rules! pt {
    ($pt:expr) => {
        unsafe { Port::<u8>::new($pt) }
    };
}

/// A VGA cursor.
#[derive(Debug)]
pub struct VGACursor {
    /// The x co-ordinate of the cursor.
    pub x: usize,

    /// The y co-ordinate of the cursor.
    pub y: usize,

    width: usize,
    height: usize,
    enabled: bool,
}

impl VGACursor {
    /// A new cursor.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height,
            enabled: false,
        }
    }

    /// Enable the cursor by writing to some ports.
    pub fn enable(&mut self, start: u8, end: u8) {
        // pt!(0x3DA).write(0x0A);

        // let mut res = pt!(0x3D5).read();

        // pt!(0x3D5).write((res & 0xC0) | start);

        // pt!(0x3DA).write(0x0B);

        // res = pt!(0x3D5).read();
        // pt!(0x3D5).write((res & 0xE0) | end);

        // self.enabled = true;
    }

    /// Disable the cursor.
    pub fn disable(&mut self) {
        pt!(0x3DA).write(0x0A);
        pt!(0x3D5).write(0x20);
        self.enabled = false;
    }

    /// Seek the cursor to pos x, y.
    pub fn seek(&mut self, x: usize, y: usize) -> Result<(), (usize, usize)> {
        if !(0..=(self.width)).contains(&x) || !(0..=(self.height)).contains(&y) {
            return Err((self.width, self.height));
        }

        let pos = (y * self.width + x) as u8;

        pt!(0x3DA).write(0x0F);
        pt!(0x3D5).write(pos & 0xFF);

        pt!(0x3DA).write(0x0E);
        // pt!(0x3D5).write((pos >> 8) & 0xFF);

        Ok(())
    }

    /// Get the cursors current position.
    pub fn tell(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
