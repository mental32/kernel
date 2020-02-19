use core::mem::size_of;

use volatile::Volatile;

use vga::{RawChar, VGABuffer};

pub struct Framebuffer {
    width: usize,
    height: usize,
    addr: usize,
}

impl Framebuffer {
    pub fn new(addr: usize, width: usize, height: usize) -> Self {
        Self {
            addr,
            width,
            height,
        }
    }
}

// /// Like print!
// #[macro_export]
// macro_rules! sprintln {
//     ($writer:ident, $($arg:tt)*) => {
//         x86_64::instructions::interrupts::without_interrupts(|| {
//             $writer.write_fmt(format_args!($($arg)*)).unwrap();
//         })
//     };

//     ($($arg:tt)*) => {
//         {
//             let mut writer = $crate::GLOBAL_WRITER.lock();
//             $crate::sprintln!(writer, $($arg)*);
//         }
//     }
// }

impl VGABuffer for Framebuffer {
    fn height(&self) -> usize {
        self.height
    }

    fn width(&self) -> usize {
        self.width
    }

    fn write(&mut self, x: usize, y: usize, ch: RawChar) -> Result<(), (usize, usize)> {
        if y >= self.height() || x >= self.width() {
            return Err(((self.width(), self.height())));
        }

        unsafe {
            let ptr = (self.addr
                + (size_of::<RawChar>() * x)
                + (size_of::<RawChar>() * self.width() * y))
                as *mut Volatile<RawChar>;

            (&mut *ptr).write(ch);
        };

        Ok(())
    }

    fn read(&self, x: usize, y: usize) -> Result<RawChar, (usize, usize)> {
        // if y >= self.height() || x >= self.width() {
        return Err((self.width(), self.height()));
        // }

        // Ok(self.data[y][x].read())
    }
}
