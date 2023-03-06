#![no_std]
#![feature(atomic_mut_ptr, const_ptr_as_ref, const_ptr_write)]

use core::ptr::NonNull;

pub mod linked_list;
pub mod mpsc_queue;
pub mod unsafe_ref;

pub trait Node<Link> {
    unsafe fn to_link(node: NonNull<Self>) -> NonNull<Link>;
    unsafe fn from_link(link: NonNull<Link>) -> NonNull<Self>;
}
