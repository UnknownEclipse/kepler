use crate::{
    frame_allocator::{FrameAllocError, FrameAllocator},
    Frame, Page,
};

/// An error occurring while attempting to access the page table.
#[derive(Debug)]
pub enum PageTableError {
    FrameAllocError,
}

impl From<FrameAllocError> for PageTableError {
    fn from(_: FrameAllocError) -> Self {
        Self::FrameAllocError
    }
}

/// A page table.
///
/// # Safety
/// 1. Implementors should be very certain their implementation is safe and works correctly. Errors
/// here can result in errors everywhere else!
pub unsafe trait PageTable {
    /// # Safety
    /// 1. The page must be unused
    /// 2. The frame must not be in active use. Some kernels may keep multiple mappings
    /// to the same frame for various reasons, but as long as accesses do not alias it
    /// is fine.
    unsafe fn map<P>(&mut self, options: &MapOptions, phys_alloc: &P) -> Result<(), PageTableError>
    where
        P: ?Sized + FrameAllocator;

    /// Map a page as not present. This leaves `usize::BITS - 1` available bits for storing
    /// arbirary data such as locations of paged out memory.
    ///
    /// # Safety
    /// If the memory referenced is in use a page fault will occur upon the next access.
    unsafe fn map_missing<P>(
        &mut self,
        page: Page,
        bits: usize,
        phys_alloc: &P,
    ) -> Result<(), PageTableError>
    where
        P: ?Sized + FrameAllocator;

    /// Unmap the requested page, removing it from the page table.
    ///
    /// # Safety
    /// 1. The page must be valid and not used anywhere else.
    // unsafe fn unmap<P>(&mut self, page: Page, phys: &P)
    // where
    //     P: ?Sized + FrameAllocator;

    /// Attempt to look up the frame that a given page is mapped to.
    fn lookup(&mut self, page: Page) -> Option<Frame>;

    unsafe fn load(&'static self);
}

/// Options for a single mapping.
#[derive(Debug)]
pub struct MapOptions {
    pub frame: Frame,
    pub page: Page,
    pub present: bool,
    pub write: bool,
    pub execute: bool,
    pub caching: Caching,
    pub user_bits: u8,
    pub flush_tlb: bool,
    pub user_accessible: bool,
}

impl MapOptions {
    pub fn new(frame: Frame, page: Page) -> Self {
        Self {
            frame,
            page,
            present: false,
            write: false,
            execute: false,
            caching: Caching::WriteBack,
            user_bits: 0,
            flush_tlb: true,
            user_accessible: false,
        }
    }

    /// Set the caching mode for the mapping.
    pub fn caching(&mut self, caching: Caching) -> &mut Self {
        self.caching = caching;
        self
    }

    /// Make the mapping executable.
    pub fn execute(&mut self) -> &mut Self {
        self.execute = true;
        self
    }

    /// Make the mapping writable
    pub fn write(&mut self) -> &mut Self {
        self.write = true;
        self
    }

    /// Mark the mapping as present.
    pub fn present(&mut self) -> &mut Self {
        self.present = true;
        self
    }

    /// Mark the mapping as accessible to userspace.
    pub fn user_accessible(&mut self) -> &mut Self {
        self.user_accessible = true;
        self
    }

    /// Don't flush the tlb entry during a map operation. This should only be used in *very* specific
    /// conditions.
    pub fn ignore_tlb_flush(&mut self) -> &mut Self {
        self.flush_tlb = false;
        self
    }

    /// Map everything using the provided page table and physical memory allocator
    /// # Safety
    pub unsafe fn map<T, P>(&self, page_table: &mut T, phys_alloc: &P) -> Result<(), PageTableError>
    where
        P: FrameAllocator,
        T: PageTable,
    {
        page_table.map(self, phys_alloc)
    }
}

/// The caching policy used by the cpu for an individual mapping.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Caching {
    NoCache,
    WriteThrough,
    #[default]
    WriteBack,
}
