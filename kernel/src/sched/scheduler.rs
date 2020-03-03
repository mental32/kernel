use x86_64::{instructions::hlt, VirtAddr};

use super::multitasking::ThreadIdent;

#[derive(Debug)]
#[repr(u64)]
pub enum SwitchReason {
    Paused,
    Yield,
    Blocked,
    Exit,
}

pub trait KernelScheduler {
    fn schedule(&mut self) -> Option<(VirtAddr, ThreadIdent)>;
    fn park(&mut self) -> Result<ThreadIdent, &'static str>;
    fn spawn(&mut self, f: fn() -> !) -> Result<ThreadIdent, &'static str>;
    fn exists(&self, ident: ThreadIdent) -> bool;
    fn thread_count(&self) -> usize;
    fn current_thread_id(&self) -> ThreadIdent;
    fn synchronous_context_switch(&mut self, reason: SwitchReason) -> Result<(), ()>;
    fn add_paused_thread(
        &mut self,
        paused_stack_pointer: VirtAddr,
        paused_thread_id: ThreadIdent,
        switch_reason: SwitchReason,
    );

    fn exit_thread(&mut self) -> ! {
        self.synchronous_context_switch(SwitchReason::Exit)
            .expect("can't exit last thread");
        unreachable!("finished thread continued");
    }

    fn yield_now(&mut self) {
        let _ = self.synchronous_context_switch(SwitchReason::Yield);
    }

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
