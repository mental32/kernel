use {spin::Mutex, cpuio::Port};

static A_PORT: Mutex<Port<u8>> = Mutex::new(unsafe { Port::new(0x3D4) });
static B_PORT: Mutex<Port<u8>> = Mutex::new(unsafe { Port::new(0x3D5) });

/// A VGA cursor.
#[derive(Debug)]
pub struct VGACursor {
    /// The x co-ordinate of the cursor.
    pub x: usize,

    /// The y co-ordinate of the cursor.
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
    pub fn enable(&mut self, start: u8, end: u8) {
        let mut a_port_handle = A_PORT.lock();
        let mut b_port_handle = B_PORT.lock();

        a_port_handle.write(0x0A);

        let mut res = b_port_handle.read();

        b_port_handle.write((res & 0xC0) | start);

        a_port_handle.write(0x0B);

        res = b_port_handle.read();
        b_port_handle.write((res & 0xE0) | end);

        self.enabled = true;
    }

    /// Disable the cursor.
    pub fn disable(&mut self) {
        A_PORT.lock().write(0x0A);
        B_PORT.lock().write(0x20);
        self.enabled = false;
    }

    /// Seek the cursor to pos x, y.
    pub fn seek(&mut self, width: usize, x: usize, y: usize) -> crate::Result<()> {
        let pos = (y * width + x) as u8;

        let mut a_port_handle = A_PORT.lock();
        let mut b_port_handle = B_PORT.lock();

        a_port_handle.write(0x0F);
        b_port_handle.write((pos & 0xFF));

        a_port_handle.write(0x0E);
        b_port_handle.write(((pos >> 8) & 0xFF));

        Ok(())
    }

    /// Get the cursors current position.
    pub fn tell(&self) -> (usize, usize) {
        (self.x, self.y)
    }
}
