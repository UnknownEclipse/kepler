use alloc::{boxed::Box, vec::Vec};

use ahash::RandomState;
use spin::mutex::SpinMutex;

use crate::thread::Thread;

#[derive(Debug)]
struct ParkingLot {
    buckets: Box<[Bucket]>,
    mask: u64,
    hash_builder: RandomState,
}

impl ParkingLot {
    fn bucket(&self, key: usize) -> &Bucket {
        todo!()
    }
}

#[derive(Debug)]
struct Bucket {
    queue: SpinMutex<Vec<Waiter>>,
}

#[derive(Debug)]
struct Waiter {
    key: usize,
    thread: Thread,
}
