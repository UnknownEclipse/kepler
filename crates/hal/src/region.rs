use core::ptr;

use hal_core::{page::PageSize, region::Region};

use crate::addr::{Phys, Virt};

pub type VirtRegion<S> = Region<Virt, S>;
pub type PhysRegion<S> = Region<Phys, S>;

pub fn region_as_slice<S>(region: VirtRegion<S>) -> *mut [u8]
where
    S: PageSize,
{
    let base = region.start.base().to_u64() as usize as *mut u8;
    ptr::slice_from_raw_parts_mut(base, region.len() as usize * S::SIZE.get())
}
