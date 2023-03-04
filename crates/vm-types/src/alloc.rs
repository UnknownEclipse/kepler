use core::ptr::NonNull;

pub unsafe trait FrameAllocator {
    // fn allocate_frame(&self) -> Option<
}

pub unsafe trait VirtualMemoryManager {
    fn allocate_pages(
        &self,
        count: usize,
        options: PageAllocOptions,
    ) -> Result<NonNull<[u8]>, VmError>;

    unsafe fn deallocate_pages(&self, pages: NonNull<[u8]>) -> Result<NonNull<[u8]>, VmError>;
}

pub struct PageAllocOptions {}

impl PageAllocOptions {
    pub fn commit(&mut self) -> &mut Self {
        todo!()
    }

    pub fn write(&mut self) -> &mut Self {
        todo!()
    }

    pub fn execute(&mut self) -> &mut Self {
        todo!()
    }

    pub fn cache(&mut self) -> &mut Self {
        todo!()
    }

    pub fn pinned(&mut self) -> &mut Self {
        todo!()
    }
}

pub enum VmError {}
