pub mod x86_64;

pub use self::x86_64::{init, interrupts, CpuId};

#[derive(Debug, Clone, Copy)]
pub enum IpiTarget {
    Others,
    Single(CpuId),
}

pub mod cpu {
    use hal::task::init_hw_thread;

    pub use super::x86_64::cpu::{get, init, CpuId};
}

// #[derive(Debug, Clone, Copy)]
// pub struct CpuMask {
//     bits: u32,
// }

// impl CpuMask {
//     pub fn insert(&mut self, cpu: CpuId) {
//         todo!()
//     }

//     pub fn remove(&mut self, cpu: CpuId) {
//         todo!()
//     }

//     pub fn contains(&self, cpu: CpuId) -> bool {
//         todo!()
//     }
// }
