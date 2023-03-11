pub mod instr;
pub mod interrupts;
pub mod paging;
pub mod port;
pub mod random;
pub mod reg;
pub mod syscall;
pub mod task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Privilege {
    Ring0,
    Ring1,
    Ring2,
    Ring3,
}
