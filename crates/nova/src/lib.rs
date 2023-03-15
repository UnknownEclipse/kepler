#![no_std]
// #![feature(nonnull_slice_from_raw_parts, pointer_is_aligned)]

// extern crate alloc;

// use alloc::boxed::Box;
// use core::{alloc::Layout, num::NonZeroU64};

// use local::Local;

// mod local;
// mod page;
// mod segment;

// pub struct Nova {
//     threads: Box<[Local]>,
//     hooks: &'static dyn SysHooks,
// }

// pub trait SysHooks: 'static + Send + Sync {
//     fn page_size(&self) -> usize;
//     fn alloc(&self, layout: Layout);
// }

// fn sys_hooks() -> &'static dyn SysHooks {
//     &DefaultSysHooks
// }

// struct DefaultSysHooks;

// impl SysHooks for DefaultSysHooks {
//     fn page_size(&self) -> usize {
//         unimplemented!()
//     }

//     fn alloc(&self, layout: Layout) {
//         unimplemented!()
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub struct ThreadId(NonZeroU64);
