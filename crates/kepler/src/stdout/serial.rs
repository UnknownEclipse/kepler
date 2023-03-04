use core::fmt::{self, Write};

use hal::interrupts;
use spin::{mutex::SpinMutex, Once};
use uart_16550::SerialPort;

const PORT_NUMBER: u16 = 0x3f8;

static SERIAL_PORT: Once<SpinMutex<SerialPort>> = Once::new();

pub fn serial_port() -> &'static SpinMutex<SerialPort> {
    SERIAL_PORT.call_once(|| unsafe {
        let mut serial_port = SerialPort::new(PORT_NUMBER);
        serial_port.init();
        SpinMutex::new(serial_port)
    })
}

macro_rules! print {
    ($($arg:tt)*) => (
        $crate::stdout::serial::_print(format_args!($($arg)*))
    );
}

macro_rules! println {
    () => ($crate::stdout::serial::print!("\n"));
    ($($arg:tt)*) => ($crate::stdout::serial::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    static SERIAL_PORT: SpinMutex<Option<SerialPort>> = SpinMutex::new(None);

    interrupts::without(|_| {
        let mut guard = SERIAL_PORT.lock();
        let port = guard.get_or_insert_with(|| {
            let mut serial_port = unsafe { SerialPort::new(PORT_NUMBER) };
            serial_port.init();
            serial_port
        });
        port.write_fmt(args).expect("format error");
    });
}

pub(crate) use print;
pub(crate) use println;
