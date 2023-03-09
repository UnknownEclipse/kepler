use hal::interrupts::{ExceptionHandler, StackFrame};

#[derive(Debug)]
pub struct PageFaultHandler;

impl ExceptionHandler for PageFaultHandler {
    type Error = u64;
    type Output = ();

    fn handle(stack_frame: &mut StackFrame, error: Self::Error) -> Self::Output {
        todo!()
    }
}
