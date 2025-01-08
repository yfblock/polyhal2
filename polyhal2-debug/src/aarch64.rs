use arm_pl011::Pl011Uart;
use polyhal2_base::addr::PhysAddr;
use spin::Mutex;

use crate::DebugConsole;

const UART_BASE: PhysAddr = PhysAddr::new(0x0900_0000);
// actually 7e201000
// const UART_BASE: PhysAddr = PhysAddr::new(0x3F201000);

static UART: Mutex<Pl011Uart> = Mutex::new(Pl011Uart::new(UART_BASE.mapped_vaddr().get_mut_ptr()));

// Initialize the UART
polyhal2_boot::ph_ctor!(UART_INIT, || UART.lock().init());

impl DebugConsole {
    /// Writes a byte to the console.
    pub fn putchar(c: u8) {
        let mut uart = UART.lock();
        match c {
            b'\n' => {
                uart.putchar(b'\r');
                uart.putchar(b'\n');
            }
            c => uart.putchar(c),
        }
    }

    /// Reads a byte from the console, or returns [`None`] if no input is available.
    pub fn getchar() -> Option<u8> {
        UART.lock().getchar()
    }
}
