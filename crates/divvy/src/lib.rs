#![no_std]
#![feature(
    allocator_api,
    const_maybe_uninit_uninit_array,
    maybe_uninit_uninit_array,
    nonnull_slice_from_raw_parts,
    slice_ptr_get,
    sync_unsafe_cell
)]

pub mod bump;
pub mod global;
pub mod hybrid;
pub mod nop;
