use core::alloc::Layout;

use hal::vm_types::{PageTable, VirtRegion};

use self::capability::Capabilities;
use crate::error::KernResult;

pub mod capability;

pub struct Process {
    capabilities: Capabilities,
}

/// Memory objects: objects that may be mapped into a process' address space
pub trait Mem {
    fn region_layout(&self) -> Layout;

    fn map<P>(&self, region: VirtRegion, page_table: &mut P) -> KernResult<()>
    where
        P: ?Sized + PageTable;

    fn unmap<P>(&self, region: VirtRegion, page_table: &mut P) -> KernResult<()>
    where
        P: ?Sized + PageTable;
}

/// I/O objects: objects that can be read from and written to
///
/// I/O devices work a bit differently from traditional unix file descriptors:
/// they are intended to be massively async and expose command buffers as their
/// native api.
pub trait Io {}
