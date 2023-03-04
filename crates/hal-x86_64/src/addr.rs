use hal_core::addr::{Addr, AddrSpace};

pub type VirtAddr = Addr<Virt>;
pub type PhysAddr = Addr<Phys>;

#[derive(Debug)]
pub struct Virt;

impl const AddrSpace for Virt {
    fn is_valid(addr: u64) -> bool {
        matches!(addr.wrapping_shr(47), 0 | 0x1ffff)
    }

    fn truncate(addr: u64) -> u64 {
        let addr = addr & !(0xffff << 48);
        let high = addr & (1 << 47) != 0;
        if high {
            addr | (0xffff << 48)
        } else {
            addr
        }
    }
}

#[derive(Debug)]
pub struct Phys;

impl const AddrSpace for Phys {
    fn is_valid(_: u64) -> bool {
        true
    }

    fn truncate(addr: u64) -> u64 {
        addr
    }
}
