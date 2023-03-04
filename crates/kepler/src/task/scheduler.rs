use core::ptr::NonNull;

use spin::Lazy;

use super::head::TaskHead;

#[derive(Debug)]
pub struct CpuScheduler {
    current_thread: Option<NonNull<TaskHead>>,
}

unsafe impl Send for CpuScheduler {}
unsafe impl Sync for CpuScheduler {}

impl CpuScheduler {
    pub fn new() -> Self {
        todo!()
    }
}

static SCHEDULER: Lazy<CpuScheduler> = Lazy::new(CpuScheduler::new);
