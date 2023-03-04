use core::{cell::SyncUnsafeCell, ops::Shl};

use bitflags::bitflags;
use hal_core::{
    addr::Addr,
    page::{Page, Size4KiB},
};
use x86_64::structures::paging::PageTableFlags;

use crate::{
    addr::{Phys, PhysAddr, Virt},
    reg::cr3,
};

#[derive(Debug)]
pub struct PageTable {
    base: u64,
    table: &'static RawPageTable,
}

impl PageTable {
    pub unsafe fn from_cr3(offset: u64) -> PageTable {
        let r = cr3::read();
        let phys = r.wrapping_shr(12).shl(12);
        let virt = phys + offset;

        let table = {
            let ptr = virt as usize as *mut PageTable4Kib;
            &*ptr.cast()
        };

        Self {
            base: offset,
            table,
        }
    }

    // pub unsafe fn map(
    //     &mut self,
    //     options: MapOptions,
    //     frame_allocator: impl Fn() -> Option<Page<Phys, Size4KiB>>,
    // ) -> Result<(), ()> {
    // }
}

#[derive(Debug, Default)]
pub enum Caching {
    #[default]
    WriteBack,
    WriteThrough,
    None,
}

#[derive(Debug)]
pub struct Mapping {
    virt: u64,
    phys: u64,
    flags: PageTableFlags,
}

impl Mapping {
    pub fn new(src: Page<Phys, Size4KiB>, dst: Page<Virt, Size4KiB>) -> Self {
        todo!()
    }

    pub fn caching(&mut self, caching: Caching) -> &mut Self {
        let flags = match caching {
            Caching::WriteBack => PageTableFlags::empty(),
            Caching::WriteThrough => PageTableFlags::WRITE_THROUGH,
            Caching::None => PageTableFlags::NO_CACHE,
        };

        self.flags
            .remove(PageTableFlags::NO_CACHE | PageTableFlags::WRITE_THROUGH);
        self.flags.insert(flags);

        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.flags |= PageTableFlags::WRITABLE;
        self
    }

    pub fn execute(&mut self) -> &mut Self {
        self.flags.remove(PageTableFlags::NO_EXECUTE);
        self
    }

    pub fn present(&mut self) -> &mut Self {
        self.flags.insert(PageTableFlags::PRESENT);
        self
    }

    pub fn user_accessible(&mut self) -> &mut Self {
        self.flags.insert(PageTableFlags::USER_ACCESSIBLE);
        self
    }
}

#[derive(Debug)]
pub struct MapOptions {
    page: Page<Virt, Size4KiB>,
    frame: Page<Phys, Size4KiB>,
    flags: Flags,
}

bitflags! {
    struct Flags: u64 {}
}

#[repr(transparent)]
#[derive(Debug)]
struct RawPageTable(SyncUnsafeCell<PageTable4Kib>);

#[repr(C, align(4096))]
#[derive(Debug)]
struct PageTable4Kib {
    entries: [Entry4KiB; 512],
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Entry4KiB(u64);

impl Entry4KiB {
    pub fn addr(&self) -> PhysAddr {
        let v = bitfrob::u64_get_value(12, 63, self.0);
        PhysAddr::new(v).expect("address should be valid")
    }
}
