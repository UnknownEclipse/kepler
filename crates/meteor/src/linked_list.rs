use core::{cell::Cell, ptr::NonNull};

pub struct UnsafeLinkedList {
    head: Option<NonNull<Link>>,
    tail: Option<NonNull<Link>>,
}

impl UnsafeLinkedList {
    pub unsafe fn pop_front(&mut self) -> Option<NonNull<Link>> {
        todo!()
    }

    pub unsafe fn pop_back(&mut self) -> Option<NonNull<Link>> {
        todo!()
    }

    pub unsafe fn push_front(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn push_back(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn remove(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn remove_next(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn remove_prev(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn insert_after(&mut self, link: NonNull<Link>) {
        todo!()
    }

    pub unsafe fn insert_before(&mut self, link: NonNull<Link>) {
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct Link {
    next: Cell<Option<NonNull<Link>>>,
    prev: Cell<Option<NonNull<Link>>>,
}

impl Link {
    pub const fn new() -> Self {
        Self {
            next: Cell::new(None),
            prev: Cell::new(None),
        }
    }
}
