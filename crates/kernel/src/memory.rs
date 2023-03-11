use core::{
    alloc::AllocError,
    iter::Step,
    ops::Range,
    ptr::{self, NonNull},
};

use hal::{
    interrupts,
    paging::DirectlyMappedPageTable,
    vm_types::{FrameAllocator, MapOptions, Page, PageTable, PageTableError, VirtAddr, VirtRegion},
};
use limine::{LimineHhdmRequest, LimineMemmapRequest};
use log::trace;
use spin::{mutex::SpinMutex, Lazy};

use self::frame_allocator::hhdm_end;
use crate::error::{KernErrorKind, KernResult};

mod allocator;
mod frame_allocator;
mod kernel_address_space;
mod page_fault;

pub unsafe fn init() -> KernResult<()> {
    trace!("beginning initialization");
    frame_allocator::init();
    Lazy::force(&KERNEL_ADDRESS_SPACE);
    allocator::init()?;
    trace!("finished initialization");
    Ok(())
}

#[derive(Debug, Clone)]
pub enum AddrSpace {
    Kernel,
    // Process(Arc<ProcessAddrSpace>),
}

impl AddrSpace {
    pub fn allocate(&self, options: &AllocOptions) -> KernResult<NonNull<[u8]>> {
        match self {
            AddrSpace::Kernel => {
                interrupts::without(|_| KERNEL_ADDRESS_SPACE.lock().allocate(options))
            }
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

    pub fn eager_commit(&mut self) -> &mut Self {
        self.eager_commit = true;
        self
    }

    pub fn allocate_in_address_space(&self, addr_space: &AddrSpace) -> KernResult<NonNull<[u8]>> {
        addr_space.allocate(self)
    }
}

#[derive(Debug)]
pub struct KernelAddressSpace {
    kernel_heap_start: VirtAddr,
    kernel_heap_end: VirtAddr,
    kernel_heap_ptr: VirtAddr,
    page_table: DirectlyMappedPageTable,
}

impl KernelAddressSpace {
    pub fn page_table(&mut self) -> &mut DirectlyMappedPageTable {
        &mut self.page_table
    }

    /// Allocate a virtual region usable by the kernel. If requested, guard pages will
    /// be inserted above and below the allocation.
    pub fn allocate(&mut self, options: &AllocOptions) -> KernResult<NonNull<[u8]>> {
        let num_usable_pages = options.num_pages;
        let num_pages = num_usable_pages + options.start_guard_pages + options.end_guard_pages;

        // let region = self.allocate_unmapped_region(num_pages)?;
        let start: *mut u8 = self.kernel_heap_ptr.as_ptr();
        let end = start.wrapping_add(num_pages * PAGE_SIZE);

        let start = self.kernel_heap_ptr;
        let end = VirtAddr::from_ptr(end);

        if !(start..self.kernel_heap_end).contains(&end) {
            return Err(KernErrorKind::AllocError.into());
        }
        self.kernel_heap_ptr = end;

        let page_table = &mut self.page_table;

        let mut page = Page::from_base(start).unwrap();

        for _ in 0..options.start_guard_pages {
            map_guard(page, page_table)?;
            page = Step::forward(page, 1);
        }

        let start = page;
        if options.eager_commit {
            for _ in 0..num_usable_pages {
                map_normal(page, page_table)?;
                page = Step::forward(page, 1);
            }
        } else {
            page = Step::forward(page, num_usable_pages);
        }
        let end = page;

        for _ in 0..options.end_guard_pages {
            map_guard(page, page_table)?;
            page = Step::forward(page, 1);
        }

        let ptr = start.addr().as_ptr::<u8>();
        let len = Step::steps_between(&start, &end).unwrap() * PAGE_SIZE;
        unsafe {
            let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
            Ok(NonNull::new_unchecked(ptr))
        }
    }

    /// Allocate a region of the kernel address space, but does not perform any mapping
    /// or other operations.
    fn allocate_unmapped_region(&mut self, num_pages: usize) -> KernResult<VirtRegion> {
        let start = Page::from_base(self.kernel_heap_ptr).unwrap();

        let end = Step::forward_checked(start, num_pages).ok_or(AllocError)?;
        if self.kernel_heap_end < end.addr() {
            return Err(AllocError.into());
        }

        self.kernel_heap_ptr = end.addr();
        Ok(VirtRegion { start, end })
    }
}

fn map_guard<P>(page: Page, page_table: &mut P) -> Result<(), PageTableError>
where
    P: PageTable,
{
    unsafe { page_table.map_missing(page, 2, &frame_allocator::Global) }
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

    let address_space_size = kernel_heap_end.as_usize() - kernel_heap_start.as_usize();

    trace!(
        "kernel address space is {}gb",
        address_space_size / 1_000_000_000
    );

    KernelAddressSpace {
        kernel_heap_end,
        kernel_heap_ptr,
        kernel_heap_start,
        page_table,
    }
}
