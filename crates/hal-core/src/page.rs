use core::{fmt::Debug, marker::PhantomData};

use self::private::Sealed;
use crate::{
    addr::{Addr, AddrSpace},
    pow2::Pow2Usize,
};

pub struct Page<A, S>(Addr<A>, PhantomData<S>);

impl<A, S> Page<A, S>
where
    A: ~const AddrSpace,
{
    pub const fn zero() -> Page<A, S> {
        let addr = Addr::new(0).unwrap();
        Page(addr, PhantomData)
    }

    pub const fn base(&self) -> Addr<A> {
        self.0
    }
}

impl<A, S> Clone for Page<A, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, S> Copy for Page<A, S> {}

impl<A, S> Debug for Page<A, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Page").field(&self.0).field(&self.1).finish()
    }
}

impl<A, S> PartialEq for Page<A, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A, S> Eq for Page<A, S> {}

impl<A, S> PartialOrd for Page<A, S> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A, S> Ord for Page<A, S> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
pub trait PageSize: Sealed {
    const SIZE: Pow2Usize;
}

#[derive(Debug)]
pub struct Size4KiB;

impl Sealed for Size4KiB {}

impl PageSize for Size4KiB {
    const SIZE: Pow2Usize = Pow2Usize::from_log2(12);
}

mod private {
    pub trait Sealed {}
}
