use core::{fmt::Debug, time::Duration};

use bitflags::bitflags;
use bitfrob::{u64_get_bit, u64_get_value};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ControllerCapabilities(u64);

impl Debug for ControllerCapabilities {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ControllerCapabilities")
            .field("bits", &self.0)
            .field(
                "requires_contiguous_queues",
                &self.requires_contiguous_queues(),
            )
            .field("maximum_queue_entries", &self.maximum_queue_entries())
            .field(
                "supported_arbitration_mechanism",
                &self.supported_arbitration_mechanism(),
            )
            .field("timeout", &self.timeout())
            .field("doorbell_stride", &self.doorbell_stride())
            .field(
                "supports_nvme_subsystem_reset",
                &self.supports_nvme_subsystem_reset(),
            )
            .field("supported_command_sets", &self.supported_command_sets())
            .field("supports_boot_partition", &self.supports_boot_partition())
            .field("controller_power_scope", &self.controller_power_scope())
            .field("min_page_size", &self.min_page_size())
            .field("max_page_size", &self.max_page_size())
            .finish_non_exhaustive()
    }
}

bitflags! {
    pub struct SupportedArbitrationMechanisms: u8 {
        const WEIGHTED_ROUND_ROBIN = 0b001;
        const VENDOR_SPECIFIC = 0b010;
        const ROUND_ROBIN = 0b100;
    }
}

bitflags! {
    pub struct SupportedCommandSets: u8 {
        const NO_IO_COMMAND_SETS = 1 << 7;
        const IO_COMMAND_SET_WITH_IDENTIFY = 1 << 6;
        const NVM_COMMAND_SET = 1;
    }
}

bitflags! {
    pub struct ControllerReadyModes: u8 {
        const WITH_MEDIA_SUPPORT = 0;
        const WITHOUT_MEDIA_SUPPORT = 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerPowerScope {
    NotReported,
    ControllerScope,
    DomainScope,
    NvmSubsystemScope,
}

impl ControllerCapabilities {
    #[inline]
    pub fn requires_contiguous_queues(&self) -> bool {
        u64_get_bit(16, self.0)
    }

    #[inline]
    pub fn maximum_queue_entries(&self) -> u16 {
        u64_get_value(0, 15, self.0) as u16
    }

    #[inline]
    pub fn supported_arbitration_mechanism(&self) -> SupportedArbitrationMechanisms {
        let bits = u64_get_value(17, 18, self.0) as u8;
        SupportedArbitrationMechanisms::from_bits(bits).unwrap()
            | SupportedArbitrationMechanisms::ROUND_ROBIN
    }

    #[inline]
    pub fn timeout(&self) -> Duration {
        const TICK_INTERVAL: Duration = Duration::from_millis(500);
        let ticks = u64_get_value(24, 31, self.0) as u32;
        TICK_INTERVAL * ticks
    }

    #[inline]
    pub fn doorbell_stride(&self) -> u8 {
        let shift = u64_get_value(32, 35, self.0) as u32;
        2 << (2 + shift)
    }

    #[inline]
    pub fn supports_nvme_subsystem_reset(&self) -> bool {
        u64_get_bit(36, self.0)
    }

    #[inline]
    pub fn supported_command_sets(&self) -> SupportedCommandSets {
        let bits = u64_get_value(37, 45, self.0) as u8;
        SupportedCommandSets::from_bits_truncate(bits)
    }

    #[inline]
    pub fn supports_boot_partition(&self) -> bool {
        u64_get_bit(45, self.0)
    }

    #[inline]
    pub fn controller_power_scope(&self) -> ControllerPowerScope {
        match u64_get_value(46, 47, self.0) {
            0b00 => ControllerPowerScope::NotReported,
            0b01 => ControllerPowerScope::ControllerScope,
            0b10 => ControllerPowerScope::DomainScope,
            0b11 => ControllerPowerScope::NvmSubsystemScope,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn min_page_size(&self) -> u64 {
        let v = u64_get_value(48, 51, self.0);
        1 << (12 + v)
    }

    #[inline]
    pub fn max_page_size(&self) -> u64 {
        let v = u64_get_value(52, 55, self.0);
        1 << (12 + v)
    }

    #[inline]
    pub fn supports_persistent_memory_region(&self) -> bool {
        u64_get_bit(56, self.0)
    }

    #[inline]
    pub fn supports_controller_memory_buffer(&self) -> bool {
        u64_get_bit(57, self.0)
    }

    #[inline]
    pub fn supports_nvm_subsystem_shutdown(&self) -> bool {
        u64_get_bit(58, self.0)
    }

    #[inline]
    pub fn controller_ready_modes(&self) -> ControllerReadyModes {
        let bits = u64_get_value(59, 60, self.0) as u8;
        ControllerReadyModes::from_bits(bits).unwrap()
    }
}
