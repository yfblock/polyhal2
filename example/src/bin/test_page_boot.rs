#![no_std]
#![no_main]
#![feature(used_with_arg)]

use core::panic::PanicInfo;

use polyhal2::pagetable;
use polyhal2_debug::{DebugConsole, println};
extern crate polyhal2_debug;

polyhal2::boot::ph_ctor!(TEST_CTOR, || DebugConsole::putchar(b'3'));
fn main(_hart_id: usize) {
    println!("Entering kernel ...");
    println!("Hello World!");
    DebugConsole::putchar(b'5');
    loop {}
}

#[panic_handler]
fn panic_handler(message: &PanicInfo) -> ! {
    loop {}
}

// Specific a boot function and the size of the boot_stack
polyhal2::boot::entry_point!(main, 0x5000);
