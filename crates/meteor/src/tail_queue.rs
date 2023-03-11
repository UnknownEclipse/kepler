use core::{cell::Cell, fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::{DynSinglePtrLink, Node};

pub struct TailQueue<T> {
    inner: UnsafeTailQueue,
    f: PhantomData<fn(T)>,
}

impl<T> TailQueue<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeTailQueue::new(),
            f: PhantomData,
        }
    }
}

impl<T> Debug for TailQueue<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TailQueue").finish_non_exhaustive()
    }
}

impl<T> Default for TailQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TailQueue<T>
where
    T: Node<Link>,
{
    #[inline]
    pub fn push(&mut self, node: T) {
        let link = T::into_link(node);
        unsafe { self.inner.push(link) };
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            let link = self.inner.pop()?;
            Some(T::from_link(link))
        }
    }
}

#[derive(Debug, Default)]
pub struct UnsafeTailQueue {
    head: Option<NonNull<Link>>,
    tail: Option<NonNull<Link>>,
}

impl UnsafeTailQueue {
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub unsafe fn push(&mut self, link: NonNull<Link>) {
        link.as_ref().next.set(None);

        if let Some(tail) = self.tail {
            tail.as_ref().next.set(Some(link));
            self.tail = Some(link);
        } else {
            self.head = Some(link);
            self.tail = Some(link);
        }
    }

    pub unsafe fn pop(&mut self) -> Option<NonNull<Link>> {
        let head = self.head?;
        if let Some(next) = head.as_ref().next.get() {
            self.head = Some(next);
        } else {
            self.head = None;
            self.tail = None;
        }
        Some(head)
    }
}

unsafe impl Sync for UnsafeTailQueue {}
unsafe impl Send for UnsafeTailQueue {}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Link {
    next: Cell<Option<NonNull<Link>>>,
}

impl Link {
    #[inline]
    pub const fn new() -> Self {
        Link {
            next: Cell::new(None),
        }
    }
}

unsafe impl Sync for Link {}
unsafe impl Send for Link {}

impl<T> Node<Link> for T
where
    T: Node<DynSinglePtrLink>,
{
    #[inline]
    fn into_link(node: Self) -> NonNull<Link> {
        Node::<DynSinglePtrLink>::into_link(node).cast()
    }

    #[inline]
    unsafe fn from_link(link: NonNull<Link>) -> Self {
        T::from_link(link.cast())
    }
}
