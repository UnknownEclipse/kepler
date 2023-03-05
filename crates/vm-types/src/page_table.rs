use crate::{
    frame_allocator::{PhysAllocError, PhysicalMemoryAllocator},
    Frame, Page,
};

#[derive(Debug)]
pub enum PageTableError {
    PhysAllocError,
}

impl From<PhysAllocError> for PageTableError {
    fn from(_: PhysAllocError) -> Self {
        Self::PhysAllocError
    }
}

pub unsafe trait PageTable {
    fn map<P>(&mut self, options: MapOptions, phys_alloc: &P) -> Result<(), PageTableError>
    where
        P: ?Sized + PhysicalMemoryAllocator;

    unsafe fn unmap<P>(&mut self, page: Page, phys: &P)
    where
        P: ?Sized + PhysicalMemoryAllocator;

    unsafe fn lookup(&mut self, page: Page) -> Option<Frame>;
}

#[derive(Debug)]
pub struct MapOptions {
    frame: Frame,
    page: Page,
    present: bool,
    write: bool,
    execute: bool,
    caching: Option<Caching>,
    user_bits: u8,
}

impl MapOptions {
    pub fn new(frame: Frame, page: Page) -> Self {
        Self {
            frame,
            page,
            present: false,
            write: false,
            execute: false,
            caching: Some(Caching::WriteBack),
            user_bits: 0,
        }
    }

    pub fn map<T, P>(self, page_table: &mut T, phys_alloc: &P) -> Result<(), PageTableError>
    where
        P: PhysicalMemoryAllocator,
        T: PageTable,
    {
        page_table.map(self, phys_alloc)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Caching {
    WriteThrough,
    WriteBack,
}
