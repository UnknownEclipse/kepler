use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::{
    fmt::{Debug, Write},
    num::NonZeroU64,
    sync::atomic::{AtomicU64, Ordering},
};

use hal::interrupts::without;
use spin::mutex::SpinMutex;
use tracing::{
    field::{Field, Visit},
    span, Subscriber,
};

use crate::stdio::{stdout, Stdout};

#[derive(Debug, Default)]
pub struct KernelSubscriber {
    inner: SpinMutex<Inner>,
    ids: AtomicU64,
}

impl Subscriber for KernelSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let id = self.ids.fetch_add(1, Ordering::Relaxed) + 1;
        let id = NonZeroU64::new(id).unwrap();
        let id = span::Id::from_non_zero_u64(id);

        let span = OwnedSpan {
            id: id.clone(),
            name: span.metadata().name().to_string(),
        };

        without(|_| {
            let mut inner = self.inner.lock();
            inner.spans.push(span);
        });

        id
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        todo!()
    }

    fn event(&self, event: &tracing::Event<'_>) {
        struct Visitor<W>(W);

        impl<W> Visit for Visitor<W>
        where
            W: Write,
        {
            fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
                if field.name() == "message" {
                    write!(self.0, " {:?}", value).unwrap();
                } else {
                    write!(self.0, " {}={:?}", field.name(), value).unwrap();
                }
            }
        }

        without(|_| {
            let inner = self.inner.lock();
            let level = event.metadata().level();

            stdout()
                .lock(|w| -> core::fmt::Result {
                    write!(w, "{:>5}", level)?;

                    let mut spans = inner.stack.iter();
                    if let Some(top) = spans.next() {
                        write!(w, " {}", inner.get(top).name)?;
                    }
                    for id in spans {
                        write!(w, ":{}", inner.get(id).name)?;
                    }

                    event.record(&mut Visitor(&mut *w));
                    w.write_char('\n')?;
                    Ok(())
                })
                .unwrap();
        })
    }

    fn enter(&self, span: &span::Id) {
        without(|_| {
            let mut inner = self.inner.lock();
            inner.stack.push(span.clone());
        });
    }

    fn exit(&self, span: &span::Id) {
        without(|_| {
            let mut inner = self.inner.lock();
            for i in (0..inner.stack.len()).rev() {
                if inner.stack[i] == *span {
                    inner.stack.remove(i);
                    return;
                }
            }
        });
    }
}

#[derive(Debug, Default)]
struct Inner {
    spans: Vec<OwnedSpan>,
    stack: Vec<span::Id>,
}

impl Inner {
    fn get(&self, id: &span::Id) -> &OwnedSpan {
        let i = id.into_u64() - 1;
        &self.spans[i as usize]
    }
}

#[derive(Debug)]
struct OwnedSpan {
    id: span::Id,
    name: String,
}
