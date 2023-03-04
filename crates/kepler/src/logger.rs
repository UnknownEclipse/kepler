use tracing_core::Subscriber;

use crate::stdout::serial;

#[derive(Debug)]
pub struct KernelSubscriber {}

impl KernelSubscriber {
    pub fn new() -> Self {
        KernelSubscriber {}
    }
}

impl Subscriber for KernelSubscriber {
    fn enabled(&self, metadata: &tracing_core::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &tracing_core::span::Attributes<'_>) -> tracing_core::span::Id {
        todo!()
    }

    fn record(&self, span: &tracing_core::span::Id, values: &tracing_core::span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &tracing_core::span::Id, follows: &tracing_core::span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing_core::Event<'_>) {
        serial::println!("{:?}", event);
    }

    fn enter(&self, span: &tracing_core::span::Id) {
        todo!()
    }

    fn exit(&self, span: &tracing_core::span::Id) {
        todo!()
    }
}
