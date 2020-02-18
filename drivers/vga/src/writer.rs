use {
    crate::{Attribute, Color, RawChar, VGABuffer, VGACursor},
    core::fmt::{self, Write},
};

#[derive(Debug, PartialEq)]
enum VGAStatus {
    Normal,
    Escape,
    CSI,
}

/// Struct responsible for writing data into the VGA address space.
pub struct VGAWriter<'a, T: VGABuffer> {
    /// The VGA cursor that gets used.
    pub cursor: VGACursor,

    attr: Attribute,
    buffer: &'a mut T,
    csi_param: Option<usize>,
    status: VGAStatus,
}

impl<'a, T: VGABuffer> Write for VGAWriter<'a, T> {
    fn write_str(&mut self, st: &str) -> fmt::Result {
        self.print_str(st).unwrap();
        Ok(())
    }
}

impl<'a, T: VGABuffer> VGAWriter<'a, T> {
    /// Create a new writer that operates over a fixed sized
    /// arena of video memory.
    pub fn new(buffer: &'a mut T) -> Self {
        Self {
            attr: Attribute::default(),
            cursor: VGACursor::new(buffer.width(), buffer.height()),
            status: VGAStatus::Normal,
            csi_param: None,
            buffer,
        }
    }

    /// Set the writers attribute.
    pub fn set_attr(&mut self, attr: Attribute) {
        self.attr = attr;
    }

    /// Get the size of the VGABuffer as a (width, height) tuple.
    pub fn size(&self) -> (usize, usize) {
        (self.buffer.width(), self.buffer.height())
    }

    /// Fill the buffer with a single character.
    pub fn print_fill_char(&mut self, ch: char) -> Result<(), (usize, usize)> {
        self.cursor.x = 0;
        self.cursor.y = 0;

        let (width, height) = self.size();

        for x in 0..width {
            for y in 0..height {
                self.set_byte(x, y, ch as u8)?;
            }
        }

        Ok(())
    }

    /// Print a string.
    pub fn print_str(&mut self, st: &str) -> Result<(), (usize, usize)> {
        for byte in st.bytes() {
            self.print_char(byte as char)?;
        }

        Ok(())
    }

    /// Print a single character byte.
    pub fn print_char(&mut self, ch: char) -> Result<(), (usize, usize)> {
        use VGAStatus::*;

        if self.status == CSI {
            return self.handle_csi(ch);
        }

        if self.status == Escape {
            if ch == '[' {
                self.status = CSI;
            }

            return Ok(());
        }

        match ch as u8 {
            0x1B => {
                self.status = Escape;
            }

            // \b
            0x08 => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            }

            // \n
            0x0A => {
                self.cursor.y += 1;
                self.cursor.x = 0;

                if self.cursor.y >= self.buffer.height() {
                    self.scroll()?;
                    self.clear_row(self.buffer.height() - 1)?;
                    self.cursor.y = self.buffer.height() - 1;
                }
            }

            0x20..=0x7E => self.write(ch)?,
            _ => self.write(0xfe as char)?,
        }

