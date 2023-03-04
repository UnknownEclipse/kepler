use core::fmt::{self, Write};

use tracing::{span, Event};

use crate::stdout::{serial::serial_port, terminal::terminal_raw};

pub struct Subscriber {}

impl tracing::Subscriber for Subscriber {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        todo!()
    }

    fn record(&self, span: &span::Id, values: &span::Record<'_>) {
        todo!()
    }

    fn record_follows_from(&self, span: &span::Id, follows: &span::Id) {
        todo!()
    }

    fn event(&self, event: &Event<'_>) {
        struct Visitor<'a> {
            writer: &'a mut dyn Write,
        }

        impl<'a> tracing::field::Visit for Visitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
                write!(self.writer, " '{}'={:?}", field.name(), value)
                    .expect("failed to write to log");
            }
        }

        let mut terminal = terminal_raw();
        if let Some(t) = terminal {
            let mut writer = t.lock();

            write!(writer, "[{}]", event.metadata().target()).expect("failed to write to log");
            event.record(&mut Visitor {
                writer: &mut *writer,
            })
        }

        let mut terminal = terminal_raw();
        if let Some(t) = terminal {
            let mut writer = t.lock();

            write!(writer, "[{}]", event.metadata().target()).expect("failed to write to log");
            event.record(&mut Visitor {
                writer: &mut *writer,
            })
        }

        let mut serial_port = serial_port();
        let mut writer = serial_port.lock();

        write!(writer, "[{}]", event.metadata().target()).expect("failed to write to log");
        event.record(&mut Visitor {
            writer: &mut *writer,
        })
    }

    fn enter(&self, span: &span::Id) {
        todo!()
    }

    fn exit(&self, span: &span::Id) {
        todo!()
    }
}
