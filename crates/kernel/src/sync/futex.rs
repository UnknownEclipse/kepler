//! This crate provides kernel-level futex primitives. Userspace futexes are built on
//! top of this. (Namely, user addresses are translated to their kernel equivalents)

use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{
    cell::Cell,
    hash::{BuildHasher, Hash, Hasher},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use ahash::RandomState;
use hal::interrupts;
use meteor::{
    tail_list::{self, TailList},
    Node,
};
use spin::{mutex::SpinMutex, Lazy};
use tracing::trace;

use crate::task::{self, Task};

pub fn wait(atomic: &AtomicU32, value: u32) {
    tracing::trace!("futex.wait({:?})", FutexKey::from_atomic(atomic));
    let bucket = TABLE.bucket(atomic);
    bucket.wait(atomic, value);
}

pub fn wake_one(atomic: *const AtomicU32) -> bool {
    tracing::trace!("futex.wake_one({:?})", FutexKey::from_atomic(atomic));
    let bucket = TABLE.bucket(atomic);
    bucket.wake_one(atomic)
}

pub fn wake_all(atomic: *const AtomicU32) -> usize {
    tracing::trace!("futex.wake_all({:?})", FutexKey::from_atomic(atomic));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FutexKey(usize);

impl FutexKey {
    pub fn from_atomic(atomic: *const AtomicU32) -> Self {
        Self(atomic as usize)
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
            let key = FutexKey::from_atomic(atomic);

            let mut queue = self.queue.lock();
            if atomic.load(Ordering::Acquire) != value {
                return;
            }

            queue.push_back(Waiter {
                key,
                thread: task::current(),
            });

            drop(queue);

            task::park();
        })
    }

    pub fn wake_one(&self, atomic: *const AtomicU32) -> bool {
        interrupts::without(|_| {
            let mut queue = self.queue.lock();
            let key = FutexKey::from_atomic(atomic);

            for i in 0..queue.len() {
                if queue[i].key == key {
                    if let Some(waiter) = queue.remove(i) {
                        trace!("futex.wake_one");
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
            let key = FutexKey::from_atomic(atomic);

            let mut i = 0;
            while i < queue.len() {
                if queue[i].key == key {
                    if let Some(waiter) = queue.remove(i) {
                        trace!("futex.wake_all");
                        waiter.thread.unpark();
                        woken += 1;
                    }
                } else {
                    i += 1;
                }
            }

            woken
        })
    }
}

type PinListTypes = dyn pin_list::Types<
    Id = pin_list::id::DebugChecked,
    Protected = (),
    Removed = (),
    Unprotected = (),
>;

#[derive(Default)]
struct Bucket2 {
    /// Todo: Consider using a priority queue/btreemap here
    queue: SpinMutex<TailList<WaiterRef>>,
}

impl Bucket2 {
    pub fn wait(&self, atomic: &AtomicU32, value: u32) {
        interrupts::without(|_| {
            let key = FutexKey::from_atomic(atomic);

            let mut queue = self.queue.lock();

            if atomic.load(Ordering::Acquire) != value {
                return;
            }

            let waiter = Waiter2 {
                key,
                thread: Cell::new(Some(task::current())),
                link: tail_list::Link::new(),
            };

            let node = WaiterRef(NonNull::from(&waiter));
            queue.push_back(node);

            drop(queue);
            let task = task::current();
            trace!("futex.wait({:?})", task);
            task::park();
        })
    }

    pub fn wake_one(&self, atomic: *const AtomicU32) -> bool {
        interrupts::without(|_| {
            let mut queue = self.queue.lock();
            let key = FutexKey::from_atomic(atomic);

            let waiter = queue
                .drain_filter(|waiter| unsafe { waiter.0.as_ref().key == key })
                .next();

            match waiter {
                Some(waiter) => {
                    waiter.wake();
                    true
                }
                None => false,
            }
        })
    }

    pub fn wake_all(&self, atomic: *const AtomicU32) -> usize {
        interrupts::without(|_| {
            let mut queue = self.queue.lock();

            let key = FutexKey::from_atomic(atomic);

            let waiters = queue.drain_filter(|waiter| unsafe { waiter.0.as_ref().key == key });
            let mut woken = 0;
            for waiter in waiters {
                woken += 1;
                waiter.wake();
            }
            woken
        })
    }
}

#[derive(Debug)]
struct Waiter {
    key: FutexKey,
    thread: Task,
    // link: tail_queue::Link,
}

struct Waiter2 {
    key: FutexKey,
    thread: Cell<Option<Task>>,
    link: tail_list::Link,
}

struct WaiterRef(NonNull<Waiter2>);

impl WaiterRef {
    fn wake(self) {
        let task = unsafe { self.0.as_ref().thread.take().unwrap_unchecked() };
        trace!("futex.wake({:?})", task);
        task.unpark();
    }
}

impl Node<tail_list::Link> for WaiterRef {
    fn into_link(node: Self) -> NonNull<tail_list::Link> {
        node.0.cast()
    }

    unsafe fn from_link(link: NonNull<tail_list::Link>) -> Self {
        Self(link.cast())
    }
}
