use core::{iter::Step, ptr};

use crate::{Page, PageSize, Size4KiB};

#[derive(Debug, Clone, Copy)]
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

impl IntoIterator for VirtRegion {
    type IntoIter = Iter;
    type Item = Page;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            start: self.start,
            end: self.end,
        }
    }
}
#[derive(Debug)]
pub struct Iter {
    start: Page,
    end: Page,
}

impl Iterator for Iter {
    type Item = Page;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        let page = self.start;
        self.start = Step::forward(page, 1);
        Some(page)
    }
}
