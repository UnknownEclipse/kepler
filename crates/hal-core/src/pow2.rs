#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pow2Usize(u8);

impl Pow2Usize {
    pub const fn from_log2(v: u32) -> Self {
        Self(v as u8)
    }

    pub const fn get(self) -> usize {
        1 << self.0
    }
}
