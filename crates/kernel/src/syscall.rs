//! As a microkernel, Sol does support syscalls, however they are not the primary mode
//! of interfacing with the kernel. Instead, syscalls are used to establish a set of
//! queues that exist in shared memory with the kernel. The queues are then used
//! to batch operations that would traditionally fall under the "syscall" umbrella.
//! As such we really just need to create some basic operations and everything else
//! will go through the queue interface.
//!
//! These necessary syscalls include:
//!
//! 1. Futex
//! Futexes are the primary means of synchronization, within processes, between processes,
//! and between the kernel and processes. Queues are waited upon by treating the
//! head/tail value as a futex.
//!
//! 2. mem
//!
//! The mem system call is used to interact with memory mappings. It takes a similar
//! role as mmap/unmap on linux.

const SYSCALLS: &[extern "C" fn()] = &[];

extern "C" fn sys_futex() {}
