use alloc::sync::Arc;

use crate::memory::ProcAddrSpace;

#[derive(Debug)]
pub struct Process {
    address_space: Arc<ProcAddrSpace>,
}
