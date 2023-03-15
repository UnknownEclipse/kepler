use core::mem;

use hal::interrupts;
use meteor::tail_list::TailList;
use spin::mutex::SpinMutex;

use super::{park, try_current, Task};
use crate::error::KernResult;

#[derive(Debug, Default)]
pub struct WaitList {
    list: SpinMutex<TailList<Task>>,
}

impl WaitList {
    pub const fn new() -> Self {
        Self {
            list: SpinMutex::new(TailList::new()),
        }
    }

    pub fn wait(&self) -> KernResult<()> {
        self.wait_if(&mut || true).map(|_| ())
    }

    pub fn wait_if(&self, f: &mut dyn FnMut() -> bool) -> KernResult<bool> {
        interrupts::without(|_| {
            let mut list = self.list.lock();
            if !f() {
                return Ok(false);
            }

            let this = try_current()?;
            list.push_back(this);
            mem::drop(list);

            // TODO: Ideally this would be try_park(), however we can't remove the task from the
            // list if that fails, which could result in bugs down the line. Better to
            // panic than have tricky bugs. If we ever make the backing list lockless
            // (or find another way around this), change to a try_park().
            // Also, park should only really fail if the scheduler is uninitialized,
            // and as we know the call to try_current() succeeded, that should
            // should never occur.
            park();
            Ok(true)
        })
    }

    pub fn wake_one(&self) -> Result<Task, ()> {
        todo!()
    }

    pub fn wake_if(&self, f: &mut dyn FnMut(&Task) -> bool) -> usize {
        todo!()
    }

    pub fn wake_all(&self) -> usize {
        self.wake_if(&mut |_| true)
    }
}

impl WaitList {}
