use core::{fmt::Debug, iter::Step, marker::PhantomData, ops::Range};

use crate::{
    addr::{Addr, AddrSpace},
    align_down,
};

pub struct Page<A, S>(Addr<A>, PhantomData<S>)
where
    A: AddrSpace,
    S: PageSize;

impl<A, S> Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    pub fn from_base(base: Addr<A>) -> Option<Self> {
        let align = S::SIZE.try_into().unwrap();
        if base.is_aligned(align) {
            Some(Self(base, PhantomData))
        } else {
            None
        }
    }

    pub fn containing(addr: Addr<A>) -> Self {
        Self(addr.align_down(S::SIZE as u64), PhantomData)
    }

    pub const unsafe fn from_base_unchecked(base: Addr<A>) -> Self {
        Self(base, PhantomData)
    }

    pub fn zero() -> Self {
        Self(Addr::zero(), PhantomData)
    }

    pub fn base(&self) -> Addr<A> {
        self.0
    }

    pub fn contains(&self, addr: Addr<A>) -> bool {
        self.as_addr_range().contains(&addr)
    }

    pub fn as_addr_range(&self) -> Range<Addr<A>> {
        todo!()
    }
}

impl<A, S> Debug for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Page")
            .field("base", &self.0)
            .field("size", &S::SIZE)
            .finish()
    }
}

impl<A, S> Copy for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
}

impl<A, S> Clone for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, S> PartialEq for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A, S> Eq for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
}

impl<A, S> PartialOrd for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A, S> Ord for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A, S> Step for Page<A, S>
where
    A: AddrSpace,
    S: PageSize,
{
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        end.base()
            .checked_sub_addr(start.base())
            .and_then(|v| v.try_into().ok())
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        let count = count as u64;
        let page_size = S::SIZE as u64;
        let offset = count.checked_mul(page_size)?;
        let new_base = start.base().checked_add(offset)?;
        Page::from_base(new_base)
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        let count = count as u64;
        let page_size = S::SIZE as u64;
        let offset = count.checked_mul(page_size)?;
        let new_base = start.base().checked_sub(offset)?;
        Page::from_base(new_base)
    }

    fn forward(start: Self, count: usize) -> Self {
        Step::forward_checked(start, count).expect("overflow in `Step::forward`")
    }

    unsafe fn forward_unchecked(start: Self, count: usize) -> Self {
        let count = count as u64;
        let page_size = S::SIZE as u64;
        let offset = count * page_size;
        Page::from_base_unchecked(start.base().unchecked_add(offset))
    }

    fn backward(start: Self, count: usize) -> Self {
        Step::backward_checked(start, count).expect("overflow in `Step::backward`")
    }

    unsafe fn backward_unchecked(start: Self, count: usize) -> Self {
        let count = count as u64;
        let page_size = S::SIZE as u64;
        let offset = count * page_size;
        Page::from_base_unchecked(start.base().unchecked_sub(offset))
    }
}

pub trait PageSize {
    const SIZE: usize;
}
