use core::fmt::{self, Write};

use hal::interrupts;
use limine::{LimineTerminal, LimineTerminalRequest, LimineTerminalResponse};
use lock_api::{MappedMutexGuard, MutexGuard};
use spin::{
    mutex::{SpinMutex, SpinMutexGuard},
    Once,
};

static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);
static WRITER: lock_api::Mutex<SpinMutex<()>, Option<TerminalRaw>> = lock_api::Mutex::new(None);
static TERMINAL: Once<Option<SpinMutex<TerminalRaw>>> = Once::new();

pub fn terminal_raw() -> Option<&'static SpinMutex<TerminalRaw>> {
    TERMINAL
        .call_once(|| unsafe { TerminalRaw::new().map(SpinMutex::new) })
        .as_ref()
}

pub fn terminal() -> Option<TerminalLock> {
    let mut guard = WRITER.lock();
    if guard.is_none() {
        *guard = unsafe { TerminalRaw::new() };
    }
    let guard = MutexGuard::try_map(guard, |raw| raw.as_mut()).ok()?;
    Some(TerminalLock { guard })
}

#[non_exhaustive]
pub struct Terminal;

impl Terminal {
    pub fn lock(&self) -> TerminalLock {
        let guard = MutexGuard::map(WRITER.lock(), |option| {
            option.as_mut().expect("no terminal found")
        });
        TerminalLock { guard }
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.lock().write_str(s)
    }
}

pub struct TerminalLock {
    guard: MappedMutexGuard<'static, SpinMutex<()>, TerminalRaw>,
}

impl fmt::Write for TerminalLock {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.guard.write_str(s)
    }
}

macro_rules! print {
    ($($arg:tt)*) => (
        $crate::stdout::terminal::_print(format_args!($($arg)*))
    );
}

macro_rules! println {
    () => ($crate::stdout::terminal::print!("\n"));
    ($($arg:tt)*) => ($crate::stdout::terminal::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    if let Some(t) = terminal_raw() {
        interrupts::without(|_| {
            t.lock()
                .write_fmt(args)
                .expect("failed to write to terminal");
        })
    }
}

pub(crate) use print;
pub(crate) use println;

#[derive(Debug)]
pub struct TerminalRaw {
    response: &'static LimineTerminalResponse,
    terminal: &'static LimineTerminal,
}

impl TerminalRaw {
    pub unsafe fn new() -> Option<Self> {
        let response = TERMINAL_REQUEST.get_response().get()?;
        let terminal = response.terminals().first()?;

        Some(Self { response, terminal })
    }
}

unsafe impl Send for TerminalRaw {}

impl fmt::Write for TerminalRaw {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(w) = self.response.write() {
            w(self.terminal, s);
        }
        Ok(())
    }
}

pub struct LockedTerminal {
    inner: SpinMutexGuard<'static, Option<TerminalRaw>>,
}

impl fmt::Write for LockedTerminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(t) = &mut *self.inner {
            t.write_str(s)?;
        }
        Ok(())
    }
}
