use core::{mem, sync::atomic::AtomicPtr};

use crate::{page::Page, ThreadId};

const SEGMENT_SIZE: usize = 1 << 22;

#[repr(C)]
#[derive(Debug)]
pub struct Segment {
    thread_id: ThreadId,
    page_shift: usize,
    // page_kind: PageKind,
    pages: [Page; (SEGMENT_SIZE - 1) / mem::size_of::<Page>()],
}
