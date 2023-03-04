use core::{fmt::Debug, marker::PhantomData, ops::Rem};

use num_traits::Num;

use crate::{align_down, align_up};

#[repr(transparent)]
pub struct Addr<A>(u64, PhantomData<A>)
where
    A: AddrSpace;

impl<A> Addr<A>
where
    A: ~const AddrSpace,
{
    pub fn from_bits(raw: u64) -> Option<Self> {
        Some(Self(A::create(raw)?, PhantomData))
    }

    pub const fn from_bits_truncate(raw: u64) -> Self {
        Self(A::truncate(raw), PhantomData)
    }

    pub const unsafe fn from_bits_unchecked(raw: u64) -> Self {
        Self(raw, PhantomData)
    }

    pub fn zero() -> Self {
        Self(0, PhantomData)
    }

    pub const fn bits(&self) -> u64 {
        self.0
    }

    pub fn checked_add(&self, count: u64) -> Option<Self> {
        self.0.checked_add(count).and_then(Self::from_bits)
    }

    pub fn wrapping_add(&self, count: u64) -> Self {
        Self::from_bits(self.0.wrapping_add(count)).expect("invalid address")
    }

    pub unsafe fn unchecked_add(&self, count: u64) -> Self {
        unsafe { Self::from_bits_unchecked(self.0 + count) }
    }

    pub fn checked_sub(&self, count: u64) -> Option<Self> {
        self.0.checked_sub(count).and_then(Self::from_bits)
    }

    pub fn checked_sub_addr(&self, addr: Self) -> Option<u64> {
        self.0.checked_sub(addr.0)
    }

    pub fn wrapping_sub(&self, count: u64) -> Self {
        Self::from_bits(self.0.wrapping_sub(count)).expect("invalid address")
    }

    pub unsafe fn unchecked_sub(&self, count: u64) -> Self {
        unsafe { Self::from_bits_unchecked(self.0 - count) }
    }

    pub fn is_aligned(&self, align: u64) -> bool {
        self.0.rem(align) == num_traits::zero()
    }

    pub fn align_down(&self, align: u64) -> Self {
        Self(align_down(self.0, align), PhantomData)
    }

    pub fn align_up(&self, align: u64) -> Self {
        Self(align_up(self.0, align), PhantomData)
    }
}

impl<A> PartialEq for Addr<A>
where
    A: AddrSpace,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A> Eq for Addr<A> where A: AddrSpace {}

impl<A> PartialOrd for Addr<A>
where
    A: AddrSpace,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A> Ord for Addr<A>
where
    A: AddrSpace,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A> Debug for Addr<A>
where
    A: AddrSpace,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Addr").field(&self.0).finish()
    }
}

impl<A> Clone for Addr<A>
where
    A: AddrSpace,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for Addr<A> where A: AddrSpace {}

#[const_trait]
pub unsafe trait AddrSpace {
    fn create(raw: u64) -> Option<u64>;
    fn truncate(raw: u64) -> u64;
}

pub trait AddrBits: Num {}

impl AddrBits for u32 {}

impl AddrBits for u64 {}
