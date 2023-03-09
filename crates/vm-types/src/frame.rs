use core::{
    fmt::{Debug, Pointer},
    iter::Step,
    marker::PhantomData,
};

use crate::{PageSize, PhysAddr, Size4KiB};

#[repr(transparent)]
pub struct Frame<S = Size4KiB>(PhysAddr, PhantomData<S>);

impl<S> Frame<S>
where
    S: PageSize,
{
    pub const SIZE: usize = S::SIZE;

    pub fn zero() -> Self {
        Self(PhysAddr::from_usize(0), PhantomData)
    }

    pub fn containing(addr: PhysAddr) -> Self {
        let base = addr.align_down(Self::SIZE);
        Self(base, PhantomData)
    }

    pub fn from_base(base: PhysAddr) -> Option<Self> {
        if base.is_aligned(Self::SIZE) {
            Some(Self(base, PhantomData))
        } else {
            None
        }
    }
}

impl<S> Frame<S> {
    pub fn addr(&self) -> PhysAddr {
        self.0
    }
}

impl<S> Copy for Frame<S> {}

impl<S> Clone for Frame<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Debug for Frame<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Frame").field("addr", &self.addr()).finish()
    }
}

impl Pointer for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Frame").field(&self.0).finish()
    }
}

impl<S> PartialEq for Frame<S> {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}

impl<S> Eq for Frame<S> {}

impl<S> PartialOrd for Frame<S> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.addr().partial_cmp(&other.addr())
    }
}

impl<S> Ord for Frame<S> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.addr().cmp(&other.addr())
    }
}

impl<S> Step for Frame<S>
where
    S: PageSize,
{
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        end.addr()
            .as_usize()
            .checked_sub(start.addr().as_usize())
            .map(|off| off / S::SIZE)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        count
            .checked_mul(S::SIZE)
            .and_then(|offset| start.addr().as_usize().checked_add(offset))
            .map(PhysAddr::from_usize)
            .map(|addr| Frame(addr, PhantomData))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        count
            .checked_mul(S::SIZE)
            .and_then(|offset| start.addr().as_usize().checked_sub(offset))
            .map(PhysAddr::from_usize)
            .map(|addr| Frame(addr, PhantomData))
    }

    unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
        let new_base = start.addr().as_usize() + (count * S::SIZE);
        Frame(PhysAddr::from_usize(new_base), PhantomData)
    }

    unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
        let new_base = start.addr().as_usize() + (count * S::SIZE);
        Frame(PhysAddr::from_usize(new_base), PhantomData)
    }
}
