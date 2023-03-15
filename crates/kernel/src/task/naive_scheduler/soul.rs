use core::{mem, sync::atomic::Ordering};

use hal::{
    interrupts::{self, enable_and_wait},
    task::context_switch,
};
use log::{debug, trace, warn};

use super::queue::TaskQueue;
use crate::task::{
    idle::allocate_bootstrap_task,
    task_types::{State, Task},
};

#[derive(Debug)]
pub struct Soul {
    active: Task,
    // local_queue: TailList<Task>,
    local_queue: TaskQueue,
    exited: Option<Task>,
}

impl Soul {
    pub unsafe fn new(queue: TaskQueue) -> Self {
        let active = allocate_bootstrap_task();
        Self {
            active,
            local_queue: queue,
            exited: None,
        }
    }

    pub fn exit(&mut self) -> ! {
        loop {
            let new = self.local_queue.pop().expect("no waiting tasks");
            self.switch(new, false);
            unreachable!();
        }
    }

    pub fn yield_now(&mut self) {
        let Some(new ) = self.local_queue.pop() else { return };
        self.switch(new, true);
    }

    pub fn unpark(&mut self, task: Task) {
        let was_parked = task
            .head()
            .state
            .compare_exchange(
                State::Parked,
                State::Queued,
                Ordering::AcqRel,
                Ordering::Relaxed,
            )
            .is_ok();

        if was_parked {
            self.local_queue.push(task);
        }
    }

    pub fn park(&mut self) {
        let new = self.local_queue.pop().expect("no waiting tasks");
        self.switch(new, false);
    }

    pub fn enter(&mut self) -> ! {
        loop {
            unsafe { interrupts::disable() };
            debug!("scheduler.enter.loop()");
            if let Some(new) = self.local_queue.pop() {
                self.switch(new, true);
            } else {
                warn!("halt");
                unsafe { enable_and_wait() };
            }
        }
    }

    fn switch(&mut self, new: Task, requeue: bool) {
        let old = mem::replace(&mut self.active, new);

        trace!("switch {} -> {}", old, self.active);

        self.active.change_state_to_active();

        let old_state = if requeue {
            self.local_queue.push(old.clone());
            State::Queued
        } else {
            State::Parked
        };

        old.change_state(State::Active, old_state)
            .expect("invalid task state transition");

        let old_ctx = old.head().stack_ptr.as_ptr();
        let new_ctx = self.active.saved_context();

        trace!("num_refs = {}", old.head().refs.load(Ordering::Relaxed));

        if requeue {
            mem::drop(old)
        } else {
            self.exited = Some(old);
        }

        unsafe {
            context_switch(old_ctx, new_ctx);
        }
    }

    pub fn current(&self) -> Task {
        self.active.clone()
    }
}
