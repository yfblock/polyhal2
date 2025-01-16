//! Uart 16550.

use spin::Mutex;
use uart_16550::SerialPort;

use crate::DebugConsole;

static COM1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3f8) });

impl DebugConsole {
    pub fn putchar(c: u8) {
        COM1.lock().send(c);
    }

    pub fn getchar() -> Option<u8> {
        COM1.lock().try_receive().ok()
    }
}
