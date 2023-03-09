use core::fmt::Pointer;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(*const ());

impl VirtAddr {
    #[inline]
    pub const fn zero() -> Self {
        Self(0 as *const ())
    }

    #[inline]
    pub const fn from_usize(v: usize) -> Self {
        Self(v as *const ())
    }

    #[inline]
    pub const fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr.cast())
    }

    #[inline]
    pub const fn as_ptr<T>(&self) -> *mut T {
        self.0.cast_mut().cast()
    }

    #[inline]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn align_up(&self, align: usize) -> Self {
        Self(crate::align_up(self.as_usize() as u64, align as u64) as usize as *mut ())
    }

    #[inline]
    pub fn align_down(&self, align: usize) -> Self {
        Self(crate::align_down(self.as_usize() as u64, align as u64) as usize as *mut ())
    }

    #[inline]
    pub fn is_aligned(&self, align: usize) -> bool {
        assert!(align.is_power_of_two());
        self.as_usize() % align == 0
    }
}

impl Pointer for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtAddr").field(&self.0).finish()
    }
}

unsafe impl Send for VirtAddr {}
unsafe impl Sync for VirtAddr {}
