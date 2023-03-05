use core::{fmt::Debug, marker::PhantomData};

use crate::VirtAddr;

pub struct Page<S = Size4KiB>(VirtAddr, PhantomData<S>);

impl<S> Page<S>
where
    S: PageSize,
{
    pub const SIZE: usize = S::SIZE;

    pub fn containing(addr: VirtAddr) -> Self {
        let base = addr.align_down(Self::SIZE);
        Self(base, PhantomData)
    }

    pub fn from_base(base: VirtAddr) -> Option<Self> {
        if base.is_aligned(Self::SIZE) {
            Some(Self(base, PhantomData))
        } else {
            None
        }
    }
}

impl<S> Page<S> {
    pub const fn zero() -> Self {
        Self(VirtAddr::zero(), PhantomData)
    }

    pub fn addr(&self) -> VirtAddr {
        self.0
    }
}

impl<S> Copy for Page<S> {}

impl<S> Clone for Page<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Debug for Page<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Page").field("addr", &self.addr()).finish()
    }
}

impl<S> PartialEq for Page<S> {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}

impl<S> Eq for Page<S> {}

impl<S> PartialOrd for Page<S> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.addr().partial_cmp(&other.addr())
    }
}

impl<S> Ord for Page<S> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.addr().cmp(&other.addr())
    }
}

#[derive(Debug)]
pub struct Size4KiB;

pub trait PageSize {
    const SIZE: usize;
}

impl PageSize for Size4KiB {
    const SIZE: usize = 4096;
}
