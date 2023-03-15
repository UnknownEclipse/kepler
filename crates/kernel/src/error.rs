use alloc::boxed::Box;
use core::{
    alloc::AllocError as StdAllocError,
    error::Error,
    fmt::Display,
    num::{NonZeroU32, NonZeroU8},
    sync::atomic::AtomicPtr,
};

use hal::vm_types::{FrameAllocError, PageTableError};
use spin::mutex::SpinMutex;

/// General error kind. This is what is typically passed to userspace. Internal errors
/// (`KernError`) holds much more context.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernErrorKind {
    AllocError,
    /// An internal kernel error.
    Fault,
}

/// Kernel error type.
///
/// We need to keep this the same size as a pointer, bonus points if there is a niche
/// optimization available.
#[derive(Debug)]
pub struct KernError {
    inner: ErrorInner,
}

#[derive(Debug)]
enum ErrorInner {
    Dyn {
        /// The error kind
        kind: KernErrorKind,
        /// A handle to the actual error object in the registry.
        handle: RegistryHandle,
    },
    Simple(KernErrorKind),
}

impl ErrorInner {
    pub fn kind(&self) -> KernErrorKind {
        match self {
            ErrorInner::Dyn { kind, .. } => *kind,
            ErrorInner::Simple(kind) => *kind,
        }
    }
}

#[derive(Debug)]
enum AllocErrorInner {
    Std,
}

type O = Option<KernError>;

impl KernError {
    pub fn kind(&self) -> KernErrorKind {
        self.inner.kind()
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
        KernError {
            inner: ErrorInner::Simple(value),
        }
    }
}

impl From<StdAllocError> for KernError {
    fn from(_: StdAllocError) -> Self {
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

impl From<FrameAllocError> for KernError {
    fn from(_: FrameAllocError) -> Self {
        KernErrorKind::AllocError.into()
    }
}

impl Error for KernError {}

pub type KernResult<T> = Result<T, KernError>;

pub enum ErrorContext {
    None,
    Code(u16),
}

#[derive(Debug)]
struct ErrorCtx(NonZeroU32);

struct ErrorRegistry {
    levels: [AtomicPtr<()>; 8],
    grow_lock: SpinMutex<()>,
}

type BoxError = Box<dyn Error + Send + Sync>;

impl ErrorRegistry {
    pub fn register_boxed(&self, error: BoxError) -> Result<RegistryHandle, BoxError> {
        todo!()
    }

    fn register_with(
        &self,
        f: &mut dyn FnMut() -> KernResult<BoxError>,
    ) -> KernResult<Option<RegistryHandle>> {
        todo!()
    }
}

#[derive(Debug)]
struct RegistryHandle(NonZeroU8);
