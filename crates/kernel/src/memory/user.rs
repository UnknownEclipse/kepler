use core::iter::Step;

use hal::vm_types::{Page, VirtRegion};

use crate::error::{KernErrorKind, KernResult};

#[derive(Debug)]
pub struct UserAddressSpace {
    region: VirtRegion,
    heap_ptr: Page,
    stack_ptr: Page,
}

impl UserAddressSpace {
    pub fn allocate_stack(&self, count: usize) -> KernResult<VirtRegion> {
        let end = self.stack_ptr;
        let start = Step::backward_checked(end, count)
            .ok_or(KernErrorKind::AllocError)
            .unwrap();

        // if start <= self.heap_ptr {
        //     return Err(KernErrorKind::AllocError.into());
        // }

        let region = VirtRegion { end, start };
        Ok(region)
    }

    pub(crate) fn new(region: VirtRegion) -> UserAddressSpace {
        Self {
            region,
            heap_ptr: region.start,
            stack_ptr: region.end,
        }
    }
}
