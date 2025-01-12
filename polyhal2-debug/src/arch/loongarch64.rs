use polyhal2_core::addr::PhysAddr;
use spin::Mutex;

use crate::DebugConsole;

const UART_ADDR: PhysAddr = PhysAddr::new(0x01FE001E0);
static COM1: Mutex<Uart> = Mutex::new(Uart::new(UART_ADDR.mapped_vaddr().raw()));

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub const fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    pub fn putchar(&mut self, c: u8) {
        let ptr = self.base_address as *mut u8;
        loop {
            unsafe {
                if ptr.add(5).read_volatile() & (1 << 5) != 0 {
                    break;
                }
            }
        }
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    pub fn getchar(&mut self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            if ptr.add(5).read_volatile() & 1 == 0 {
                // The DR bit is 0, meaning no data
                None
            } else {
                // The DR bit is 1, meaning data!
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl DebugConsole {
    /// Writes a byte to the console.
    pub fn putchar(ch: u8) {
        if ch == b'\n' {
            COM1.lock().putchar(b'\r');
        }
        COM1.lock().putchar(ch)
    }

    /// read a byte, return -1 if nothing exists.
    #[inline]
    pub fn getchar() -> Option<u8> {
        COM1.lock().getchar()
    }
}
