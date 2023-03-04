use core::{
    fmt::Debug,
    marker::PhantomData,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{unsafe_ref::UnsafeRef, Node};

#[derive(Debug, Default)]
pub struct Link {
    next: AtomicPtr<Link>,
}

impl Link {
    pub const fn new() -> Self {
        Self {
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

#[derive(Debug)]
pub struct MpscQueue<T> {
    inner: UnsafeMpscQueue,
    _p: PhantomData<UnsafeRef<T>>,
}

impl<T> MpscQueue<T>
where
    T: Node<Link>,
{
    pub const fn with_static_stub(stub: &'static Link) -> Self {
        let stub: *const Link = stub;
        unsafe { Self::with_stub(NonNull::new_unchecked(stub.cast_mut())) }
    }

    pub const unsafe fn with_stub(stub: NonNull<Link>) -> Self {
        Self {
            inner: unsafe { UnsafeMpscQueue::new(stub) },
            _p: PhantomData,
        }
    }

    pub unsafe fn push(&self, node: NonNull<T>) {
        unsafe {
            let link = T::to_link(node);
            self.inner.push(link);
        }
    }

    pub unsafe fn pop_unsync(&self) -> Option<NonNull<T>> {
        unsafe { self.inner.pop().map(|link| T::from_link(link)) }
    }
}

struct UnsafeMpscQueue {
    head: AtomicPtr<Link>,
    tail: AtomicPtr<Link>,
    stub: NonNull<Link>,
}

impl UnsafeMpscQueue {
    pub const unsafe fn new(stub: NonNull<Link>) -> Self {
        ptr::write(stub.as_ref().next.as_mut_ptr(), ptr::null_mut());

        Self {
            head: AtomicPtr::new(stub.as_ptr()),
            tail: AtomicPtr::new(stub.as_ptr()),
            stub,
        }
    }

    pub unsafe fn push(&self, node: NonNull<Link>) {
        let link = node.as_ref();
        link.next.store_unsync(ptr::null_mut());
        let prev = self.head.swap(node.as_ptr(), Ordering::AcqRel);
        (*prev).next.store(node.as_ptr(), Ordering::Release);
    }

    pub unsafe fn pop(&self) -> Option<NonNull<Link>> {
        let mut tail = self.tail.load(Ordering::Relaxed);
        let mut next = (*tail).next.load(Ordering::Acquire);

        if tail == self.stub.as_ptr() {
            if next.is_null() {
                return None;
            }
            self.tail.store(next, Ordering::Release);
            tail = next;
            next = (*tail).next.load(Ordering::Acquire);
        }

        if !next.is_null() {
            self.tail.store(next, Ordering::Release);
            return NonNull::new(tail);
        }

        let head = self.head.load(Ordering::Acquire);
        if tail != head {
            return None;
        }

        self.push(self.stub);
        next = (*tail).next.load(Ordering::Acquire);

        if !next.is_null() {
            self.tail.store(next, Ordering::Release);
            return NonNull::new(tail);
        }
        None
    }
}

impl Debug for UnsafeMpscQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnsafeMpscQueue").finish_non_exhaustive()
    }
}

trait LoadUnsync {
    type Value;
    unsafe fn load_unsync(&self) -> Self::Value;
}

impl<T> LoadUnsync for AtomicPtr<T> {
    type Value = *mut T;

    unsafe fn load_unsync(&self) -> Self::Value {
        *self.as_mut_ptr()
    }
}

trait StoreUnsync {
    type Value;
    unsafe fn store_unsync(&self, value: Self::Value);
}

impl<T> StoreUnsync for AtomicPtr<T> {
    type Value = *mut T;

    unsafe fn store_unsync(&self, value: Self::Value) {
        *self.as_mut_ptr() = value;
    }
}
