#![no_std]
#![feature(atomic_mut_ptr)]

use core::{cell::Cell, ptr::NonNull};

pub use memoffset::offset_of;

pub mod linked_list;
pub mod linked_stack;
pub mod mpsc_queue;
pub mod singly_linked_list;
pub mod singly_linked_tail_list;
pub mod tail_list;

pub trait Node<Link> {
    fn into_link(node: Self) -> NonNull<Link>;
    /// # Safety
    /// The link pointer must be a pointer returned by the into_link call, and must not
    /// be called more than once per pointer.
    unsafe fn from_link(link: NonNull<Link>) -> Self;
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct DynSinglePtrLink(pub Cell<Option<NonNull<Self>>>);

impl DynSinglePtrLink {
    pub const fn new() -> Self {
        DynSinglePtrLink(Cell::new(None))
    }
}

unsafe impl Sync for DynSinglePtrLink {}
unsafe impl Send for DynSinglePtrLink {}

#[macro_export]
macro_rules! container_of {
    ($ptr:expr, $container:path, $field:ident) => {
        #[allow(clippy::cast_ptr_alignment)]
        {
            ($ptr as *const _ as *const u8).sub($crate::offset_of!($container, $field))
                as *const $container
        }
    };
}

#[macro_export]
macro_rules! container_of_non_null {
    ($ptr:expr, $container:path, $field:ident) => {
        #[allow(clippy::cast_ptr_alignment)]
        {
            ::core::mem::NonNull::new_unchecked(
                ($ptr.cast::<u8>().as_ptr()).sub($crate::offset_of!($container, $field))
                    as *mut $container,
            )
        }
    };
}
