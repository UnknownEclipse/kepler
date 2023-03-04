#![no_std]
#![allow(clippy::missing_safety_doc)]

use hal::{
    page::Size4KiB,
    region::{PhysRegion, VirtRegion},
};
use spin::Once;

#[derive(Debug)]
pub enum VmError {
    GlobalNotInitialized,
    Other,
}

pub unsafe trait VirtualMemoryAllocator {
    fn allocate_pages(&self, count: usize, options: PageAllocOptions)
        -> Result<*mut [u8], VmError>;

    unsafe fn deallocate_pages(&self, pages: *mut [u8]) -> Result<(), VmError>;

    unsafe fn map(
        &self,
        physical_region: PhysRegion<Size4KiB>,
        virtual_region: VirtRegion<Size4KiB>,
        options: MapOptions,
    ) -> Result<(), VmError>;

    unsafe fn unmap(&self, virtual_region: VirtRegion<Size4KiB>) -> Result<(), VmError>;

    fn allocate_physical(&self, n: usize) -> Result<PhysRegion<Size4KiB>, VmError>;

    unsafe fn deallocate_physical(&self, region: PhysRegion<Size4KiB>) -> Result<(), VmError>;

    fn allocate_virtual(&self, n: usize) -> Result<VirtRegion<Size4KiB>, VmError>;

    unsafe fn deallocate_virtual(&self, region: VirtRegion<Size4KiB>) -> Result<(), VmError>;
}

#[derive(Debug)]
pub struct PageAllocOptions {
    zeroed: bool,
    force_commit: bool,
    physically_continuous: bool,
}

#[derive(Debug)]
pub struct MapOptions {
    present: bool,
}

static GLOBAL: Once<&'static (dyn VirtualMemoryAllocator + Send + Sync)> = Once::new();

pub fn init_global<M>(vm: &'static M)
where
    M: VirtualMemoryAllocator + Send + Sync,
{
    GLOBAL.call_once(|| vm);
}

#[derive(Debug, Default)]
pub struct Global;

impl Global {
    fn global(&self) -> Result<&'static (dyn VirtualMemoryAllocator + Send + Sync), VmError> {
        GLOBAL.get().copied().ok_or(VmError::GlobalNotInitialized)
    }
}

unsafe impl VirtualMemoryAllocator for Global {
    fn allocate_pages(
        &self,
        count: usize,
        options: PageAllocOptions,
    ) -> Result<*mut [u8], VmError> {
        self.global()?.allocate_pages(count, options)
    }

    unsafe fn deallocate_pages(&self, pages: *mut [u8]) -> Result<(), VmError> {
        self.global()
            .expect("global not initialized")
            .deallocate_pages(pages)
    }

    unsafe fn map(
        &self,
        physical_region: PhysRegion<Size4KiB>,
        virtual_region: VirtRegion<Size4KiB>,
        options: MapOptions,
    ) -> Result<(), VmError> {
        self.global()?.map(physical_region, virtual_region, options)
    }

    unsafe fn unmap(&self, virtual_region: VirtRegion<Size4KiB>) -> Result<(), VmError> {
        self.global()?.unmap(virtual_region)
    }

    fn allocate_physical(&self, n: usize) -> Result<PhysRegion<Size4KiB>, VmError> {
        self.global()?.allocate_physical(n)
    }

    unsafe fn deallocate_physical(&self, region: PhysRegion<Size4KiB>) -> Result<(), VmError> {
        self.global()
            .expect("global not initialized")
            .deallocate_physical(region)
    }

    fn allocate_virtual(&self, n: usize) -> Result<VirtRegion<Size4KiB>, VmError> {
        self.global()?.allocate_virtual(n)
    }

    unsafe fn deallocate_virtual(&self, region: VirtRegion<Size4KiB>) -> Result<(), VmError> {
        self.global()
            .expect("global not initialized")
            .deallocate_virtual(region)
    }
}
