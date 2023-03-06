use alloc::boxed::Box;
use core::{mem, ptr::NonNull, sync::atomic::Ordering};

use hal::{interrupts, task::context_switch};
use skua::mpsc_queue::{Link, MpscQueue};
use spin::{mutex::SpinMutex, Lazy};

use crate::task::{
    header::{Header, TaskVTable},
    raw_task::RawTask,
};

static STUB: Stub = Stub::new();
static GLOBAL: Lazy<Scheduler> = Lazy::new(|| Scheduler::with_static_stub(&STUB));

/// The global task scheduler.
///
/// ## Implementation Notes
/// Currently this uses a simple lockless MPSC queue. It is safe and can be used by
/// multiple cores, but isn't the most scalable. In the future move this to a work
/// stealing scheduler.
#[derive(Debug)]
pub struct Scheduler {
    // queue: MpscQueue<Header>,
    queue: TaskQueue,
    current: SpinMutex<RawTask>,
}

impl Scheduler {
    pub(crate) fn global() -> &'static Self {
        let g = &*GLOBAL;
        g.current.lock().header().scheduler.set(Some(g));
        g
    }

    pub fn with_static_stub(stub: &'static Stub) -> Self {
        Self {
            queue: TaskQueue::with_stub(stub),
            current: SpinMutex::new(base_task()),
        }
    }

    pub fn schedule(&self, task: RawTask) {
        task.header().scheduler.set(Some(&GLOBAL));

        interrupts::without(|_| {
            if !task.set_scheduled() {
                return;
            }
            self.queue.push(task);
        });
    }

    pub fn redispatch(&self) {
        if let Some(task) = self.pop_task() {
            unsafe {
                self.switch_to(task);
            }
        }
    }

    fn pop_task(&self) -> Option<RawTask> {
        interrupts::without(|_| {
            let task = self.queue.pop()?;
            task.mark_not_scheduled();
            Some(task)
        })
    }

    unsafe fn switch_to(&self, task: RawTask) {
        interrupts::without(|_| {
            let cur = mem::replace(&mut *self.current.lock(), task.clone());

            switch_tasks(&cur, &task);
        });
        // let mut saved_context = ptr::null_mut();
        // unsafe { context_switch(&mut saved_context, task.header()) }
    }

    pub fn current(&self) -> RawTask {
        self.current.lock().clone()
    }

    pub fn yield_to(&self, other: &RawTask) {
        interrupts::without(|_| {
            let cur = self.current();
            self.schedule(cur.clone());
            unsafe { switch_tasks(&cur, other) };
        });
    }

    pub fn yield_now(&self) {
        interrupts::without(|_| {
            if let Some(new) = self.pop_task() {
                self.schedule(self.current());
                unsafe { self.switch_to(new) };
            }
        });
    }
}

#[derive(Debug)]
struct TaskQueue {
    mpsc_queue: MpscQueue<Header>,
    lock: SpinMutex<()>,
}

impl TaskQueue {
    pub fn with_stub(stub: &'static Stub) -> Self {
        Self {
            mpsc_queue: MpscQueue::with_static_stub(&stub.link),
            lock: SpinMutex::new(()),
        }
    }

    pub fn pop(&self) -> Option<RawTask> {
        unsafe {
            let _guard = self.lock.lock();

            self.mpsc_queue
                .pop_unsync()
                .map(|raw| RawTask::from_raw(raw))
        }
    }

    pub fn push(&self, task: RawTask) {
        let raw = task.into_raw();
        unsafe { self.mpsc_queue.push(raw) };
    }
}

unsafe impl Sync for TaskQueue {}
unsafe impl Send for TaskQueue {}

#[derive(Debug)]
pub struct Stub {
    link: Link,
}

impl Stub {
    pub const fn new() -> Self {
        Self { link: Link::new() }
    }
}

unsafe fn switch_tasks(old: &RawTask, new: &RawTask) {
    context_switch(
        old.header().context.as_mut_ptr(),
        new.header().context.load(Ordering::Relaxed),
    );
}

fn base_task() -> RawTask {
    let mut header = Box::new(Header::new(&BASE_VTABLE));
    header.name = Some("<main>");
    let raw = NonNull::new(Box::into_raw(header)).unwrap();
    unsafe { RawTask::from_raw(raw.cast()) }
}

const BASE_VTABLE: TaskVTable = TaskVTable {
    drop_in_place,
    read_value_into,
    deallocate,
};

unsafe fn drop_in_place(_ptr: NonNull<Header>) {
    unreachable!();
}

unsafe fn read_value_into(_ptr: NonNull<Header>, _dst: *mut u8) {
    unreachable!();
}

unsafe fn deallocate(_ptr: *mut u8) {
    unreachable!()
}
