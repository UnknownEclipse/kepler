use super::task_types::Task;

pub trait Scheduler: Send + Sync + 'static {
    fn unpark(&self, task: Task);
    fn park(&self);
    fn current(&self) -> Task;
    fn yield_now(&self);
    fn exit(&self) -> !;
    unsafe fn enter(&self) -> !;
}
