use core::ptr;

use bitflags::bitflags;
use vm_types::{
    Caching, Frame, FrameAllocError, FrameAllocator, MapOptions, Page, PageLookupError,
    PageTableError, PhysAddr, VirtAddr,
};

use super::{instr::invlpg, reg::cr3};

const PRESENT_BIT: u32 = 0;
const WRITE_BIT: u32 = 1;
const USER_BIT: u32 = 2;
const WRITE_THROUGH_BIT: u32 = 3;
const NO_CACHE_BIT: u32 = 4;
const ACCESSED_BIT: u32 = 5;
const DIRTY_BIT: u32 = 6;
const NO_EXEC_BIT: u32 = 63;

#[derive(Debug)]
pub struct DirectlyMappedPageTable {
    l4: &'static mut RawPageTable,
    phys_base: VirtAddr,
}

impl DirectlyMappedPageTable {
    pub unsafe fn from_l4(addr: VirtAddr, phys_base: VirtAddr) -> Self {
        let l4 = &mut *addr.as_ptr();
        Self { l4, phys_base }
    }

    pub unsafe fn active(phys_base: VirtAddr) -> Self {
        let cr3 = cr3::read() & !0b11;

        let addr = VirtAddr::from_usize(cr3);
        Self::from_l4(addr, phys_base)
    }

    pub fn new<P>(phys_base: VirtAddr, frame_allocator: &P) -> Result<Self, PageTableError>
    where
        P: ?Sized + FrameAllocator,
    {
        let frame = frame_allocator.allocate_frame()?;
        let l4 = unsafe {
            let phys_base = phys_base.as_ptr::<u8>();
            let phys = frame.addr().as_usize();
            let addr = phys_base.add(phys).cast();
            ptr::write_bytes(addr, 0, 1);
            &mut *addr
        };
        Ok(Self { l4, phys_base })
    }

    unsafe fn map_not_present<P>(
        &mut self,
        virt: Page,
        bits: usize,
        frame_allocator: &P,
    ) -> Result<(), FrameAllocError>
    where
        P: ?Sized + FrameAllocator,
    {
        let entry = self.get_entry(virt, frame_allocator)?;
        let entry_bits = bits & !1;
        ptr::write(entry, PageTableEntry(entry_bits));
        Ok(())
    }

    fn get_entry<'a, P>(
        &'a mut self,
        page: Page,
        frame_allocator: &P,
    ) -> Result<&'a mut PageTableEntry, FrameAllocError>
    where
        P: ?Sized + FrameAllocator,
    {
        let [l3_index, l2_index, l1_index, l0_index] = address_parts(page.addr().as_usize());

        let l4 = &mut *self.l4;
        let l3 = get_subtable(l4, l3_index, frame_allocator, self.phys_base)?;
        let l2 = get_subtable(l3, l2_index, frame_allocator, self.phys_base)?;
        let l1 = get_subtable(l2, l1_index, frame_allocator, self.phys_base)?;

        Ok(RawPageTable::get_entry(l1, l0_index))
    }
    // pub fn map(&self, phys: usize, virt: *mut u8 ,)
}

unsafe impl vm_types::PageTable for DirectlyMappedPageTable {
    unsafe fn map<P>(&mut self, options: &MapOptions, phys_alloc: &P) -> Result<(), PageTableError>
    where
        P: ?Sized + FrameAllocator,
    {
        let MapOptions {
            frame,
            page,
            present,
            write,
            execute,
            caching,
            user_bits,
            flush_tlb,
            user_accessible,
        } = *options;

        let mut bits = 0;

        bits |= usize::from(present) << PRESENT_BIT;
        bits |= usize::from(write) << WRITE_BIT;
        bits |= usize::from(user_accessible) << USER_BIT;
        bits |= usize::from(!execute) << NO_EXEC_BIT;
        bits |= usize::from(caching == Caching::NoCache) << NO_CACHE_BIT;
        bits |= usize::from(caching == Caching::WriteThrough) << WRITE_THROUGH_BIT;

        let user_low3 = user_bits & 0b111;
        let user_high5 = user_bits.wrapping_shr(3);

        bits |= usize::from(user_low3) << 9;
        bits |= usize::from(user_high5) << 52;
        bits |= frame.addr().as_usize();

        let entry = self.get_entry(page, phys_alloc)?;
        entry.0 = bits;

        if flush_tlb {
            unsafe { invlpg(page.addr().as_ptr()) };
        }

        Ok(())
    }

    unsafe fn map_missing<P>(
        &mut self,
        page: Page,
        bits: usize,
        phys_alloc: &P,
    ) -> Result<(), PageTableError>
    where
        P: ?Sized + FrameAllocator,
    {
        let entry = self.get_entry(page, phys_alloc)?;
        entry.0 = bits & !1;
        Ok(())
    }

    unsafe fn load(&'static self) {
        let reg = VirtAddr::from_ptr(self.l4).as_usize();
        cr3::write(reg);
    }

