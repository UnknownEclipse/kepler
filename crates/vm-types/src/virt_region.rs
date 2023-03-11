use core::{
    iter::Step,
    ptr::{self, NonNull},
};

use crate::{Page, PageSize, Size4KiB};

pub struct VirtRegion {
    pub start: Page,
    pub end: Page,
}

impl VirtRegion {
    pub fn len(&self) -> usize {
        Step::steps_between(&self.start, &self.end).unwrap_or(0) * Size4KiB::SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn as_ptr(&self) -> *mut [u8] {
        let start = self.start.addr().as_ptr();
        ptr::slice_from_raw_parts_mut(start, self.len())
    }
}
