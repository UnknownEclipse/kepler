use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    singly_linked_list::{SinglyLinkedList, UnsafeSinglyLinkedList},
    DynSinglePtrLink, Node,
};

pub struct LinkedStack<T> {
    inner: UnsafeLinkedStack,
    _p: PhantomData<fn(T)>,
}

impl<T> LinkedStack<T> {
    pub fn new() -> Self {
        Self {
            inner: UnsafeLinkedStack::new(),
            _p: PhantomData,
        }
    }
}

impl<T> LinkedStack<T>
where
    T: Node<DynSinglePtrLink>,
{
    pub fn push(&self, node: T) {
        let link = T::into_link(node);
        unsafe { self.inner.push(link.cast()) };
    }

    pub fn pop_all(&self) -> SinglyLinkedList<T> {
        let head = self.inner.pop_all().map(|head| head.cast());
        SinglyLinkedList {
            inner: UnsafeSinglyLinkedList { head },
            _p: PhantomData,
        }
    }
}

pub struct UnsafeLinkedStack {
    head: AtomicPtr<Link>,
}

impl UnsafeLinkedStack {
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub unsafe fn push(&self, link: NonNull<Link>) {
        loop {
            let head = self.head.load(Ordering::Relaxed);
            link.as_ref().next.store(head, Ordering::Relaxed);

            if self
                .head
                .compare_exchange_weak(head, link.as_ptr(), Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    pub fn pop_all(&self) -> Option<NonNull<Link>> {
        let head = self.head.swap(ptr::null_mut(), Ordering::AcqRel);
        NonNull::new(head)
    }
}

#[derive(Debug, Default)]
pub struct Link {
    next: AtomicPtr<Link>,
}
