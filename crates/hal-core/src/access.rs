use self::private::Sealed;

pub trait Read: Sealed {}
pub trait Write: Sealed {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadOnly;

impl Sealed for ReadOnly {}
impl Read for ReadOnly {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WriteOnly;

impl Sealed for WriteOnly {}
impl Write for WriteOnly {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReadWrite;

impl Sealed for ReadWrite {}
impl Read for ReadWrite {}
impl Write for ReadWrite {}

mod private {
    pub trait Sealed {}
}
