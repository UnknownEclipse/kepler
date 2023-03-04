#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(*const u8);

impl VirtAddr {
    pub const fn from_ptr<T>(ptr: *const T) -> VirtAddr {
        VirtAddr(ptr.cast())
    }

    pub fn bits(&self) -> usize {
        self.0 as usize
    }

    pub unsafe fn unchecked_add(&self, count: usize) -> VirtAddr {
        Self(self.0.add(count))
    }

    pub unsafe fn unchecked_sub(&self, count: usize) -> VirtAddr {
        Self(self.0.sub(count))
    }

    pub unsafe fn unchecked_sub_addr(&self, other: VirtAddr) -> usize {
        self.0.sub_ptr(other.0)
    }

    pub const fn as_ptr<T>(&self) -> *mut T {
        self.0.cast_mut().cast()
    }
}

unsafe impl Send for VirtAddr {}
unsafe impl Sync for VirtAddr {}
