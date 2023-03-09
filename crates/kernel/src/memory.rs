use core::{iter::Step, ops::Range};

use bitflags::bitflags;
use hal::{
    paging::DirectlyMappedPageTable,
    vm_types::{FrameAllocator, MapOptions, Page, PageTable, PageTableError, VirtAddr},
};
use limine::{LimineHhdmRequest, LimineMemmapRequest};
use log::trace;
use spin::{mutex::SpinMutex, Lazy};

use self::frame_allocator::hhdm_end;
pub use self::page_fault::PageFaultHandler;

mod allocator;
pub mod frame_allocator;
mod page_fault;

pub unsafe fn init() {
    trace!("beginning initialization");
    frame_allocator::init();
    Lazy::force(&KERNEL_ADDRESS_SPACE);
    allocator::init();
    trace!("finished initialization");
}

#[derive(Debug, Clone)]
pub enum AddrSpace {
    Kernel,
    // Process(Arc<ProcessAddrSpace>),
}

#[derive(Debug)]
pub struct KernelAddressSpace {
    // hhdm_base: VirtAddr,
    kernel_heap_start: VirtAddr,
    kernel_heap_end: VirtAddr,
    kernel_heap_ptr: VirtAddr,
    page_table: DirectlyMappedPageTable,
}

impl KernelAddressSpace {
    pub fn page_table(&mut self) -> &mut DirectlyMappedPageTable {
        &mut self.page_table
    }
}

bitflags! {
    struct UnCommittedPageFlags: u8 {
        /// This page is a guard page. Accesses should result in an error.
        const GUARD = 1;
    }
}

#[derive(Debug)]
pub enum VirtualRegionAllocError {
    PageTable(PageTableError),
    NoSpace,
}

impl From<PageTableError> for VirtualRegionAllocError {
    fn from(value: PageTableError) -> Self {
        Self::PageTable(value)
    }
}

impl KernelAddressSpace {
    /// Allocate a virtual region usable by the kernel. If requested, guard pages will
    /// be inserted above and below the allocation.
    pub fn allocate_virtual_region(
        &mut self,
        size: usize,
        guard_pages: usize,
    ) -> Result<Range<Page>, VirtualRegionAllocError> {
        let num_usable_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;
        let num_pages = num_usable_pages + guard_pages * 2;

        let start: *mut u8 = self.kernel_heap_ptr.as_ptr();
        let end = start.wrapping_add(num_pages * PAGE_SIZE);

        let start = self.kernel_heap_ptr;
        let end = VirtAddr::from_ptr(end);

        if !(start..self.kernel_heap_end).contains(&end) {
            Err(VirtualRegionAllocError::NoSpace)
        } else {
            self.kernel_heap_ptr = end;
            let start = Page::from_base(start).unwrap();
            let end = Page::from_base(end).unwrap();

            let page_table = &mut self.page_table;

            let mut page = start;
            for _ in 0..guard_pages {
                map_guard(page, page_table)?;
                page = Step::forward(page, 1);
            }

            for _ in 0..num_usable_pages {
                map_normal(page, page_table)?;
                page = Step::forward(page, 1);
            }

            for _ in 0..guard_pages {
                map_guard(page, page_table)?;
                page = Step::forward(page, 1);
            }
            Ok(Step::forward(start, guard_pages)..Step::backward(end, guard_pages))
        }
    }
}

fn map_guard<P>(page: Page, page_table: &mut P) -> Result<(), PageTableError>
where
    P: PageTable,
{
    unsafe {
        page_table.map_missing(
            page,
            usize::from(UnCommittedPageFlags::GUARD.bits()) << 1,
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

fn hhdm_start() -> *mut u8 {
    HHDM_REQUEST.get_response().get().unwrap().offset as usize as *mut u8
}

const PAGE_SIZE: usize = 4096;

static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
static MMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest::new(0);
pub static KERNEL_ADDRESS_SPACE: Lazy<SpinMutex<KernelAddressSpace>> =
    Lazy::new(|| unsafe { SpinMutex::new(make_kernel_addrspace()) });

unsafe fn get_active_page_table() -> DirectlyMappedPageTable {
    let hhdm_response = HHDM_REQUEST
        .get_response()
        .get()
        .expect("higher-half direct map failed");

    let phys_base = VirtAddr::from_usize(hhdm_response.offset as usize);

    DirectlyMappedPageTable::active(phys_base)
}

unsafe fn make_kernel_addrspace() -> KernelAddressSpace {
    let page_table = get_active_page_table();

    let kernel_heap_start = hhdm_end();
    let kernel_heap_end = VirtAddr::from_usize(usize::MAX);
    let kernel_heap_ptr = kernel_heap_start;

    KernelAddressSpace {
        kernel_heap_end,
        kernel_heap_ptr,
        kernel_heap_start,
        page_table,
    }
}
