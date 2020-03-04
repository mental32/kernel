use core::fmt::Arguments;

use spin::Mutex;

use serial::{SerialIO, GLOBAL_DEFAULT_SERIAL, sprint};
use vga::VGAWriter;

use crate::dev::vga::{VGAFramebuffer};

pub trait LogProducer: Sync + Send {
    fn info(&self, _message: Arguments) {}
    fn warn(&self, _message: Arguments) {}
    fn error(&self, _message: Arguments) {}
    fn fatal(&self, _message: Arguments) {}
}

impl LogProducer for SerialIO {
    fn info(&self, message: Arguments) {
        self.print_args(format_args!("[INFO] {}\n", message));
    }
}

impl LogProducer for GLOBAL_DEFAULT_SERIAL {
    fn info(&self, message: Arguments) {
        sprint!("{}", format_args!("[INFO] {}\n", message));
    }
}

impl LogProducer for Mutex<VGAWriter<'_, VGAFramebuffer<'_>>> {
    fn info(&self, message: Arguments) {
        let mut handle = self.lock();
        crate::vprint!(handle, "{}", format_args!("[INFO] {}\n", message));
    }
}
