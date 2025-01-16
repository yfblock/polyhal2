#![no_std]
#![feature(linkage)]
#![feature(used_with_arg)]

use core::fmt::{Arguments, Write};

use polyhal2_boot::uart_interface;

#[cfg_attr(target_arch = "aarch64", path = "arch/aarch64.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "arch/loongarch64.rs")]
#[cfg_attr(target_arch = "x86_64", path = "arch/x86_64.rs")]
pub mod arch;

pub struct DebugConsole;

uart_interface!(DebugConsole::putchar, DebugConsole::getchar);

impl Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes().iter().for_each(|c| DebugConsole::putchar(*c));
        Ok(())
    }
}

/// Write arguments to console
#[doc(hidden)]
pub fn _print(args: Arguments) {
    write!(DebugConsole, "{}", args).unwrap();
}

/// A macro to print
///
/// # Demo
///
/// print!("Hello World!\n");
#[macro_export]
macro_rules! print {
    ($($args:expr),*) => {
        $crate::_print(format_args!($($args),*));
    };
}

/// A macro to print
///
/// # Demo
///
/// println!("Hello World!");
#[macro_export]
macro_rules! println {
    ($($args:expr),*) => {
        $crate::_print(format_args!("{}\n", format_args!($($args),*)));
    };
}
