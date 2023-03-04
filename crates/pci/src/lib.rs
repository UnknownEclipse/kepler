#![no_std]

use core::{fmt::Debug, ops::BitAnd};

pub use pci_types as types;
use pci_types::{ClassId, ConfigSpace, DeviceId, HeaderType, PciAddr, SubclassId, VendorId};
use pci_x86::IoPortConfigSpace;
use types::Bar;

#[derive(Debug, Clone, Copy)]
pub struct DefaultConfigSpace;

unsafe impl ConfigSpace for DefaultConfigSpace {
    fn exists(&self, addr: PciAddr) -> bool {
        IoPortConfigSpace.exists(addr)
    }

    unsafe fn read(&self, addr: PciAddr, offset: u16) -> u32 {
        IoPortConfigSpace.read(addr, offset)
    }

    unsafe fn write(&self, addr: PciAddr, offset: u16, value: u32) {
        IoPortConfigSpace.write(addr, offset, value)
    }
}

pub fn enumerate<C>(config_space: C) -> EnumerateDevices<C>
where
    C: ConfigSpace + Clone,
{
    EnumerateDevices {
        inner: RawDeviceAddresses {
            config_space,
            device_functions: None,
            packed: 0,
        },
    }
}

#[derive(Debug)]
pub struct EnumerateDevices<C> {
    inner: RawDeviceAddresses<C>,
}

impl<C> Iterator for EnumerateDevices<C>
where
    C: ConfigSpace + Clone,
{
    type Item = PciDevice<C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|addr| PciDevice {
            addr,
            config_space: self.inner.config_space.clone(),
        })
    }
}

pub struct PciDevice<C> {
    addr: PciAddr,
    config_space: C,
}

impl<C> Debug for PciDevice<C>
where
    C: Debug + ConfigSpace,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PciDevice")
            .field("addr", &self.addr)
            .field("config_space", &self.config_space)
            .field("vendor_id", &self.id().0)
            .field("device_id", &self.id().1)
            .field("class_id", &self.class_id().0)
            .field("subclass_id", &self.class_id().1)
            .finish()
    }
}

impl<C> PciDevice<C>
where
    C: ConfigSpace,
{
    pub fn id(&self) -> (VendorId, DeviceId) {
        let reg = unsafe { self.config_space.read(self.addr, 0) };
        let vendor = reg as u16;
        let device = reg.wrapping_shr(16) as u16;
        (VendorId(vendor), DeviceId(device))
    }

    pub fn class_id(&self) -> (ClassId, SubclassId) {
        let reg = unsafe { self.config_space.read(self.addr, 0x8) };
        let bytes = reg.to_be_bytes();
        (ClassId(bytes[0]), SubclassId(bytes[1]))
    }

    pub fn bar0(&self) -> u32 {
        unsafe { self.config_space.read(self.addr, 0x10) }
    }

    pub fn read(&self, offset: u16) -> u32 {
        unsafe { self.config_space.read(self.addr, offset) }
    }

    unsafe fn write(&self, offset: u16, value: u32) {
        self.config_space.write(self.addr, offset, value)
    }

    unsafe fn bar(&self, index: u16) -> Bar {
        let offset = 0x10 + index * 4;

        let bar = self.read(offset);
        let kind = bar.wrapping_shr(1).bitand(0b11);

        match kind {
            0 => {
                self.write(offset, u32::MAX);
                let temp = self.read(offset);
                self.write(offset, bar);

                let size = !(temp & 0xfffffff0) + 1;

                let addr = bar & 0xfffffff0;
                let prefetchable = bar & (1 << 3) != 0;

                Bar::Memory32 {
                    addr,
                    size,
                    prefetchable,
                }
            }
            2 => {
                todo!()
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct RawDeviceAddresses<C> {
    packed: u16,
    device_functions: Option<EnumerateCurrentDeviceFunctions<C>>,
    config_space: C,
}

const LIMIT: u16 = (1 << 13) - 1;

impl<C> Iterator for RawDeviceAddresses<C>
where
    C: ConfigSpace + Clone,
{
    type Item = PciAddr;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(device) = &mut self.device_functions {
                if let Some(addr) = device.next() {
                    return Some(addr);
                }
            }
            if LIMIT < self.packed {
                return None;
            }
            self.device_functions = Some(EnumerateCurrentDeviceFunctions {
                bus: self.packed.wrapping_shr(5) as u8,
                device: self.packed.bitand(0b11111) as u8,
                function: 0,
                config_space: self.config_space.clone(),
            });
            self.packed += 1;
        }
    }
}

#[derive(Debug)]
struct EnumerateCurrentDeviceFunctions<C> {
    bus: u8,
    device: u8,
    function: u8,
    config_space: C,
}

impl<C> Iterator for EnumerateCurrentDeviceFunctions<C>
where
    C: ConfigSpace,
{
    type Item = PciAddr;

    fn next(&mut self) -> Option<Self::Item> {
        if 8 <= self.function {
            return None;
        }

        let mut addr = PciAddr::new(0, self.bus, self.device, self.function);
        if self.function == 0 {
            if !self.config_space.exists(addr) {
                self.function = u8::MAX;
                return None;
            }
            let header_type = unsafe { read_header_type(addr, &self.config_space) };
            if header_type.0 & 0x80 != 0 {
                self.function += 1;
            } else {
                self.function = u8::MAX;
            }
            return Some(addr);
        }

        loop {
            self.function += 1;

            if self.config_space.exists(addr) {
                return Some(addr);
            }
            addr = PciAddr::new(0, self.bus, self.device, self.function);
            if 8 <= self.function {
                return None;
            }
        }
    }
}

unsafe fn read_header_type(addr: PciAddr, config_space: impl ConfigSpace) -> HeaderType {
    let reg = config_space.read(addr, 0xc);
    HeaderType(reg.wrapping_shr(16) as u8)
}
