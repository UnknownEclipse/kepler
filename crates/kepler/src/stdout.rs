pub mod serial;
pub mod terminal;

use core::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicUsize};

use cache_padded::CachePadded;
use log::{set_logger, Log, SetLoggerError};
pub use terminal::terminal;

pub fn init_logger() -> Result<(), SetLoggerError> {
    set_logger(&Logger)
}

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        serial::println!("[{} {}] {}", record.target(), record.level(), record.args());
        terminal::println!("[{} {}] {}", record.target(), record.level(), record.args());
    }

    fn flush(&self) {}
}

struct Buffer<'a> {
    slice: &'a UnsafeCell<[MaybeUninit<u8>]>,
    head1: CachePadded<AtomicUsize>,
    head2: CachePadded<AtomicUsize>,
    tail: CachePadded<AtomicUsize>,
}

impl<'a> Buffer<'a> {
    pub fn push_slice(&mut self, slice: &[u8]) -> bool {
        todo!()
    }
}
