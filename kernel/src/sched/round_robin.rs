use alloc::collections::{BTreeMap, BTreeSet, VecDeque};

use core::mem;

use x86_64::VirtAddr;

use serial::sprintln;

use super::{
    multitasking::{
        context_switch,
        thread::{Thread, ThreadIdent},
    },
    scheduler::{KernelScheduler, SwitchReason},
};
use crate::info;

#[derive(Debug)]
pub struct RoundRobin {
    inner: Option<RoundRobinState>,
}

#[derive(Debug)]
struct RoundRobinState {
    pub threads: BTreeMap<ThreadIdent, Thread>,
    pub idle_thread_id: Option<ThreadIdent>,
    pub current_thread_id: ThreadIdent,
    pub paused_threads: VecDeque<ThreadIdent>,
    pub blocked_threads: BTreeSet<ThreadIdent>,
    pub wakeups: BTreeSet<ThreadIdent>,
}

impl RoundRobinState {
    pub fn new() -> Self {
        let root_thread = Thread::create_root_thread();
        let root_id = root_thread.id();
        let mut threads = BTreeMap::new();

        threads
            .insert(root_id, root_thread)
            .expect_none("map is not empty after creation");

        Self {
            threads,
            current_thread_id: root_id,
            paused_threads: VecDeque::new(),
            blocked_threads: BTreeSet::new(),
            wakeups: BTreeSet::new(),
            idle_thread_id: None,
        }
    }

    pub fn next_thread(&mut self) -> Option<ThreadIdent> {
        self.paused_threads.pop_front()
    }

    fn check_for_wakeup(&mut self, thread_id: ThreadIdent) {
        if self.wakeups.remove(&thread_id) {
            assert!(self.blocked_threads.remove(&thread_id));
            self.paused_threads.push_back(thread_id);
        }
    }
}

impl RoundRobin {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    pub fn init(&mut self) {
        if self.inner.is_some() {
            panic!("scheduler already initialized!");
        } else {
            self.inner = Some(RoundRobinState::new());
        }
    }
}

impl KernelScheduler for RoundRobin {
    fn current_thread_id(&self) -> ThreadIdent {
        self.inner.as_ref().unwrap().current_thread_id
    }

    fn synchronous_context_switch(&mut self, reason: SwitchReason) -> Result<(), ()> {
        match self.schedule() {
            Some((next_stack_pointer, prev_thread_id)) => unsafe {
                context_switch::context_switch_to(next_stack_pointer, prev_thread_id, reason);
                Ok(())
            },

            None => Err(()),
        }
    }

    fn add_paused_thread(
        &mut self,
        paused_stack_pointer: VirtAddr,
        paused_thread_id: ThreadIdent,
        switch_reason: SwitchReason,
    ) {
        let state = self.inner.as_mut().unwrap();
        let paused_thread = state
            .threads
            .get_mut(&paused_thread_id)
            .expect("paused thread does not exist");

        paused_thread
            .stack_pointer()
            .replace(paused_stack_pointer)
            .expect_none("running thread should have stack pointer set to None");

        if Some(paused_thread_id) == state.idle_thread_id {
            return; // do nothing
        }

        match switch_reason {
            SwitchReason::Paused | SwitchReason::Yield => {
                state.paused_threads.push_back(paused_thread_id)
            }
            SwitchReason::Blocked => {
                state.blocked_threads.insert(paused_thread_id);
                state.check_for_wakeup(paused_thread_id);
            }

            SwitchReason::Exit => {
                let _ = state
                    .threads
                    .remove(&paused_thread_id)
                    .expect("thread not found");
                // TODO: free stack memory again
            }
        }
    }

    fn schedule(&mut self) -> Option<(VirtAddr, ThreadIdent)> {
        let state = self.inner.as_mut().unwrap();

        let next_thread_id = state.next_thread().unwrap_or_else(|| {
            state
                .idle_thread_id
                .expect("System idle thread is not active.")
        });

        let next_thread = state
            .threads
            .get_mut(&next_thread_id)
            .expect("next thread does not exist");

        let next_stack_pointer = next_thread
            .stack_pointer()
            .take()
            .expect("paused thread has no stack pointer");

        let prev_thread_id = mem::replace(&mut state.current_thread_id, next_thread.id());

        Some((next_stack_pointer, prev_thread_id))
    }

    fn park(&mut self) -> Result<ThreadIdent, &'static str> {
        Err("Cant park")
    }

    fn spawn(&mut self, f: fn() -> !) -> Result<ThreadIdent, &'static str> {
        let thread = Thread::create_with_stack(f, 4).unwrap();

        let thread_id = thread.id();

        let state = self.inner.as_mut().unwrap();

        state
            .threads
            .insert(thread_id, thread)
            .expect_none("thread already exists");

        state.paused_threads.push_back(thread_id);

        Ok(thread_id)
    }

    fn exists(&self, _ident: ThreadIdent) -> bool {
        false
    }

    fn thread_count(&self) -> usize {
        0
    }
}
