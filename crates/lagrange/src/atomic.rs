use alloc::{boxed::Box, vec::Vec};
use core::{
    hash::{BuildHasher, Hash, Hasher},
    mem,
    sync::atomic::{AtomicU32, Ordering},
    task::Waker,
};

use ahash::RandomState;
use spin::{mutex::SpinMutex, Lazy};

use crate::thread::{self, park};

pub fn wait(atomic: &AtomicU32, value: u32) {
    let bucket = PARKING_LOT.bucket(atomic);
    bucket.wait(atomic, value);
}

pub fn wake_one(atomic: *const AtomicU32) -> bool {
    let bucket = PARKING_LOT.bucket(atomic);
    bucket.wake_one(atomic as usize)
}

pub fn wake_all(atomic: *const AtomicU32) -> usize {
    let bucket = PARKING_LOT.bucket(atomic);
    bucket.wake_all(atomic as usize)
}

static PARKING_LOT: Lazy<ParkingLot> = Lazy::new(ParkingLot::new);

#[derive(Debug)]
struct ParkingLot {
    buckets: Box<[Bucket]>,
    hash_builder: RandomState,
    mask: u64,
}

impl ParkingLot {
    pub fn new() -> Self {
        let mask = 0;
        let buckets = Default::default();
        Self {
            hash_builder: RandomState::new(),
            buckets,
            mask,
        }
    }

    pub fn bucket(&self, atomic: *const AtomicU32) -> &Bucket {
        let mut hasher = self.hash_builder.build_hasher();
        (atomic as usize).hash(&mut hasher);
        let hash = hasher.finish();
        let index = hash & self.mask;
        &self.buckets[index as usize]
    }
}

#[derive(Debug)]
struct Bucket {
    queue: SpinMutex<Vec<Waiter>>,
}

impl Bucket {
    pub fn wake_all(&self, key: usize) -> usize {
        let mut queue = self.queue.lock();
        let waiters = queue.drain_filter(|waiter| waiter.key == key);
        let mut n = 0;
        for waiter in waiters {
            n += 1;
            waiter.waker.wake();
        }
        n
    }

    pub fn wake_one(&self, key: usize) -> bool {
        let mut queue = self.queue.lock();
        let mut waiters = queue.drain_filter(|waiter| waiter.key == key);
        if let Some(waiter) = waiters.next() {
            waiter.waker.wake();
            true
        } else {
            false
        }
    }

    pub fn wait(&self, atomic: &AtomicU32, value: u32) {
        let mut queue = self.queue.lock();
        let key = atomic as *const AtomicU32 as usize;
        let waiter = Waiter {
            key,
            waker: thread::current().into_waker(),
        };
        queue.push(waiter);
        mem::drop(queue);
        if atomic.load(Ordering::Acquire) == value {
            park();
        }
    }
}

#[derive(Debug)]
struct Waiter {
    key: usize,
    waker: Waker,
}