    fn lookup(&mut self, page: Page) -> Result<Frame, PageLookupError> {
        let [l3_index, l2_index, l1_index, l0_index] = address_parts(page.addr().as_usize());

        let base = self.phys_base;

        let l4 = &mut *self.l4;
        let l3 = try_get_subtable(l4, l3_index, base)?;
        let l2 = try_get_subtable(l3, l2_index, base)?;
        let l1 = try_get_subtable(l2, l1_index, base)?;

        let entry = l1.0[l0_index];
        if entry.is_present() {
            Ok(entry.frame())
        } else {
            Err(PageLookupError::MissingPageEntry(entry.0))
        }
    }
}

fn try_get_subtable(
    parent: &mut RawPageTable,
    i: usize,
    phys_base: VirtAddr,
) -> Result<&mut RawPageTable, PageLookupError> {
    let phys_base = phys_base.as_ptr::<u8>();
    let entry = parent.get_entry(i);

    if entry.is_present() {
        let phys = entry.frame().addr().as_usize();
        let virt = unsafe { phys_base.add(phys) };
        unsafe { Ok(&mut *virt.cast()) }
    } else {
        Err(PageLookupError::MissingPageTable(entry.0))
    }
}

fn get_subtable<'a, P>(
    parent: &'a mut RawPageTable,
    i: usize,
    frame_allocator: &P,
    phys_base: VirtAddr,
) -> Result<&'a mut RawPageTable, FrameAllocError>
where
    P: ?Sized + FrameAllocator,
{
    let phys_base = phys_base.as_ptr::<u8>();
    let i = i as usize;
    let entry = parent.get_entry(i);

    if entry.is_present() {
        let phys = entry.frame().addr().as_usize();
        let virt = unsafe { phys_base.add(phys) };
        return unsafe { Ok(&mut *virt.cast()) };
    }

    let frame = frame_allocator.allocate_frame()?;
    unsafe {
        let table: *mut RawPageTable = phys_base.add(frame.addr().as_usize()).cast();
        table.write_bytes(0, 1);
        *entry = PageTableEntry::new(frame);

        Ok(&mut *table)
    }
}

#[repr(C, align(4096))]
#[derive(Debug, Clone, Copy)]
struct RawPageTable([PageTableEntry; 512]);

impl RawPageTable {
    fn get_entry(&mut self, i: usize) -> &mut PageTableEntry {
        &mut self.0[i]
    }
}

fn address_parts(addr: usize) -> [usize; 4] {
    let addr = addr.wrapping_shr(12);
    let l0 = addr & 0x1ff;
    let addr = addr.wrapping_shr(9);
    let l1 = addr & 0x1ff;
    let addr = addr.wrapping_shr(9);
    let l2 = addr & 0x1ff;
    let addr = addr.wrapping_shr(9);
    let l3 = addr & 0x1ff;
    [l3, l2, l1, l0]
}

struct PageTableEntryBuilder {
    flags: PageTableEntryFlags,
}

impl PageTableEntryBuilder {
    pub fn new() -> Self {
        Self {
            flags: PageTableEntryFlags::PRESENT,
        }
    }

    pub fn write(&mut self) -> &mut Self {
        self.flags |= PageTableEntryFlags::WRITE;
        self
    }

    pub fn user_accessible(&mut self) -> &mut Self {
        self.flags |= PageTableEntryFlags::USER;
        self
    }

    pub fn build(&self, frame: Frame) -> PageTableEntry {
        PageTableEntry(self.flags.bits() | frame.addr().as_usize())
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
struct PageTableEntry(usize);

bitflags! {
    pub struct PageTableEntryFlags: usize {
        const PRESENT = 1 << 0;
        const WRITE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const NO_CACHE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
    }
}

impl PageTableEntry {
    /// Construct a minimal page table entry with the following attributes:
    /// ```
    /// return PageTableEntry  {
    ///     address: addr,
    ///     present: true,
    ///     writable: true,
    ///     executable: true,
    ///     cache: WriteBack,
    ///     user_bits: 0,
    ///     protection_key: 0,
    ///     available: 0,
    ///     global: false,
    ///     user_accessible: false,
    ///     page_access_table: false,
    ///     dirty: false,
    ///     accessed: false,
    /// }
    /// ```
    pub fn new(frame: Frame) -> Self {
        let bits = frame.addr().as_usize() & !0xfff;
        let bits = bits | 3;
        Self(bits)
    }

    pub fn is_present(&self) -> bool {
        PageTableEntryFlags::from_bits_truncate(self.0).contains(PageTableEntryFlags::PRESENT)
    }

    pub fn frame(&self) -> Frame {
        let addr = self.0 & !0xfff & !(0xfff << 52);
        let addr = PhysAddr::from_usize(addr);
        Frame::from_base(addr).unwrap()
    }
}
