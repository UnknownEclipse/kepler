use core::{cell::SyncUnsafeCell, mem};

use hal::interrupts;
use meteor::tail_queue::TailQueue;
use spin::{mutex::SpinMutex, Once};

mod soul;

use self::soul::Soul;
use super::{sched::Scheduler, task_types::Task};

/// A dumb as rocks single-soul scheduler.
pub struct NaiveScheduler {
    soul: Once<SyncUnsafeCell<Soul>>,
    queue: SpinMutex<TailQueue<Task>>,
}

impl NaiveScheduler {
    pub fn new() -> Self {
        Self {
            soul: Once::new(),
            queue: Default::default(),
        }
    }

    fn soul(&self) -> *mut Soul {
        self.soul.get().unwrap().get()
    }
}

impl Scheduler for NaiveScheduler {
    fn unpark(&self, task: Task) {
        interrupts::without(|_| unsafe {
            if let Some(soul) = self.soul.get() {
                (*soul.get()).unpark(task);
            } else {
                self.queue.lock().push(task);
            }
        });
    }

    fn park(&self) {
        interrupts::without(|_| unsafe { (*self.soul()).park() });
    }

    fn current(&self) -> Task {
        interrupts::without(|_| unsafe { (*self.soul()).current() })
    }

    fn yield_now(&self) {
        interrupts::without(|_| unsafe { (*self.soul()).yield_now() })
    }

    fn exit(&self) -> ! {
        interrupts::without(|_| unsafe { (*self.soul()).exit() })
    }

    unsafe fn enter(&self) -> ! {
        let queue = mem::take(&mut *self.queue.lock());
        self.soul
            .call_once(|| SyncUnsafeCell::new(Soul::new(queue)));

        interrupts::without(|_| unsafe { (*self.soul()).enter() })
    }
}

// pub struct NaiveScheduler {
//     current: Once<SpinMutex<Task>>,
//     queue: TaskQueue,
//     drop_pending: SpinMutex<Vec<Task>>,
// }

// impl NaiveScheduler {
//     fn yield_inner(&self, reschedule: bool) -> bool {
//         // self.clean_exited();

//         let new = self.queue.pop();
//         if let Some(new) = new {
//             change_state(&new, State::Queued, State::Active);

//             let old = self.swap_current(new.clone());

//             trace!("switch {:?} -> {:?}", old, new);

//             if reschedule {
//                 change_state(&old, State::Active, State::Queued);
//                 self.queue.push(old.clone());
//             } else {
//                 change_state(&old, State::Active, State::Parked);
//             }
//             unsafe { task_switch(&old, &new) };
//             trace!("end switch");
//             true
//         } else {
//             false
//         }
//     }

//     fn exit_inner(&self) -> ! {
//         trace!("scheduler.exit");

//         let Some(new) = self.queue.pop() else {
//             panic!("scheduler.exit: no queued tasks");
//         };
//         trace!("new: {:?}", new);

//         let old = self.current();
//         if old.change_state(State::Active, State::Exited).is_err() {
//             panic!("scheduler.exit: inconsistent task state");
//         }

//         new.change_state(State::Queued, State::Active).unwrap();
//         // let mut drop_pending = self.drop_pending.lock();
//         // drop_pending.push(old);
//         // let old: *const Task = drop_pending.last().unwrap();
//         // unsafe { trace!("{}", (*old).head().refs.load(Ordering::Acquire)) };
//         // mem::drop(drop_pending);
//         trace!("switching away");
//         unsafe { task_switch(&old, &new) };
//         unreachable!();
//     }

//     fn unpark_inner(&self, task: Task) {
//         // self.clean_exited();

//         if !self.entered() {
//             if task.change_state(State::Parked, State::Queued).is_err() {
//                 error!("task in unexpected state");
//                 return;
//             }
//             self.queue.push(task);
//             return;
//         }

//         let current = self.current();

//         if task.head().policy.should_preempt(current.head().policy) {
//             trace!("unparking task, preempting current");

//             if task.change_state(State::Parked, State::Active).is_err() {
//                 panic!("invalid state change");
//             }

//             let old = self.swap_current(task.clone());
//             self.queue.push(old.clone());
//             unsafe { task_switch(&old, &task) };
//         } else {
//             if task.change_state(State::Parked, State::Queued).is_err() {
//                 trace!("skipping task unpark (not actually parked)");
//                 return;
//             }
//             self.queue.push(task);
//         }
//     }

//     unsafe fn enter_inner(&self) -> ! {
//         let root = allocate_bootstrap_task();
//         self.current.call_once(|| SpinMutex::new(root));

//         loop {
//             let Some(new) = self.queue.pop() else {
//                 panic!();
//             };

//             change_state(&new, State::Queued, State::Active);
//             let old = self.swap_current(new.clone());
//             change_state(&old, State::Active, State::Queued);
//             self.queue.push(old.clone());
//             task_switch(&old, &new);
//         }
//     }

//     fn entered(&self) -> bool {
//         self.current.get().is_some()
//     }

//     fn swap_current(&self, new: Task) -> Task {
//         mem::replace(&mut *self.current.get().unwrap().lock(), new)
//     }

//     pub fn new() -> Self {
//         Self {
//             current: Once::new(),
//             queue: Default::default(),
//             drop_pending: Default::default(),
//         }
//     }

//     fn clean_exited(&self) {
//         self.drop_pending.lock().clear();
//     }
// }

// impl Scheduler for NaiveScheduler {
//     fn unpark(&self, task: Task) {
//         interrupts::without(|_| self.unpark_inner(task));
//     }

//     fn park(&self) {
//         interrupts::without(|_| {
//             self.yield_inner(false);
//         })
//     }

//     fn current(&self) -> Task {
//         interrupts::without(|_| self.current.get().unwrap().lock().clone())
//     }

//     fn yield_now(&self) {
//         interrupts::without(|_| self.yield_inner(true));
//     }

//     fn exit(&self) -> ! {
//         interrupts::without(|_| self.exit_inner())
//     }

//     unsafe fn enter(&self) -> ! {
//         if interrupts::are_enabled() {
//             interrupts::disable();
//         }

//         self.enter_inner()
//     }
// }

// fn change_state(task: &Task, old: State, new: State) {
//     task.change_state(old, new).expect("invalid state")
// }

// #[derive(Debug, Default)]
// struct TaskQueue {
//     queue: SpinMutex<VecDeque<Task>>,
// }

// impl TaskQueue {
//     pub fn pop(&self) -> Option<Task> {
//         self.queue.lock().pop_front()
//     }

//     pub fn push(&self, task: Task) {
//         self.queue.lock().push_back(task);
//     }
// }
