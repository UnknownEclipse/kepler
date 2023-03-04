#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature<A> {
    Random,
    Arch(A),
}
