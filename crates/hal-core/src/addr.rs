use core::{fmt::Debug, marker::PhantomData};

pub struct Addr<A>(u64, PhantomData<A>);

impl<A> Addr<A>
where
    A: ~const AddrSpace,
{
    pub const fn new(addr: u64) -> Option<Self> {
        if A::is_valid(addr) {
            Some(Self(addr, PhantomData))
        } else {
            None
        }
    }

    pub const unsafe fn new_unchecked(addr: u64) -> Self {
        Self(addr, PhantomData)
    }

    pub const fn new_truncate(addr: u64) -> Self {
        Self(A::truncate(addr), PhantomData)
    }

    pub const fn to_u64(&self) -> u64 {
        self.0
    }

    pub const fn checked_add(&self, count: u64) -> Option<Self> {
        self.0.checked_add(count).and_then(Self::new)
    }

    pub const fn checked_sub(&self, count: u64) -> Option<Self> {
        self.0.checked_sub(count).and_then(Self::new)
    }

    pub const fn sub_ptr(&self, other: Self) -> u64 {
        self.0 - other.0
    }

    pub const fn offset_from(&self, base: Self) -> i64 {
        let a = self.0;
        let b = base.0;
        if a >= b {
            (a - b) as i64
        } else {
            -((b - a) as i64)
        }
    }

    pub const fn checked_sub_ptr(&self, other: Self) -> Option<u64> {
        self.0.checked_sub(other.0)
    }
}

impl<A> Clone for Addr<A> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A> Copy for Addr<A> {}

impl<A> Debug for Addr<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Addr").field(&self.0).field(&self.1).finish()
    }
}

impl<A> PartialEq for Addr<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A> Eq for Addr<A> {}

impl<A> PartialOrd for Addr<A> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<A> Ord for Addr<A> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[const_trait]
pub trait AddrSpace {
    fn is_valid(addr: u64) -> bool;
    fn truncate(addr: u64) -> u64;
}
