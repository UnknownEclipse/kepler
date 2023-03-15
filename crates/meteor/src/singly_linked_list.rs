use core::{cell::Cell, fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::{DynSinglePtrLink, Node};

pub struct SinglyLinkedList<T>
where
    T: Node<Link>,
{
    pub(crate) inner: UnsafeSinglyLinkedList,
    pub(crate) _p: PhantomData<fn(T)>,
}

impl<T> SinglyLinkedList<T>
where
    T: Node<Link>,
{
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeSinglyLinkedList::new(),
            _p: PhantomData,
        }
    }
}

impl<T> Debug for SinglyLinkedList<T>
where
    T: Node<Link>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SinglyLinkedList").finish_non_exhaustive()
    }
}

impl<T> Default for SinglyLinkedList<T>
where
    T: Node<Link>,
{
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

    pub fn drain_filter<F>(&mut self, f: F) -> DrainFilter<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        todo!()
    }
}

impl<T> Drop for SinglyLinkedList<T>
where
    T: Node<Link>,
{
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[derive(Debug, Default)]
pub struct UnsafeSinglyLinkedList {
    pub(crate) head: Option<NonNull<Link>>,
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

impl<T> IntoIterator for SinglyLinkedList<T>
where
    T: Node<Link>,
{
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

pub struct IntoIter<T>
where
    T: Node<Link>,
{
    list: SinglyLinkedList<T>,
}

impl<T> Iterator for IntoIter<T>
where
    T: Node<Link>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

pub struct DrainFilter<'a, T, F> {
    cur: &'a Cell<Option<NonNull<Link>>>,
    f: F,
    _p: PhantomData<fn(T)>,
}

impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    T: Node<Link>,
    F: FnMut(&T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.cur.get()?;
        let next2 = unsafe { next.as_ref().next.get() };
        let next_node = unsafe { T::from_link(next) };

        if (self.f)(&next_node) {
            self.cur.set(next2);
            Some(next_node)
        } else {
            _ = T::into_link(next_node);
            self.cur = unsafe { &next.as_ref().next };
            None
        }
    }
}

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
