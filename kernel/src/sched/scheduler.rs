use x86_64::instructions::hlt;

use super::thread::ThreadIdent;

pub trait KernelScheduler {
    fn schedule(&mut self) -> Option<ThreadIdent>;
    fn park(&mut self) -> Result<ThreadIdent, &'static str>;
    fn spawn(&mut self, f: impl FnOnce() -> ()) -> Result<ThreadIdent, &'static str>;
    fn exists(&self, ident: ThreadIdent) -> bool;
    fn thread_count(&self) -> usize;

    /// Keeps the running executor active until all tasks have ended.
    fn run_until_complete(&mut self) {
        while self.thread_count() != 0 {
            hlt()
        }
    }

    /// Spawns an idle task and calls run_until_complete.
    fn run_forever(&mut self) -> Result<(), &'static str> {
        self.spawn(|| loop {
            hlt()
        })?;

        self.run_until_complete();
        Ok(())
    }
}
