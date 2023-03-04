use crate::{unsafe_mut::UnsafeMut, volatile::Volatile};

pub type StackFrameRef<'a, T> = &'a mut UnsafeMut<Volatile<T>>;

pub trait ExceptionHandler {
    type StackFrame;
    type Error;
    type Output;

    fn handle(frame: &mut Self::StackFrame, error: Self::Error) -> Self::Output;
}

pub trait InterruptHandler {
    type StackFrame;

    fn handle(frame: &mut Self::StackFrame);
}
