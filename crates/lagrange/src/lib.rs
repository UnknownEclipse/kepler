#![no_std]
#![feature(
    atomic_mut_ptr,
    drain_filter,
    slice_ptr_len,
    new_uninit,
    never_type,
    strict_provenance_atomic_ptr,
    strict_provenance
)]

extern crate alloc;

pub mod futex;
mod scheduler;
pub mod sync;
mod task;
pub mod thread;
