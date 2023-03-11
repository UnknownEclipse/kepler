use core::{cell::Cell, fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::{DynSinglePtrLink, Node};

pub struct SinglyLinkedList<T> {
    inner: UnsafeSinglyLinkedList,
    f: PhantomData<fn(T)>,
}

impl<T> SinglyLinkedList<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeSinglyLinkedList::new(),
            f: PhantomData,
        }
    }
}

impl<T> Debug for SinglyLinkedList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SinglyLinkedList").finish_non_exhaustive()
    }
}

impl<T> Default for SinglyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SinglyLinkedList<T>
where
    T: Node<Link>,
{
    #[inline]
    pub fn push_front(&mut self, node: T) {
        let link = T::into_link(node);
        unsafe { self.inner.push_front(link) };
    }

    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            let link = self.inner.pop_front()?;
            Some(T::from_link(link))
        }
    }
}

#[derive(Debug, Default)]
pub struct UnsafeSinglyLinkedList {
    head: Option<NonNull<Link>>,
}

impl UnsafeSinglyLinkedList {
    #[inline]
    pub const fn new() -> Self {
        Self { head: None }
    }

    pub unsafe fn push_front(&mut self, link: NonNull<Link>) {
        link.as_ref().next.set(self.head);
        self.head = Some(link);
    }

    pub unsafe fn pop_front(&mut self) -> Option<NonNull<Link>> {
        let head = self.head?;
        self.head = head.as_ref().next.get();
        Some(head)
    }
}

unsafe impl Sync for UnsafeSinglyLinkedList {}
unsafe impl Send for UnsafeSinglyLinkedList {}

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
