#![no_std]

#[derive(Debug, Clone, Copy)]
pub struct PciAddr {
    segment: u16,
    bus: u8,
    device_func_bits: u8,
}

impl PciAddr {
    pub fn new(segment: u16, bus: u8, device: u8, function: u8) -> Self {
        Self {
            segment,
            bus,
            device_func_bits: (device << 3) | function,
        }
    }

    pub fn segment(&self) -> u16 {
        self.segment
    }

    pub fn bus(&self) -> u8 {
        self.bus
    }

    pub fn device(&self) -> u8 {
        self.device_func_bits.wrapping_shr(3)
    }

    pub fn function(&self) -> u8 {
        self.device_func_bits & 0b111
    }
}

pub unsafe trait ConfigSpace {
    fn exists(&self, addr: PciAddr) -> bool;
    unsafe fn read(&self, addr: PciAddr, offset: u16) -> u32;
    unsafe fn write(&self, addr: PciAddr, offset: u16, value: u32);
}

unsafe impl<C: ConfigSpace> ConfigSpace for &C {
    fn exists(&self, addr: PciAddr) -> bool {
        (**self).exists(addr)
    }

    unsafe fn read(&self, addr: PciAddr, offset: u16) -> u32 {
        (**self).read(addr, offset)
    }

    unsafe fn write(&self, addr: PciAddr, offset: u16, value: u32) {
        (**self).write(addr, offset, value)
    }
}

struct IoAccessMechanism {}

#[derive(Debug, Clone, Copy)]
pub struct DeviceId(pub u16);

#[derive(Debug, Clone, Copy)]
pub struct VendorId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassId(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubclassId(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct RevId(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct HeaderType(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct Status(u16);

#[derive(Debug, Clone, Copy)]
pub struct Command(u16);

#[derive(Debug)]
pub enum Bar {
    Memory32 {
        addr: u32,
        size: u32,
        prefetchable: bool,
    },
    Memory64 {
        addr: u64,
        size: u64,
        prefetchable: bool,
    },
    Io {
        port: u32,
    },
}

#[derive(Debug)]
pub struct Header<'a, C> {
    addr: PciAddr,
    config_space: &'a C,
}

pub struct BaseHeader {
    device_id: DeviceId,
    vendor_id: VendorId,
    status: Status,
    command: Command,
    class_id: ClassId,
    subclass_id: SubclassId,
    rev_id: RevId,
    bist: u8,
    header_type: HeaderType,
    latency_timer: u8,
    cache_line_size: u8,
}

unsafe fn read_base_header(config_space: impl ConfigSpace, addr: PciAddr) -> BaseHeader {
    let reg0 = config_space.read(addr, 0x0);
    let vendor_id = VendorId(reg0 as u16);
    let device_id = DeviceId(reg0.wrapping_shr(16) as u16);

    let reg1 = config_space.read(addr, 0x4);
    let status = Status(reg1.wrapping_shr(16) as u16);
    let command = Command(reg1 as u16);

    let reg2 = config_space.read(addr, 0x8);
    let class_id = ClassId(reg2.wrapping_shr(24) as u8);
    let subclass_id = SubclassId(reg2.wrapping_shr(16) as u8);
    let rev_id = RevId(reg2 as u8);

    let reg3 = config_space.read(addr, 0xc);
    let bist = reg3.wrapping_shr(24) as u8;
    let header_type = HeaderType(reg3.wrapping_shr(16) as u8);
    let latency_timer = reg3.wrapping_shr(8) as u8;
    let cache_line_size = reg3 as u8;

    BaseHeader {
        device_id,
        vendor_id,
        status,
        command,
        class_id,
        subclass_id,
        rev_id,
        bist,
        header_type,
        latency_timer,
        cache_line_size,
    }
}

impl<'a, C> Header<'a, C>
where
    C: ConfigSpace,
{
    pub fn device_id(&self) -> (VendorId, DeviceId) {
        let v = unsafe { self.config_space.read(self.addr, 0) };
        let vendor_id = (v & 0xffff) as u16;
        let device_id = v.wrapping_shr(16) as u16;
        (VendorId(vendor_id), DeviceId(device_id))
    }
}
