use core::{
    alloc::AllocError,
    iter::Step,
    ptr::{self, NonNull},
};

use hal::{
    paging::DirectlyMappedPageTable,
    vm_types::{Page, VirtAddr, VirtRegion},
};
use log::trace;
use spin::{mutex::SpinMutex, Lazy};

use super::{map_guard, map_lazy, map_normal, AllocOptions, PAGE_SIZE};
use crate::{
    error::{KernErrorKind, KernResult},
    memory::{frame_allocator::hhdm_end, get_active_page_table},
};

pub static KERNEL_ADDRESS_SPACE: Lazy<SpinMutex<KernelAddressSpace>> =
    Lazy::new(|| unsafe { SpinMutex::new(make_kernel_addrspace()) });

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
            for _ in 0..num_usable_pages {
                map_lazy(page, page_table)?;
                page = Step::forward(page, 1);
            }
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
