use spin::Mutex;

use {
    super::{
        scheduler::KernelScheduler,
        thread::{ThreadControlBlock, ThreadIdent},
    },
    crate::state::KernelStateObject,
};

pub struct RoundRobin {
    // threads: VecDeque<ThreadControlBlock>,
    threads: [ThreadControlBlock; 2],
}

impl RoundRobin {
    pub fn new(_state: &Mutex<KernelStateObject>) -> Self {
        // let threads = VecDeque::new();
        let threads = [ThreadControlBlock::new(0), ThreadControlBlock::new(1)];
        Self { threads }
    }
}

impl KernelScheduler for RoundRobin {
    fn schedule(&mut self) -> Option<ThreadIdent> {
        None
    }

    fn park(&mut self) -> Result<ThreadIdent, &'static str> {
        Err("Cant park")
    }

    fn spawn(&mut self, _f: impl FnOnce() -> ()) -> Result<ThreadIdent, &'static str> {
        Err("Cant spawn")
    }

    fn exists(&self, _ident: ThreadIdent) -> bool {
        false
    }

    fn thread_count(&self) -> usize {
        0
    }
}
