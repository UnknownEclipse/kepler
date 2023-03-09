use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::fmt::{Debug, Write};

use spin::mutex::SpinMutex;
use tracing::{
    field::{Field, Visit},
    span, Level, Metadata, Subscriber,
};

use crate::stdout;

pub struct KernelSubscriber {
    inner: SpinMutex<Inner>,
}

impl KernelSubscriber {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl Subscriber for KernelSubscriber {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let meta = span.metadata();
        let name = meta.name().to_owned();
        let target = meta.target().to_owned();
        let level = *meta.level();
        let data = SpanData {
            name,
            target,
            level,
        };

        let mut inner = self.inner.lock();
        let spans = &mut inner.spans;
        let id = spans.len();
        spans.push(data);
        span::Id::from_u64(id as u64 + 1)
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing::Event<'_>) {
        let mut writer = stdout::serial::serial_port().lock();
        let meta = event.metadata();
        let guard = self.inner.lock();

        write!(writer, "[{} {}][", meta.target(), meta.level()).expect("format failed");

        for (i, span) in guard.stack.iter().enumerate() {
            let data = &guard.spans[span.clone().into_u64() as usize - 1];
            if i == 0 {
                write!(writer, "{}", data.name).expect("format failed");
            } else {
                write!(writer, "<-{}", data.name).expect("format failed");
            }
        }
        writer.write_char(']').expect("format failed");

        struct Visitor<'a, W>(&'a mut W);

        impl<'a, W> Visit for Visitor<'a, W>
        where
            W: Write,
        {
            fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                write!(self.0, " {}=\"{:?}\"", field.name(), value).unwrap();
            }
        }

        event.record(&mut Visitor(&mut *writer));
        writeln!(writer).expect("format failed");
    }

    fn enter(&self, span: &span::Id) {
        self.inner.lock().stack.push(span.clone());
    }

    fn exit(&self, span: &span::Id) {
        let mut inner = self.inner.lock();

        let Some((i,_)) = inner
            .stack
            .iter()
            .enumerate()
            .rev()
            .find(|(_, id)| *id == span) else {
                return;
            };

        inner.stack.remove(i);
    }
}

struct SpanData {
    name: String,
    target: String,
    level: Level,
}

#[derive(Default)]
struct Inner {
    spans: Vec<SpanData>,
    stack: Vec<span::Id>,
}
