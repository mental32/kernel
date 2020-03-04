use smallvec::{smallvec, SmallVec};
use spin::RwLock;

pub use super::LogProducer;

pub struct SystemLogger<'a> {
    level: usize,
    writers: Option<SmallVec<[RwLock<&'a dyn LogProducer>; 1]>>,
}

macro_rules! log_handler {
    ($name:ident) => {
        pub fn $name(&mut self, msg: &str) {
            if let Some(iterable) = self.writers.as_mut() {
                for writer in iterable.iter_mut() {
                    writer.write().$name(format_args!("{}", msg));
                }
            }
        }
    };
}

impl<'a> SystemLogger<'a> {
    pub const fn new() -> Self {
        Self {
            level: 0,
            writers: None,
        }
    }

    #[inline]
    pub fn add_producer(&mut self, producer: &'a dyn LogProducer) {
        if let Some(writers) = self.writers.as_mut() {
            writers.push(RwLock::new(producer));
        } else {
            self.writers = Some(smallvec![RwLock::new(producer)])
        }
    }

    log_handler!(info);
    // log_handler!(warn);
    // log_handler!(error);
    // log_handler!(fatal);
}
