use hal::vm_types::VirtRegion;

#[derive(Debug)]
pub struct Capabilities {
    /// The address space this process can occupy. This is useful for a cluster of
    /// small tasks living in the same address space.
    pub address_space_bounds: VirtRegion,
    /// The maximum amount of physical memory that can be allocated to this process.
    pub memory_allocation_limit: usize,
}
