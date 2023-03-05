#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr<T = usize>(T);

macro_rules! define_phys_addr_ops {
    ($t:ty) => {
        impl PhysAddr<$t> {
            #[inline]
            pub fn new(v: $t) -> Self {
                Self(v)
            }

            #[inline]
            pub fn align_up(&self, align: usize) -> Self {
                Self(crate::align_up(self.0 as u64, align as u64) as $t)
            }

            #[inline]
            pub fn align_down(&self, align: usize) -> Self {
                Self(crate::align_down(self.0 as u64, align as u64) as $t)
            }

            #[inline]
            pub fn is_aligned(&self, align: usize) -> bool {
                assert!(align.is_power_of_two());
                self.0 % (align as $t) != 0
            }
        }
    };
}

impl PhysAddr<u32> {
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl PhysAddr<usize> {
    #[inline]
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl PhysAddr<u64> {
    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

define_phys_addr_ops!(u32);
define_phys_addr_ops!(usize);
define_phys_addr_ops!(u64);
