use core::num::NonZeroU8;

use vm_types::VirtAddr;

#[repr(C, packed(4))]
#[derive(Debug)]
struct Tss {
    _reserved1: u32,
    rsp: [VirtAddr; 3],
    _reserved2: [u32; 2],
    ist: [VirtAddr; 7],
    _reserved3: [u32; 2],
    _reserved4: u16,
    iopb: u16,
}

impl Tss {
    pub fn ist(&self, index: IstIndex) -> VirtAddr {
        self.ist[index.index()]
    }

    pub fn set_ist(&mut self, index: IstIndex, addr: VirtAddr) {
        self.ist[index.index()] = addr;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IstIndex(pub NonZeroU8);

impl IstIndex {
    fn index(&self) -> usize {
        usize::from(self.0.get() - 1)
    }
}
