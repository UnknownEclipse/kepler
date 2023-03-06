use core::{ptr::NonNull, sync::atomic::AtomicPtr};

pub struct Event {}

impl Event {}

#[derive(Debug)]
struct Waiter {
    prev: Option<NonNull<Waiter>>,
}
