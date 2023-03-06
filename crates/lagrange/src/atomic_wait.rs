// use alloc::{boxed::Box, vec::Vec};
// use core::{
//     hash::{BuildHasher, Hash, Hasher},
//     mem,
//     ptr::NonNull,
//     sync::atomic::AtomicU32,
// };

// use ahash::RandomState;
// use spin::{mutex::SpinMutex, Lazy};

// use crate::{
//     task::header::Header,
//     thread::{self, Thread},
// };

// pub fn wait(atomic: &AtomicU32, value: u32) {
//     let key = atomic as *const _ as usize;
//     let queue = PARKING_LOT.queue(key);
//     queue.wait(atomic, value);
// }

// pub fn wake_one(atomic: *const AtomicU32) {
//     let key = atomic as usize;
//     let queue = PARKING_LOT.queue(key);
//     queue.wake_all();
// }

// pub fn wake_all(atomic: *const AtomicU32) {
//     let key = atomic as usize;
//     let queue = PARKING_LOT.queue(key);
//     queue.wake_all();
// }

// static PARKING_LOT: Lazy<ParkingLot> = Lazy::new(ParkingLot::new);

// struct ParkingLot {
//     queues: &'static [WaitQueue],
//     hash_builder: RandomState,
//     mask: u64,
// }

// impl ParkingLot {
//     pub fn new() -> Self {
//         let bucket_count = 64usize;
//         let mask = bucket_count.trailing_zeros().into();

//         let mut queues = Vec::with_capacity(bucket_count);
//         queues.resize_with(bucket_count, WaitQueue::new);
//         let queues = Box::leak(queues.into_boxed_slice());
//         Self {
//             queues,
//             mask,
//             hash_builder: RandomState::new(),
//         }
//     }

//     fn queue(&self, key: usize) -> &WaitQueue {
//         let mut hasher = self.hash_builder.build_hasher();
//         key.hash(&mut hasher);
//         let hash = hasher.finish();
//         let bucket = hash & self.mask;
//         &self.queues[bucket as usize]
//     }
// }

// struct Waiter {
//     next: Option<NonNull<Waiter>>,
//     prev: Option<NonNull<Waiter>>,
//     thread: Thread,
//     key: usize,
// }

// #[derive(Debug)]
// struct WaitQueue {
//     list: SpinMutex<WaitList>,
// }

// impl WaitQueue {
//     fn new() -> Self {
//         todo!()
//     }

//     fn wake_all(&self) {
//         let mut list = self.list.lock().take();
//         list.wake_all();
//     }

//     fn wake_one(&self, key: usize) -> bool {
//         self.list.lock().wake_one(key)
//     }

//     fn wait(&self, atomic: &AtomicU32, value: u32) {
//         let mut waiter = Waiter {
//             key: atomic as *const AtomicU32 as usize,
//             thread: thread::current(),
//             next: None,
//             prev: None,
//         };
//         let mut list = self.list.lock();
//         if atomic.load(core::sync::atomic::Ordering::Relaxed) == value {
//             return;
//         }
//         list.push(NonNull::from(&mut waiter));
//         mem::drop(list);

//     }
// }

// unsafe impl Send for WaitQueue {}
// unsafe impl Sync for WaitQueue {}

// #[derive(Debug, Default)]
// struct WaitList {
//     head: Option<NonNull<Waiter>>,
//     tail: Option<NonNull<Waiter>>,
// }

// impl WaitList {
//     pub fn take(&mut self) -> Self {
//         Self {
//             head: self.head.take(),
//             tail: self.tail.take(),
//         }
//     }

//     pub fn push(&mut self, waiter: NonNull<Waiter>) {
//         if let Some(tail) = self.tail {
//             unsafe {
//                 waiter.as_mut().prev = Some(tail);
//                 tail.as_mut().next = Some(waiter);
//             }
//         } else {
//             self.head = Some(waiter);
//             self.tail = Some(waiter);
//         }
//     }

//     pub fn wake_one(&mut self, key: usize) -> bool {
//         todo!("proper impl");

//         let mut prev = self.head;
//         let mut cur = self.head;

//         while let Some(node) = cur {
//             let node = unsafe { node.as_ref() };

//             if node.key == key {
//                 let next = node.next;
//                 if let Some(mut prev) = prev {
//                     unsafe {
//                         prev.as_mut().next = next;
//                     }
//                 } else {
//                     self.head = next;
//                 }
//                 return true;
//             }

//             prev = cur;
//             cur = node.next;
//         }
//         false
//     }

//     pub fn wake_all(&mut self) {
//         let mut node = self.head;
//         while let Some(current) = node {}
//     }
// }
