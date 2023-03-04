mod header;
mod join_handle;
pub(crate) mod raw_task;

pub use self::join_handle::JoinHandle;

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    Normal,
}
