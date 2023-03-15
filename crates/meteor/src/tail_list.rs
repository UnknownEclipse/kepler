use core::{cell::Cell, fmt::Debug, marker::PhantomData, ptr::NonNull};

use crate::{DynSinglePtrLink, Node};

/// An unbounded intrusive queue.
pub struct TailList<T> {
    pub(crate) inner: UnsafeTailList,
    _p: PhantomData<fn(T)>,
}

impl<T> TailList<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: UnsafeTailList::new(),
            _p: PhantomData,
        }
    }

    pub unsafe fn from_unsafe(list: UnsafeTailList) -> Self {
        Self {
            inner: list,
            _p: PhantomData,
        }
    }
}

impl<T> Debug for TailList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TailQueue").finish_non_exhaustive()
    }
}

impl<T> Default for TailList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> TailList<T>
where
    T: Node<Link>,
{
    #[inline]
    pub fn push_back(&mut self, node: T) {
        let link = T::into_link(node);
        unsafe { self.inner.push_back(link) };
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
        DrainFilter {
            cur: Cell::from_mut(&mut self.inner.head),
            tail: Cell::from_mut(&mut self.inner.tail),
            f,
            _p: PhantomData,
        }
    }
}

pub struct DrainFilter<'a, T, F> {
    cur: &'a Cell<Option<NonNull<Link>>>,
    tail: &'a Cell<Option<NonNull<Link>>>,
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
        loop {
            let next = self.cur.get()?;
            let next2 = unsafe { next.as_ref().next.get() };
            let next_node = unsafe { T::from_link(next) };

            if (self.f)(&next_node) {
                if next2.is_none() {
                    let ptr = NonNull::from(self.cur);
                    self.tail.set(Some(ptr.cast()));
                }

                self.cur.set(next2);
                return Some(next_node);
            } else {
                _ = T::into_link(next_node);
                self.cur = unsafe { &next.as_ref().next };
                continue;
            }
        }
    }
}
#[derive(Debug, Default)]
pub struct UnsafeTailList {
    pub(crate) head: Option<NonNull<Link>>,
    pub(crate) tail: Option<NonNull<Link>>,
}

impl UnsafeTailList {
    #[inline]
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub unsafe fn push_back(&mut self, link: NonNull<Link>) {
        link.as_ref().next.set(None);

        if let Some(tail) = self.tail {
            tail.as_ref().next.set(Some(link));
            self.tail = Some(link);
        } else {
            self.head = Some(link);
            self.tail = Some(link);
        }
    }

    pub unsafe fn pop_front(&mut self) -> Option<NonNull<Link>> {
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

unsafe impl Sync for UnsafeTailList {}
unsafe impl Send for UnsafeTailList {}

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
