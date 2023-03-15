use core::sync::atomic::{AtomicU32, Ordering};

use super::futex::{wait, wake_all};

pub struct Barrier {
    value: AtomicU32,
    target: u32,
}

impl Barrier {
    pub fn new(count: u32) -> Self {
        Self {
            value: AtomicU32::new(0),
            target: count,
        }
    }

    pub fn wait(&self) {
        let count = self.value.fetch_add(1, Ordering::Release) + 1;
        if count == self.target {
            wake_all(&self.value);
            return;
        }

        loop {
            let count = self.value.load(Ordering::Acquire);
            if count >= self.target {
                return;
            }
            wait(&self.value, count);
        }
    }
}
