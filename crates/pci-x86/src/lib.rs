#![no_std]

use core::ops::Shl;

use hal_x86_64::port::Port;
use pci_types::{ConfigSpace, PciAddr};

const CONFIG_ADDRESS: Port<u32> = Port::new(0xcf8);
const CONFIG_DATA: Port<u32> = Port::new(0xcfc);

#[derive(Debug)]
pub struct IoPortConfigSpace;

unsafe impl ConfigSpace for IoPortConfigSpace {
    fn exists(&self, addr: PciAddr) -> bool {
        let v = unsafe { self.read(addr, 0) };
        (v & 0xffff) != 0xffff
    }

    unsafe fn read(&self, addr: PciAddr, offset: u16) -> u32 {
        set_address(addr, offset);
        CONFIG_DATA.read()
    }

    unsafe fn write(&self, addr: PciAddr, offset: u16, value: u32) {
        set_address(addr, offset);
        CONFIG_DATA.write(value);
    }
}

unsafe fn set_address(addr: PciAddr, offset: u16) {
    const ENABLE_BIT: u32 = 1 << 31;

    let addr = u32::from(addr.bus()).shl(16)
        | u32::from(addr.device()).shl(11)
        | u32::from(addr.function()).shl(8)
        | u32::from(offset)
        | ENABLE_BIT;

    CONFIG_ADDRESS.write(addr);
}
