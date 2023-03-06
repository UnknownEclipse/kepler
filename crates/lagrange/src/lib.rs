#![no_std]
#![feature(
    atomic_mut_ptr,
    slice_ptr_len,
    new_uninit,
    never_type,
    strict_provenance_atomic_ptr
)]

extern crate alloc;

pub mod atomic_wait;
pub mod mutex;
pub mod scheduler;
pub mod task;
pub mod thread;
