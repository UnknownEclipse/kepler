use core::{mem::MaybeUninit, ops::Shl};

use bitfrob::u32_get_value;

#[derive(Debug, Clone, Copy)]
pub struct SubmissionQueueEntry {
    command_dword0: CommandDword0,
    namespace_id: u32,
    command_dword2: MaybeUninit<u32>,
    command_dword3: MaybeUninit<u32>,
    metadata_ptr: MaybeUninit<u64>,
    data_ptr: MaybeUninit<[u64; 2]>,
    command_dword10: MaybeUninit<u32>,
    command_dword11: MaybeUninit<u32>,
    command_dword12: MaybeUninit<u32>,
    command_dword13: MaybeUninit<u32>,
    command_dword14: MaybeUninit<u32>,
    command_dword15: MaybeUninit<u32>,
}

impl SubmissionQueueEntry {
    pub fn new(command_dword0: CommandDword0, namespace_id: u32) -> Self {
        Self {
            command_dword0,
            namespace_id,
            command_dword2: MaybeUninit::uninit(),
            command_dword3: MaybeUninit::uninit(),
            metadata_ptr: MaybeUninit::uninit(),
            data_ptr: MaybeUninit::uninit(),
            command_dword10: MaybeUninit::uninit(),
            command_dword11: MaybeUninit::uninit(),
            command_dword12: MaybeUninit::uninit(),
            command_dword13: MaybeUninit::uninit(),
            command_dword14: MaybeUninit::uninit(),
            command_dword15: MaybeUninit::uninit(),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct CommandDword0(u32);

pub enum DataTransferKind {
    Prp,
    SglQwordAlign,
    SglUnknownAlign,
}

#[derive(Debug, Clone, Copy)]
pub enum FusedOp {
    Normal,
    FirstCommand,
    SecondCommand,
    Reserved,
}

impl CommandDword0 {
    pub fn new(command_id: u16, opcode: u8) -> CommandDword0 {
        Self(u32::from(command_id).shl(16) | u32::from(opcode))
    }

    pub fn command_id(&self) -> u16 {
        self.0.wrapping_shr(16) as u16
    }

    pub fn fuse(&self) -> FusedOp {
        match u32_get_value(8, 9, self.0) {
            0b00 => FusedOp::Normal,
            0b01 => FusedOp::FirstCommand,
            0b10 => FusedOp::SecondCommand,
            0b11 => FusedOp::Reserved,
            _ => unreachable!(),
        }
    }

    pub fn opcode(&self) -> u8 {
        self.0 as u8
    }
}
