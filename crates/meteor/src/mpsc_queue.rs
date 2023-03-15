use core::{
    fmt::Debug,
    marker::PhantomData,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    tail_list::{TailList, UnsafeTailList},
    DynSinglePtrLink, Node,
};

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
    _p: PhantomData<fn(T)>,
}

impl<T> MpscQueue<T>
where
    T: Node<Link>,
{
    pub fn with_static_stub(stub: &'static Link) -> Self {
        let stub: *const Link = stub;
        unsafe { Self::with_stub(NonNull::new_unchecked(stub.cast_mut())) }
    }

    /// # Safety
    /// 1. The pointer must remain valid for the lifetime of the queue.
    /// 2. No other queue may use the same stub.
    pub unsafe fn with_stub(stub: NonNull<Link>) -> Self {
        Self {
            inner: unsafe { UnsafeMpscQueue::new(stub) },
            _p: PhantomData,
        }
    }

    pub fn push(&self, node: T) {
        unsafe {
            let link = T::into_link(node);
            self.inner.push(link);
        }
    }

    /// # Safety
    /// This may only be called from a single thread at a time.
    pub unsafe fn pop_unsync(&self) -> Option<T> {
        unsafe { self.inner.pop().map(|link| T::from_link(link)) }
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl<T> MpscQueue<T>
where
    T: Node<DynSinglePtrLink>,
{
    pub unsafe fn take(&self) -> TailList<T> {
        let inner = self.inner.take();
        TailList::from_unsafe(inner)
    }
}

struct UnsafeMpscQueue {
    head: AtomicPtr<Link>,
    tail: AtomicPtr<Link>,
    stub: NonNull<Link>,
}

impl UnsafeMpscQueue {
    pub unsafe fn new(stub: NonNull<Link>) -> Self {
        stub.as_ref().next.store(ptr::null_mut(), Ordering::Relaxed);

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

    pub fn is_empty(&self) -> bool {
        unsafe {
            let tail = self.tail.load(Ordering::Relaxed);
            let next = (*tail).next.load(Ordering::Acquire);

            tail == self.stub.as_ptr() && next.is_null()
        }
    }

    pub unsafe fn take(&self) -> UnsafeTailList {
        let head = self.head.swap(self.stub.as_ptr(), Ordering::Relaxed);
        let tail = self.tail.swap(self.stub.as_ptr(), Ordering::Relaxed);

        UnsafeTailList {
            head: NonNull::new(head.cast()),
            tail: NonNull::new(tail.cast()),
        }
    }
}

unsafe impl Send for UnsafeMpscQueue {}
unsafe impl Sync for UnsafeMpscQueue {}

impl Debug for UnsafeMpscQueue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("UnsafeMpscQueue").finish_non_exhaustive()
    }
}

impl<T> Node<Link> for T
where
    T: Node<DynSinglePtrLink>,
{
    fn into_link(node: Self) -> NonNull<Link> {
        T::into_link(node).cast()
    }

    unsafe fn from_link(link: NonNull<Link>) -> Self {
        T::from_link(link.cast())
    }
}
trait LoadUnsync {
    type Value;
    unsafe fn load_unsync(&self) -> Self::Value;
}

impl<T> LoadUnsync for AtomicPtr<T> {
    type Value = *mut T;

    unsafe fn load_unsync(&self) -> Self::Value {
        *self.as_ptr()
    }
}

trait StoreUnsync {
    type Value;
    unsafe fn store_unsync(&self, value: Self::Value);
}

impl<T> StoreUnsync for AtomicPtr<T> {
    type Value = *mut T;

    unsafe fn store_unsync(&self, value: Self::Value) {
        *self.as_ptr() = value;
    }
}
