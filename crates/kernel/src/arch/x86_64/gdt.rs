use core::{mem, ops::BitAnd};

use spin::lazy::Lazy;
use x86_64::{
    instructions::tables::{lgdt, load_tss},
    registers::segmentation::{Segment, CS},
    structures::{gdt::SegmentSelector, tss::TaskStateSegment, DescriptorTablePointer},
};

use super::tss::TSS;

static GDT: Lazy<(Gdt, Selectors)> = Lazy::new(build_gdt);

pub unsafe fn init() {
    let (gdt, selectors) = Lazy::force(&GDT);

    gdt.load();
    CS::set_reg(selectors.code_segment);
    load_tss(selectors.tss_selector);
}

pub fn selectors() -> Selectors {
    GDT.1
}

#[derive(Debug, Clone, Copy)]
pub struct Selectors {
    pub code_segment: SegmentSelector,
    pub tss_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
}

fn build_gdt() -> (Gdt, Selectors) {
    let mut gdt = Gdt::new();
    gdt.tss.set_tss(&TSS);

    let selectors = Selectors {
        code_segment: SegmentSelector(0x28),
        tss_selector: SegmentSelector(0x48),
        user_data_selector: SegmentSelector(0x40),
        user_code_selector: SegmentSelector(0x40 - 8),
    };

    (gdt, selectors)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct GdtEntry {
    limit: u16,
    base_low16: u16,
    base_mid8: u8,
    access: u8,
    granularity: u8,
    base_high8: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct TssEntry {
    len: u16,
    base_low16: u16,
    base_mid8: u8,
    flags0: u8,
    flags1: u8,
    base_high8: u8,
    base_upper32: u32,
    reserved: u32,
}

impl TssEntry {
    fn set_tss(&mut self, tss: &'static TaskStateSegment) {
        let addr: *const TaskStateSegment = tss;
        let addr = addr as usize as u64;
        self.set_address(addr);
    }

    fn set_address(&mut self, addr: u64) {
        self.base_low16 = (addr & 0xffff) as u16;
        self.base_mid8 = addr.wrapping_shr(16).bitand(0xff).try_into().unwrap();
        self.base_high8 = addr.wrapping_shr(24).bitand(0xff).try_into().unwrap();
        self.base_upper32 = addr.wrapping_shr(32).try_into().unwrap();
    }
}

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
struct Gdt {
    null: GdtEntry,
    code16: GdtEntry,
    data16: GdtEntry,
    code32: GdtEntry,
    data32: GdtEntry,
    code64: GdtEntry,
    data64: GdtEntry,
    user_data: GdtEntry,
    user_code: GdtEntry,
    tss: TssEntry,
}

// {0, 0, 0, 0, 0, 0}, // null
// {0xffff, 0, 0, 0x9a, 0x80, 0}, // 16-bit code
// {0xffff, 0, 0, 0x92, 0x80, 0}, // 16-bit data
// {0xffff, 0, 0, 0x9a, 0xcf, 0}, // 32-bit code
// {0xffff, 0, 0, 0x92, 0xcf, 0}, // 32-bit data
// {0, 0, 0, 0x9a, 0xa2, 0}, // 64-bit code
// {0, 0, 0, 0x92, 0xa0, 0}, // 64-bit data
// {0, 0, 0, 0xF2, 0, 0}, // user data
// {0, 0, 0, 0xFA, 0x20, 0}, // user code
// {0x68, 0, 0, 0x89, 0x20, 0, 0, 0} // tss
impl Gdt {
    pub fn new() -> Self {
        Gdt {
            null: GdtEntry {
                limit: 0,
                base_low16: 0,
                base_mid8: 0,
                access: 0,
                granularity: 0,
                base_high8: 0,
            },
            code16: GdtEntry {
                limit: 0xffff,
                base_low16: 0,
                base_mid8: 0,
                access: 0x9a,
                granularity: 0x80,
                base_high8: 0,
            },
            data16: GdtEntry {
                limit: 0xffff,
                base_low16: 0,
                base_mid8: 0,
                access: 0x92,
                granularity: 0x80,
                base_high8: 0,
            },
            code32: GdtEntry {
                limit: 0xffff,
                base_low16: 0,
                base_mid8: 0,
                access: 0x9a,
                granularity: 0xcf,
                base_high8: 0,
            },
            data32: GdtEntry {
                limit: 0xffff,
                base_low16: 0,
                base_mid8: 0,
                access: 0x92,
                granularity: 0xcf,
                base_high8: 0,
            },
            code64: GdtEntry {
                limit: 0,
                base_low16: 0,
                base_mid8: 0,
                access: 0x9a,
                granularity: 0xa2,
                base_high8: 0,
            },
            data64: GdtEntry {
                limit: 0,
                base_low16: 0,
                base_mid8: 0,
                access: 0x92,
                granularity: 0xa0,
                base_high8: 0,
            },
            user_code: GdtEntry {
                limit: 0,
                base_low16: 0,
                base_mid8: 0,
                access: 0xfa,
                granularity: 0x20,
                base_high8: 0,
            },
            user_data: GdtEntry {
                limit: 0,
                base_low16: 0,
                base_mid8: 0,
                access: 0xf2,
                granularity: 0,
                base_high8: 0,
            },
            tss: TssEntry {
                len: 0x68,
                base_low16: 0,
                base_mid8: 0,
                flags0: 0x89,
                flags1: 0x20,
                base_high8: 0,
                base_upper32: 0,
                reserved: 0,
            },
        }
    }

    pub fn load(&'static self) {
        unsafe { self.load_unchecked() };
    }

    unsafe fn load_unchecked(&self) {
        let ptr = DescriptorTablePointer {
            base: x86_64::VirtAddr::from_ptr(self),
            limit: mem::size_of::<Self>() as u16 - 1,
        };
        unsafe { lgdt(&ptr) };
    }
}
