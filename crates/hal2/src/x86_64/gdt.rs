use core::mem;

use bitfrob::{u8_with_bit, u8_with_value};
use vm_types::VirtAddr;

use super::{instr::lgdt, Privilege};

#[derive(Debug)]
pub struct Gdt<const N: usize> {
    descriptors: [u64; N],
    len: u16,
}

#[repr(C, packed(2))]
#[derive(Debug, Clone, Copy)]
pub struct GdtPtr {
    limit: u16,
    base: VirtAddr,
}

impl<const N: usize> Gdt<N> {
    pub fn new() -> Self {
        Self {
            descriptors: [0; N],
            len: 1,
        }
    }

    pub fn push(&mut self, segment: Segment) -> Selector {
        let sel = Selector(self.len);
        let buf = &mut self.descriptors;
        match segment {
            Segment::Code(s) => {
                buf[self.len as usize] = s.descriptor.to_u64();
                self.len += 1;
            }
            Segment::Data(s) => {
                buf[self.len as usize] = s.descriptor.to_u64();
                self.len += 1;
            }
            Segment::System(s) => {
                buf[self.len as usize] = s.descriptor.to_u64();
                buf[self.len as usize + 1] = s.base_high.into();
                self.len += 2;
            }
        }
        sel
    }

    pub unsafe fn load(&'static self) {
        let limit = 8 * self.len - 1;
        let ptr = GdtPtr {
            base: VirtAddr::from_ptr(self),
            limit,
        };

        lgdt(&ptr);
    }
}

impl<const N: usize> Default for Gdt<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Selector(u16);

#[derive(Debug, Clone, Copy)]
pub enum Segment {
    Code(CodeSegment),
    Data(DataSegment),
    System(SystemSegment),
}

impl Segment {
    pub fn kernel_code16() -> Self {
        Self::Code(CodeSegment::kernel16())
    }

    pub fn kernel_data16() -> Self {
        Self::Data(DataSegment::kernel16())
    }

    pub fn kernel_code32() -> Self {
        Self::Code(CodeSegment::kernel32())
    }

    pub fn kernel_data32() -> Self {
        Self::Data(DataSegment::kernel32())
    }

    pub fn kernel_code64() -> Self {
        Self::Code(CodeSegment::kernel64())
    }

    pub fn kernel_data64() -> Self {
        Self::Data(DataSegment::kernel64())
    }

    pub fn tss64(tss: *const Tss64) -> Self {
        let mut descriptor = Descriptor::zeroed();
        let addr = tss as usize;
        descriptor.set_base(addr as u32);
        descriptor.set_limit(mem::size_of::<Tss64>() as u32);
        descriptor.access = 0b10001001;

        Self::System(SystemSegment {
            descriptor,
            base_high: addr.wrapping_shr(32) as u32,
        })
    }
}

#[repr(C, packed(4))]
pub struct Tss64 {
    _reserved1: u32,
    pub rsp: [VirtAddr; 3],
    _reserved2: [u32; 2],
    pub ist: [VirtAddr; 7],
    _reserved3: [u32; 2],
    _reserved4: u16,
    pub iopb: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct CodeSegment {
    descriptor: Descriptor,
}

impl CodeSegment {
    pub fn kernel16() -> Self {
        Self::new(0, 0xffff).mode(Mode::X16).readable(true)
    }

    pub fn kernel32() -> Self {
        Self::new(0, 0xffffffff).mode(Mode::X32).readable(true)
    }

    pub fn kernel64() -> Self {
        Self::new(0, 0).mode(Mode::X64).readable(true)
    }

    pub fn user() -> Self {
        todo!()
    }

    pub fn new(base: u32, limit: u32) -> Self {
        let mut descriptor = Descriptor::zeroed();
        descriptor.set_present(true);
        descriptor.set_kind(Kind::Code);
        descriptor.set_exec(true);
        descriptor.set_base(base);
        descriptor.set_limit(limit);
        Self { descriptor }
    }

    pub fn privilege(mut self, privilege: Privilege) -> Self {
        self.descriptor.set_dpl(privilege);
        self
    }

    pub fn conforming(mut self, conforming: bool) -> Self {
        self.descriptor.set_conforming(conforming);
        self
    }