        Ok(())
    }

    fn write(&mut self, ch: char) -> Result<(), (usize, usize)> {
        let mut col = self.cursor.x;
        let mut row = self.cursor.y;

        if col == self.buffer.width() {
            if row == self.buffer.height() - 1 {
                self.scroll()?;
                self.clear_row(self.buffer.height() - 1)?;
                row = self.buffer.height() - 1;
            } else {
                row += 1;
            }

            col = 0;
        }

        assert!(row < self.buffer.height());
        assert!(col < self.buffer.width());
        self.set_byte(col, row, ch as u8)?;

        col += 1;

        self.cursor.x = col;
        self.cursor.y = row;

        Ok(())
    }

    fn handle_csi(&mut self, ch: char) -> Result<(), (usize, usize)> {
        match ch {
            '0'..='9' => {}

            ';' => {}

            '@'..='~' => {
                self.handle_control_seq(ch)?;
            }

            _ => (),
        }

        Ok(())
    }

    fn handle_control_seq(&mut self, ch: char) -> Result<(), (usize, usize)> {
        if ('A'..'G').contains(&ch) {
            match ch {
                'A' => {
                    let n = self.csi_param.unwrap_or(1);

                    if self.cursor.y != n {
                        self.cursor.y -= n;
                    } else {
                        self.cursor.y = 0;
                    }
                }

                'B' => {}

                'C' => {}

                'D' => {}

                'E' => {}

                'F' => {}

                'G' => {}

                _ => return Ok(()),
            }

            self.cursor.seek(self.cursor.x, self.cursor.y).unwrap();

            return Ok(());
        }

        match ch {
            'J' => {
                let n = self.csi_param.unwrap_or(0);

                match n {
                    0 => {
                        for i in (self.cursor.x)..(self.buffer.width() + 1) {
                            self.set_byte(i, self.cursor.y, ' ' as u8)?;
                        }

                        for i in (self.cursor.y + 1)..(self.buffer.height()) {
                            self.set_byte(i, self.cursor.y, ' ' as u8)?;
                        }
                    }

                    1 => {
                        for i in ((self.cursor.x)..=0).rev() {
                            self.set_byte(i, self.cursor.y, ' ' as u8)?;
                        }

                        for i in ((self.cursor.y)..=0).rev() {
                            self.set_byte(i, self.cursor.y, ' ' as u8)?;
                        }
                    }

                    2 => {
                        for i in 0..(self.buffer.height()) {
                            self.set_byte(i, self.cursor.y, ' ' as u8)?;
                        }
                    }

                    _ => (),
                }
            }

            'K' => {
                let n = self.csi_param.unwrap_or(0);

                if n == 1 {
                    let seq = (0..(self.cursor.x)).rev();
                    for i in seq {
                        self.set_byte(i, self.cursor.y, ' ' as u8)?;
                    }
                } else {
                    let seq = match n {
                        0 => (self.cursor.x)..(self.buffer.width()),
                        2 => (self.cursor.x)..(self.buffer.width() + 1),
                        _ => return Ok(()),
                    };

                    for i in seq {
                        self.set_byte(i, self.cursor.y, ' ' as u8)?;
                    }
                }
            }

            'm' => {
                if let Some(n) = self.csi_param {
                    let color = match Color::from_usize(n % 10) {
                        Some(color) => color,
                        None => return Ok(()),
                    };

                    if n / 10 == 3 {
                        self.attr.set_foreground(color);
                    } else {
                        self.attr.set_background(color);
                    }
                }
            }

            _ => (),
        }

        Ok(())
    }

    /// Set a byte on the video memory.
    ///
    /// # Examples
    ///
    /// ```
    /// writer.set_byte(0, 0, 'A');  // Sets the top left character to "A"
    /// ```
    fn set_byte(&mut self, x: usize, y: usize, byte: u8) -> Result<(), (usize, usize)> {
        self.buffer.write(
            x,
            y,
            RawChar {
                data: byte,
                attr: self.attr,
            },
        )
    }

    fn scroll(&mut self) -> Result<(), (usize, usize)> {
        for row in 1..self.buffer.height() {
            for col in 0..self.buffer.width() {
                assert!(row < self.buffer.height());
                assert!(col < self.buffer.width());

                let char_ = self.buffer.read(col, row)?;

                assert!((row - 1) < self.buffer.height());
                self.buffer.write(col, row - 1, char_)?;
            }
        }

        Ok(())
    }

    fn clear_row(&mut self, row: usize) -> Result<(), (usize, usize)> {
        let blank = RawChar {
            data: b' ',
            attr: self.attr,
        };

        for col in 0..self.buffer.width() {
            self.buffer.write(col, row, blank)?;
        }

        Ok(())
    }
}
