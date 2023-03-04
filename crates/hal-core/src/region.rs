use core::ops::RangeBounds;

use crate::{
    addr::{Addr, AddrSpace},
    page::{Page, PageSize},
};

pub struct Region<A, S> {
    pub start: Page<A, S>,
    pub end: Page<A, S>,
}

impl<A, S> Region<A, S>
where
    A: ~const AddrSpace,
    S: PageSize,
{
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn len(&self) -> u64 {
        self.end.base().sub_ptr(self.start.base()) / S::SIZE.get() as u64
    }
}

impl<A, S> Region<A, S>
where
    A: ~const AddrSpace,
{
    pub const fn from_address_range<R>(range: R) -> Self
    where
        R: RangeBounds<Addr<A>>,
    {
        todo!()
    }

    pub const fn empty() -> Self {
        Self {
            start: Page::zero(),
            end: Page::zero(),
        }
    }
}

impl<A, S> Default for Region<A, S>
where
    A: AddrSpace,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<A, S> Clone for Region<A, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, S> Copy for Region<A, S> {}
