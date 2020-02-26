use core::fmt::Arguments;

use serial::{SerialIO};
use vga::VGAWriter;

use crate::dev::vga::{VGAFramebuffer};

pub trait LogProducer {
    fn info(&mut self, _message: Arguments) {}
    fn warn(&mut self, _message: Arguments) {}
    fn error(&mut self, _message: Arguments) {}
    fn fatal(&mut self, _message: Arguments) {}
}

impl LogProducer for SerialIO {
    fn info(&mut self, message: Arguments) {
        self.print_args(format_args!("[INFO] {}\n", message));
    }
}

impl LogProducer for VGAWriter<'_, VGAFramebuffer<'_>> {
    fn info(&mut self, message: Arguments) {
        crate::vprint!(self, "{}", format_args!("[INFO] {}\n", message));
    }
}
