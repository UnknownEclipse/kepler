#![no_std]

use core::sync::atomic::AtomicU32;

pub struct QueueHead {
    head: usize,
    tail: usize,
    futex: AtomicU32,
}
