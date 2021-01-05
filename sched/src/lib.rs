//! A simple, innefficient, no-std futures executor.

#![no_std]

use core::{future::Future, task::{Context, RawWaker, Poll, Waker, RawWakerVTable}, pin::Pin};

extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque};

type Task<'a, T = ()> = Pin<Box<dyn Future<Output = T> + 'a>>;

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

#[derive(Default)]
pub struct Runtime<'a> {
    task_queue: VecDeque<Task<'a>>,
}

impl<'a> Runtime<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    fn spawn(&mut self, fut: impl Future<Output = ()> + 'a) {
        self.task_queue.push_back(Box::pin(fut))
    }

    fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.as_mut().poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }

    pub fn block_on(&mut self, fut: impl Future<Output = ()> + 'a) {
        self.spawn(fut);
        self.run();
    }
}
