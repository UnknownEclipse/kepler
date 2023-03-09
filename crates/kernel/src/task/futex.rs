use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{
    hash::{BuildHasher, Hash, Hasher},
    sync::atomic::{AtomicU32, Ordering},
};

use ahash::RandomState;
use hal::interrupts;
use spin::{mutex::SpinMutex, Lazy};

use super::thread::Thread;
use crate::task::thread;

pub fn wait(atomic: &AtomicU32, value: u32) {
    let bucket = TABLE.bucket(atomic);
    bucket.wait(atomic, value);
}

pub fn wake_one(atomic: *const AtomicU32) -> bool {
    interrupts::without(|_| {
        let bucket = TABLE.bucket(atomic);
        bucket.wake_one(atomic)
    })
}

pub fn wake_all(atomic: *const AtomicU32) -> usize {
    let bucket = TABLE.bucket(atomic);
    bucket.wake_all(atomic)
}

static TABLE: Lazy<Table> = Lazy::new(Table::new);

#[derive(Debug)]
struct Table {
    buckets: Box<[Bucket]>,
    hash_builder: RandomState,
    mask: u64,
}

impl Table {
    pub fn new() -> Self {
        let num_buckets = 64;
        let mut buckets = Vec::with_capacity(num_buckets);
        buckets.resize_with(num_buckets, Default::default);
        let buckets = buckets.into_boxed_slice();

        Self {
            buckets,
            hash_builder: RandomState::new(),
            mask: (num_buckets as u64 - 1),
        }
    }

    pub fn bucket(&self, atomic: *const AtomicU32) -> &Bucket {
        let mut hasher = self.hash_builder.build_hasher();
        (atomic as usize).hash(&mut hasher);
        let hash = hasher.finish();
        let bucket = hash & self.mask;
        &self.buckets[bucket as usize]
    }
}

#[derive(Debug, Default)]
struct Bucket {
    /// Todo: Consider using a priority queue/btreemap here
    queue: SpinMutex<VecDeque<Waiter>>,
}

impl Bucket {
    pub fn wait(&self, atomic: &AtomicU32, value: u32) {
        interrupts::without(|_| {
            let mut queue = self.queue.lock();

            queue.push_back(Waiter {
                key: atomic as *const AtomicU32 as usize,
                thread: thread::current(),
            });

            drop(queue);
            if atomic.load(Ordering::Acquire) != value {
                return;
            }
            thread::park();
        })
    }

    pub fn wake_one(&self, atomic: *const AtomicU32) -> bool {
        interrupts::without(|_| {
            let key = atomic as usize;
            let mut queue = self.queue.lock();

            for i in 0..queue.len() {
                if queue[i].key == key {
                    if let Some(waiter) = queue.remove(i) {
                        waiter.thread.unpark();
                        return true;
                    }
                }
            }
            false
        })
    }

    pub fn wake_all(&self, atomic: *const AtomicU32) -> usize {
        interrupts::without(|_| {
            let mut queue = self.queue.lock();
            let mut woken = 0;
            let key = atomic as usize;

            for _ in 0..queue.len() {
                if let Some(waiter) = queue.pop_front() {
                    if waiter.key == key {
                        waiter.thread.unpark();
                        woken += 1;
                    } else {
                        queue.push_back(waiter);
                    }
                }
            }
            woken
        })
    }
}

#[derive(Debug)]
struct Waiter {
    key: usize,
    thread: Thread,
}