    pub fn readable(mut self, readable: bool) -> Self {
        self.descriptor.set_rw(readable);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        self.descriptor.set_mode(mode);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DataSegment {
    descriptor: Descriptor,
}

impl DataSegment {
    pub fn kernel16() -> Self {
        Self::new(0, 0xffff).mode(Mode::X16).writable(true)
    }

    pub fn kernel32() -> Self {
        Self::new(0, 0xffffffff).mode(Mode::X32).writable(true)
    }

    pub fn kernel64() -> Self {
        Self::new(0, 0).mode(Mode::X64).writable(true)
    }

    pub fn user() -> Self {
        todo!()
    }

    pub fn new(base: u32, limit: u32) -> Self {
        let mut descriptor = Descriptor::zeroed();
        descriptor.set_present(true);
        descriptor.set_kind(Kind::Data);
        descriptor.set_base(base);
        descriptor.set_limit(limit);
        Self { descriptor }
    }

    pub fn privilege(mut self, privilege: Privilege) -> Self {
        self.descriptor.set_dpl(privilege);
        self
    }

    pub fn direction(mut self, dir: Direction) -> Self {
        self.descriptor.set_direction(dir);
        self
    }

    pub fn writable(mut self, writable: bool) -> Self {
        self.descriptor.set_rw(writable);
        self
    }

    pub fn mode(mut self, mode: Mode) -> Self {
        assert_ne!(mode, Mode::X64, "864");
        self.descriptor.set_mode(mode);
        self
    }
}
#[derive(Debug, Clone, Copy)]
pub struct SystemSegment {
    descriptor: Descriptor,
    base_high: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Descriptor {
    limit_low: u16,
    base_low: u16,
    base_mid: u8,
    access: u8,
    limit_high: u8,
    base_high: u8,
}

impl Descriptor {
    pub fn zeroed() -> Self {
        unsafe { mem::transmute(0u64) }
    }

    pub fn set_base(&mut self, base: u32) {
        self.base_low = base as u16;
        self.base_mid = base.wrapping_shr(16) as u8;
        self.base_high = base.wrapping_shr(24) as u8;
    }

    pub fn set_limit(&mut self, limit: u32) {
        self.limit_low = limit as u16;
        let high = limit.wrapping_shr(16) as u8 & 0xf;
        self.limit_high = u8_with_value(3, 7, self.limit_high, high);
    }

    pub fn set_dpl(&mut self, dpl: Privilege) {
        let dpl = dpl as u8;
        self.access = u8_with_value(5, 6, self.access, dpl);
    }

    pub fn set_present(&mut self, present: bool) {
        self.access = u8_with_bit(7, self.access, present);
    }

    pub fn set_kind(&mut self, kind: Kind) {
        let kind = matches!(kind, Kind::Code | Kind::Data);
        self.access = u8_with_bit(4, self.access, kind);
    }

    pub fn set_direction(&mut self, dir: Direction) {
        let set = dir == Direction::GrowsDown;
        self.access = u8_with_bit(2, self.access, set);
    }

    pub fn set_conforming(&mut self, conforming: bool) {
        self.access = u8_with_bit(2, self.access, conforming);
    }

    pub fn set_rw(&mut self, value: bool) {
        self.access = u8_with_bit(1, self.access, value);
    }

    pub fn to_u64(self) -> u64 {
        unsafe { mem::transmute(self) }
    }

    pub fn set_exec(&mut self, exec: bool) {
        self.access = u8_with_bit(3, self.access, exec);
    }

    pub fn set_granularity(&mut self, granularity: Granularity) {
        let bit = matches!(granularity, Granularity::Page);
        self.limit_high = u8_with_bit(3, self.limit_high, bit);
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let mode = match mode {
            Mode::X16 => 0b00,
            Mode::X32 => 0b10,
            Mode::X64 => 0b01,
        };
        self.limit_high = u8_with_value(1, 2, self.limit_high, mode);
    }
}

enum Kind {
    System,
    Code,
    Data,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    GrowsUp,
    GrowsDown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    X16,
    X32,
    X64,
}

#[derive(Debug)]
enum Granularity {
    Byte,
    Page,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct Access(u8);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct LimitHigh(u8);

impl LimitHigh {
    pub fn set_limit(&mut self, high: u8) {
        todo!()
    }
}
