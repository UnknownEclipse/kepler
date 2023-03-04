use core::mem::MaybeUninit;

use hal_core::{access::ReadOnly, volatile::Volatile};

pub use self::{
    controller_capabilities::*, controller_configuration::*, interrupt_mask::*, version::*,
};

mod controller_capabilities;
mod controller_configuration;
mod interrupt_mask;
mod version;

#[repr(C)]
#[derive(Debug)]
pub struct ControllerAttributes {
    pub controller_capabilities: Volatile<ControllerCapabilities, ReadOnly>,
    pub version: Volatile<Version, ReadOnly>,
    pub interrupt_mask_set: Volatile<InterruptMaskSet>,
    pub interrupt_mask_clear: Volatile<u32>,
    pub controller_configuration: Volatile<ControllerConfiguration>,
    pub controller_status: Volatile<u64, ReadOnly>,
    pub nvm_subsystem_reset: NvmSubsystemReset,
    pub admin_queue_attributes: Volatile<u32>,
    pub admin_submission_queue_address: Volatile<u64>,
    pub admin_completion_queue_address: Volatile<u64>,
    // Optional fields start
    pub controller_memory_buffer_location: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub controller_memory_buffer_size: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub boot_partition_info: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub boot_partition_read_select: MaybeUninit<Volatile<u32>>,
    pub boot_partition_memory_buffer_location: MaybeUninit<Volatile<u64>>,
    pub controller_memory_buffer_memory_space_control: MaybeUninit<Volatile<u64>>,
    pub controller_memory_buffer_status: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub controller_memory_buffer_elasticity_buffer_size: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub controller_memory_buffer_sustained_write_throughput: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub nvm_subsystem_shutdown: MaybeUninit<Volatile<u32>>,
    pub controller_ready_timeouts: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub persistent_memory_region_capabilities: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub persistent_memory_region_control: MaybeUninit<Volatile<u32>>,
    pub persistent_memory_region_status: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub persistent_memory_region_elasticity_buffer_size: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub persistent_memory_region_sustained_write_throughput: MaybeUninit<Volatile<u32, ReadOnly>>,
    pub persistent_memory_region_memory_space_control_lower: MaybeUninit<Volatile<u32>>,
    pub persistent_memory_region_memory_space_control_upper: MaybeUninit<Volatile<u32>>,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct NvmSubsystemReset(Volatile<u32>);

impl NvmSubsystemReset {
    pub unsafe fn reset(&mut self) {
        const CODE: u32 = u32::from_be_bytes(*b"NVMe");
        self.0.write(CODE);
    }
}
