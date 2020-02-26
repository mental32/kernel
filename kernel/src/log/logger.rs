use smallvec::{SmallVec};

pub use super::LogProducer;

pub type Writers<'a> = SmallVec<[&'a mut dyn LogProducer; 4]>;

pub struct SystemLogger<'a> {
    level: usize,
    writers: Writers<'a>,
}

macro_rules! log_handler {
    ($name:ident) => {

        pub fn $name(&mut self, msg: &str) {
            for writer in self.writers.iter_mut() {
                writer.$name(format_args!("{}", msg));
            }
        }

    }
}

impl<'a> SystemLogger<'a> {
    pub fn new(writers: Writers<'a>) -> Self {
        Self {
            level: 0,
            writers,
        }
    }

    log_handler!(info);
    // log_handler!(warn);
    // log_handler!(error);
    // log_handler!(fatal);
}
