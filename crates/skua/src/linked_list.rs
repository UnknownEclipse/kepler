use core::{cell::Cell, marker::PhantomData, mem, ptr::NonNull};

use crate::{unsafe_ref::UnsafeRef, Node};

#[derive(Debug, Default)]
pub struct Link {
    next: Cell<Option<NonNull<Self>>>,
    prev: Cell<Option<NonNull<Self>>>,
}

impl Link {
    pub const fn new() -> Self {
        Self {
            next: Cell::new(None),
            prev: Cell::new(None),
        }
    }
}

#[derive(Debug)]
pub struct LinkedList<T> {
    raw: RawList,
    _p: PhantomData<UnsafeRef<T>>,
}

impl<T> LinkedList<T> {
    pub const fn new() -> Self {
        Self {
            raw: RawList::new(),
            _p: PhantomData,
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LinkedList<T>
where
    T: Node<Link>,
{
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }

    pub fn pop_back(&mut self) -> Option<NonNull<T>> {
        todo!()
    }

    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        todo!()
    }

    pub unsafe fn push_back(&mut self, node: NonNull<T>) {
        let link = T::to_link(node);
        self.raw.push_back(link);
    }

    pub unsafe fn push_front(&mut self, node: NonNull<T>) {
        let link = T::to_link(node);
        self.raw.push_front(link);
    }

    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            raw: self.raw.cursor(),
            _p: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct CursorMut<'a, T> {
    raw: RawCursor<'a>,
    _p: PhantomData<&'a mut LinkedList<T>>,
}

impl<'a, T> CursorMut<'a, T>
where
    T: Node<Link>,
{
    pub fn get(&self) -> Option<NonNull<T>> {
        self.raw.get().map(|link| unsafe { T::from_link(link) })
    }

    pub fn remove(&mut self) -> Option<NonNull<T>> {
        let node = self.get();
        unsafe { self.raw.remove() };
        node
    }

    pub fn move_next(&mut self) {
        unsafe { self.raw.move_next() };
    }

    pub fn move_prev(&mut self) {
        unsafe { self.raw.move_prev() };
    }
}

#[derive(Debug, Default)]
struct RawList {
    head: Option<NonNull<Link>>,
    tail: Option<NonNull<Link>>,
}

impl RawList {
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub unsafe fn push_back(&mut self, node: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn push_front(&mut self, node: NonNull<Link>) {
        todo!()
    }

    pub fn cursor(&mut self) -> RawCursor<'_> {
        let node = self.head;
        let list = self;
        RawCursor { list, node }
    }
}

#[derive(Debug)]
struct RawCursor<'a> {
    list: &'a mut RawList,
    node: Option<NonNull<Link>>,
}

impl<'a> RawCursor<'a> {
    pub unsafe fn move_next(&mut self) {
        self.node = self.node.and_then(|node| node.as_ref().next.get());
    }

    pub unsafe fn move_prev(&mut self) {
        self.node = self.node.and_then(|node| node.as_ref().prev.get());
    }

    pub fn get(&self) -> Option<NonNull<Link>> {
        self.node
    }

    pub unsafe fn remove(&mut self) {
        let Some(node) = self.node else {
            return;
        };
        let node = node.as_ref();

        let next = node.next.get();
        let prev = node.prev.get();

        if let Some(next) = next {
            next.as_ref().prev.set(prev);
        } else {
            self.list.tail = prev;
        }

        if let Some(prev) = prev {
            prev.as_ref().next.set(next);
        } else {
            self.list.head = next;
        }

        self.node = prev;
    }
}
