use core::{alloc::AllocError, error::Error, fmt::Display};

use hal::vm_types::PageTableError;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum KernErrorKind {
    AllocError,
}

#[derive(Debug)]
pub struct KernError {
    kind: KernErrorKind,
}

impl KernError {
    pub fn kind(&self) -> KernErrorKind {
        self.kind
    }

    pub fn as_error_code(&self) -> u32 {
        self.kind() as u32
    }
}

impl Display for KernError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "kernel error occurred: {:?}", self.kind())
    }
}

impl From<KernErrorKind> for KernError {
    fn from(value: KernErrorKind) -> Self {
        KernError { kind: value }
    }
}

impl From<AllocError> for KernError {
    fn from(value: AllocError) -> Self {
        KernErrorKind::AllocError.into()
    }
}

impl From<PageTableError> for KernError {
    fn from(value: PageTableError) -> Self {
        match value {
            PageTableError::FrameAllocError => KernErrorKind::AllocError.into(),
        }
    }
}

impl Error for KernError {}

pub type KernResult<T> = Result<T, KernError>;
