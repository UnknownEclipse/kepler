use alloc::collections::VecDeque;

use meteor::tail_list::TailList;

use crate::task::Task;

pub type TaskQueue = LinkedQueue;

// #[derive(Debug, Default)]
// pub struct TaskQueue {
//     inner: SpinMutex<VecDeque<Task>>,
// }

// impl TaskQueue {
//     pub fn pop(&self) -> Option<Task> {
//         self.inner.lock().pop_front()
//     }

//     pub fn push(&self, task: Task) {
//         self.inner.lock().push_back(task);
//     }
// }

#[derive(Debug, Default)]
pub struct ArrayQueue {
    inner: VecDeque<Task>,
}

impl ArrayQueue {
    pub fn pop(&mut self) -> Option<Task> {
        self.inner.pop_front()
    }

    pub fn push(&mut self, task: Task) {
        self.inner.push_back(task);
    }
}

#[derive(Debug, Default)]
pub struct LinkedQueue {
    inner: TailList<Task>,
}

impl LinkedQueue {
    pub fn pop(&mut self) -> Option<Task> {
        self.inner.pop_front()
    }

    pub fn push(&mut self, task: Task) {
        self.inner.push_back(task);
    }
}
