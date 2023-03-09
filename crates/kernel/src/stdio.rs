use core::fmt::{self, Write};

use hal::interrupts;
use limine::{LimineTerminalRequest, LimineTerminalResponse};
use log::Log;
use spin::{
    mutex::{SpinMutex, SpinMutexGuard},
    Lazy,
};
use uart_16550::SerialPort;

pub fn stdout() -> Stdout {
    Stdout
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Stdout;

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
        println!("[{} {}] {}", record.level(), record.target(), record.args());
    }

    fn flush(&self) {}
}

const PORT_NUMBER: u16 = 0x3f8;

static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
static STDOUT: Lazy<SpinMutex<StdoutInner>> =
    Lazy::new(|| SpinMutex::new(unsafe { StdoutInner::new() }));

struct StdoutInner {
    serial_port: SerialPort,
    limine_terminal_response: Option<&'static LimineTerminalResponse>,
}

impl StdoutInner {
    fn write_limine(&self, msg: &str) {
        let Some(response) = self.limine_terminal_response else {
            return;
        };
        let Some(terminal) = response.terminals().iter().next() else {
            return;
        };
        let Some(write) = response.write() else {
            return;
        };
        write(terminal, msg);
    }
}

impl StdoutInner {
    unsafe fn new() -> Self {
        let mut serial_port = SerialPort::new(PORT_NUMBER);
        serial_port.init();
        let terminal_response = TERMINAL_REQUEST.get_response().get();

        Self {
            serial_port,
            limine_terminal_response: terminal_response,
        }
    }
}

impl Write for StdoutInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.serial_port.write_str(s)?;
        self.write_limine(s);
        Ok(())
    }
}

unsafe impl Send for StdoutInner {}
