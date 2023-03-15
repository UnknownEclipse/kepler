use alloc::{boxed::Box, vec::Vec};
use core::{
    alloc::AllocError,
    fmt::{self, Write},
};

use hal::{interrupts, task::hw_thread_id};
use limine::{LimineTerminalRequest, LimineTerminalResponse};
use log::{Level, Log};
use owo_colors::OwoColorize;
use spin::{
    mutex::{SpinMutex, SpinMutexGuard},
    Lazy,
};
use uart_16550::SerialPort;

pub fn stdout() -> Stdout {
    Stdout(())
}

#[derive(Debug)]
pub struct Stdout(());

impl Stdout {
    pub fn lock<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut StdoutLock<'_>) -> T,
    {
        interrupts::without(|_| {
            let guard = STDOUT.lock();
            let mut stdout_lock = StdoutLock { guard };
            f(&mut stdout_lock)
        })
    }
}

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.lock(|w| w.write_str(s))
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        self.lock(|w| w.write_char(c))
    }

    fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result {
        self.lock(|w| w.write_fmt(args))
    }
}
pub struct StdoutLock<'a> {
    guard: SpinMutexGuard<'a, StdoutInner>,
}

impl<'a> StdoutLock<'a> {
    pub fn register_additional_writer<W>(&mut self, writer: W) -> KernResult<()>
    where
        W: Write + Send + 'static,
    {
        let boxed = Box::try_new(writer)?;

        self.guard
            .extra_writers
            .try_reserve(1)
            .map_err(|_| AllocError)?;

        self.guard.extra_writers.push(boxed);

        Ok(())
    }
}

impl<'a> Write for StdoutLock<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.guard.write_str(s)
    }
}

macro_rules! print {
    ($($arg:tt)*) => (
        $crate::stdio::_print(format_args!($($arg)*))
    );
}

macro_rules! println {
    () => ($crate::stdio::print!("\n"));
    ($($arg:tt)*) => ($crate::stdio::print!("{}\n", format_args!($($arg)*)));
}

pub(crate) use print;
pub(crate) use println;

use crate::{error::KernResult, task};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    // Currently there isn't any case where this errors, *however* we can't use unwrap
    // because this is used in the panic handler. Recursive panics are a no-no.
    _ = stdout().write_fmt(args);
}

#[derive(Debug)]
pub struct StdoutLogger;

impl Log for StdoutLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        _ = stdout().lock(|w| -> fmt::Result {
            w.write_str("[")?;
            match record.level() {
                Level::Error => write!(w, "{}", record.level().red().bold())?,
                Level::Warn => write!(w, "{}", record.level().yellow().bold())?,
                Level::Info => write!(w, "{}", record.level().green().bold())?,
                Level::Debug => write!(w, "{}", record.level().blue().bold())?,
                Level::Trace => write!(w, "{}", record.level().white().bold())?,
            }
            write!(w, " {}", record.target().bold())?;
            write!(w, " {}={:#x}", "cpu".bold(), unsafe { hw_thread_id() })?;
            // if let Ok(t) = task::try_current() {
            //     write!(w, " {}={}", "task".bold(), t)?;
            // }
            w.write_char(']')?;
            writeln!(w, " {}", record.args())?;
            Ok(())
        });

        // println!("{}", "blue!".blue());
    }

    fn flush(&self) {}
}

const PORT_NUMBER: u16 = 0x3f8;

static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
static STDOUT: Lazy<SpinMutex<StdoutInner>> =
    Lazy::new(|| SpinMutex::new(unsafe { StdoutInner::new() }));

struct StdoutInner {
    serial_port: Option<SerialPort>,
    limine_terminal: Option<LimineWriter>,
    extra_writers: Vec<Box<dyn Write + Send>>,
}

impl StdoutInner {
    unsafe fn new() -> Self {
        let mut serial_port = SerialPort::new(PORT_NUMBER);
        serial_port.init();

        let terminal_response = TERMINAL_REQUEST.get_response().get();

        Self {
            serial_port: Some(serial_port),
            limine_terminal: terminal_response.map(LimineWriter),
            extra_writers: Vec::new(),
        }
    }
}

impl Write for StdoutInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(serial_port) = &mut self.serial_port {
            serial_port.write_str(s)?;
        }
        if let Some(limine_terminal) = &mut self.limine_terminal {
            limine_terminal.write_str(s)?;
        }
        for writer in self.extra_writers.iter_mut() {
            writer.write_str(s)?;
        }
        Ok(())
    }
}

unsafe impl Send for StdoutInner {}

struct LimineWriter(&'static LimineTerminalResponse);

impl LimineWriter {
    fn write(&self, msg: &str) {
        let Some(terminal) = self.0.terminals().iter().next() else {
            return;
        };
        let Some(write) = self.0.write() else {
            return;
        };
        write(terminal, msg);
    }
}

impl Write for LimineWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}
