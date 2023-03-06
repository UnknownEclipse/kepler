#![no_std]
#![feature(strict_provenance)]

pub use crate::once_cell::OnceCell;

mod event_count;
pub mod lazy;
mod mutex;
pub mod once_cell;
