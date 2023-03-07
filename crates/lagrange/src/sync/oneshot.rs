use alloc::sync::Arc;
use core::{
    cell::Cell,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::thread::{self, park_if, Thread};

pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    pub fn send(self, value: T) {
        unsafe { self.inner.send(value) };
    }
}

pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Receiver<T> {
    pub fn recv(self) -> T {
        unsafe { self.inner.recv() }
    }
}

pub fn oneshot<T>() -> (Sender<T>, Receiver<T>) {
    todo!()
}

struct Inner<T> {
    done: AtomicBool,
    value: Cell<Option<T>>,
    thread: Cell<Option<Thread>>,
}

impl<T> Inner<T> {
    pub unsafe fn send(&self, value: T) {
        self.value.set(Some(value));
        self.done.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.unpark();
        }
    }

    pub unsafe fn recv(&self) -> T {
        while !self.done.load(Ordering::Acquire) {
            self.thread.set(Some(thread::current()));
            park_if(|| !self.done.load(Ordering::Acquire));
        }
        self.value.take().unwrap_unchecked()
    }
}
