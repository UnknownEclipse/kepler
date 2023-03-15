use alloc::sync::Arc;
use core::ptr::{self, NonNull};

use bitflags::bitflags;
use hal::{
    interrupts,
    paging::DirectlyMappedPageTable,
    vm_types::{
        FrameAllocator, MapOptions, Page, PageTable, PageTableError, PhysAddr, VirtAddr, VirtRegion,
    },
};
use limine::{LimineHhdmRequest, LimineMemmapRequest};
use log::trace;
use spin::{mutex::SpinMutex, Lazy};

pub use self::process::ProcAddrSpace;
use self::{kernel::KERNEL_ADDRESS_SPACE, user::UserAddressSpace};
use crate::error::KernResult;

mod allocator;
mod frame_allocator;
mod kernel;
mod page_fault;
mod process;
mod user;

pub unsafe fn init() -> KernResult<()> {
    trace!("beginning initialization");
    frame_allocator::init();
    Lazy::force(&KERNEL_ADDRESS_SPACE);
    allocator::init()?;
    trace!("finished initialization");
    Ok(())
}

pub unsafe fn map_physical_addr(phys: PhysAddr) -> VirtAddr {
    VirtAddr::from_ptr(hhdm_start().add(phys.as_usize()))
}

#[derive(Debug, Clone)]
pub enum AddrSpace {
    Kernel,
    User(Arc<UserAddressSpace>),
}

impl AddrSpace {
    pub fn allocate(&self, options: &AllocOptions) -> KernResult<NonNull<[u8]>> {
        match self {
            AddrSpace::Kernel => {
                interrupts::without(|_| KERNEL_ADDRESS_SPACE.lock().allocate(options))
            }
            AddrSpace::User(_) => todo!("userspace allocations"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllocOptions {
    num_pages: usize,
    start_guard_pages: usize,
    end_guard_pages: usize,
    eager_commit: bool,
}

impl AllocOptions {
    pub fn new(size: usize) -> Self {
        let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

        Self {
            num_pages,
            start_guard_pages: 0,
            end_guard_pages: 0,
            eager_commit: true,
        }
    }

    pub fn start_guard_pages(&mut self, count: usize) -> &mut Self {
        self.start_guard_pages = count;
        self
    }

    pub fn end_guard_pages(&mut self, count: usize) -> &mut Self {
        self.end_guard_pages = count;
        self
    }

    pub fn lazy_commit(&mut self) -> &mut Self {
        self.eager_commit = false;
        self
    }

    pub fn allocate_in_address_space(&self, addr_space: &AddrSpace) -> KernResult<NonNull<[u8]>> {
        addr_space.allocate(self)
    }
}

fn map_guard<P>(page: Page, page_table: &mut P) -> Result<(), PageTableError>
where
    P: PageTable,
{
    unsafe {
        page_table.map_missing(
            page,
            MissingPageFlags::GUARD_PAGE.bits(),
            &frame_allocator::Global,
        )
    }
}

fn map_normal<P>(page: Page, page_table: &mut P) -> Result<(), PageTableError>
where
    P: PageTable,
{
    let frame = frame_allocator::Global.allocate_frame()?;

    unsafe {
        MapOptions::new(frame, page)
            .execute()
            .write()
            .present()
            .map(page_table, &frame_allocator::Global)?;
    }

    Ok(())
}

fn map_lazy<P>(page: Page, page_table: &mut P) -> Result<(), PageTableError>
where
    P: PageTable,
{
    let bits = MissingPageFlags::DELAYED_COMMIT.bits();
    unsafe { page_table.map_missing(page, bits, &frame_allocator::Global)? };
    Ok(())
}

bitflags! {
    struct MissingPageFlags: usize {
        const DELAYED_COMMIT = 1 << 1;
        const GUARD_PAGE = 1 << 2;
    }
}

fn hhdm_start() -> *mut u8 {
    HHDM_REQUEST.get_response().get().unwrap().offset as usize as *mut u8
}

pub unsafe fn handle_page_fault(addr: VirtAddr) -> KernResult<()> {
    let address_space = AddrSpace::Kernel;
    let page = Page::containing(addr);

    match address_space {
        AddrSpace::Kernel => {
            let mut kernel_addr_space = KERNEL_ADDRESS_SPACE.lock();
            let page_table = kernel_addr_space.page_table();
            handle_page_fault_with_page_table(page, page_table)
        }
        AddrSpace::User(_) => todo!("userspace allocation"),
    }
}

unsafe fn handle_page_fault_with_page_table<P>(page: Page, page_table: &mut P) -> KernResult<()>
where
    P: PageTable,
{
    trace!("committing to page {:p}", page);
    map_normal(page, page_table)?;
    Ok(())
}

const PAGE_SIZE: usize = 4096;

static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
static MMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);

unsafe fn get_active_page_table() -> DirectlyMappedPageTable {
    let hhdm_response = HHDM_REQUEST
        .get_response()
        .get()
        .expect("higher-half direct map failed");

    let phys_base = VirtAddr::from_usize(hhdm_response.offset as usize);

    DirectlyMappedPageTable::active(phys_base)
}

static USERSPACE: Lazy<SpinMutex<UserAddressSpace>> = Lazy::new(|| {
    let base = VirtAddr::from_usize(1 << 22);
    let top = base.as_usize() + (1 << 26);
    let top = VirtAddr::from_usize(top);
    let region = VirtRegion {
        end: Page::from_base(base).unwrap(),
        start: Page::from_base(top).unwrap(),
    };

    SpinMutex::new(UserAddressSpace::new(region))
});

pub fn allocate_user(size: usize) -> KernResult<NonNull<[u8]>> {
    let pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut kern_address_space = KERNEL_ADDRESS_SPACE.lock();
    let page_table = kern_address_space.page_table();

    let region = USERSPACE.lock().allocate_stack(pages).unwrap();
    let phys_alloc = &mut frame_allocator::Global;
    for page in region {
        let frame = phys_alloc.allocate_frame()?;

        unsafe {
            MapOptions::new(frame, page)
                .execute()
                .write()
                .user_accessible()
                .present()
                .map(page_table, phys_alloc)
                .unwrap();
        }
    }

    let start = region.start.addr().as_ptr();
    let len = region.len();
    trace!("len := {}", len);

    let ptr = ptr::slice_from_raw_parts_mut(start, len);
    Ok(NonNull::new(ptr).unwrap())
}
