use core::fmt::Pointer;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(usize);

impl PhysAddr {
    #[inline]
    pub fn from_usize(v: usize) -> Self {
        Self(v)
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self.0
    }

    #[inline]
    pub fn align_up(&self, align: usize) -> Self {
        Self(crate::align_up(self.0 as u64, align as u64) as usize)
    }

    #[inline]
    pub fn align_down(&self, align: usize) -> Self {
        Self(crate::align_down(self.0 as u64, align as u64) as usize)
    }

    #[inline]
    pub fn is_aligned(&self, align: usize) -> bool {
        assert!(align.is_power_of_two());
        self.0 % align == 0
    }
}

impl Pointer for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("PhysAddr").field(&self.0).finish()
    }
}
