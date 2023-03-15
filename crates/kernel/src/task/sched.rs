use super::task_types::Task;
use crate::error::KernResult;

pub trait Scheduler: Send + Sync + 'static {
    fn unpark(&self, task: Task) -> KernResult<()>;
    fn park(&self) -> KernResult<()>;
    fn current(&self) -> KernResult<Task>;
    fn yield_now(&self) -> KernResult<()>;
    fn exit(&self) -> KernResult<!>;
    unsafe fn enter(&self) -> KernResult<!>;
}
