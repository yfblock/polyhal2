use polyhal2_base::consts::KERNEL_OFFSET;

use crate::display_info;
use core::fmt::{Arguments, Write};

/// A Struct implement the Writer
pub struct WriterImpl;

impl Write for WriterImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .into_iter()
            .for_each(|c| unsafe { super::__polyhal_putchar(*c) });
        Ok(())
    }
}

/// Write arguments to console
#[doc(hidden)]
pub fn _print(args: Arguments) {
    write!(WriterImpl, "{}", args).unwrap();
}

/// Display the basic info
/// Includes title banner
pub(crate) fn display_basic() {
    display_info!(
        r#"
  _____      _       _    _          _      
 |  __ \    | |     | |  | |   /\   | |     
 | |__) |__ | |_   _| |__| |  /  \  | |     
 |  ___/ _ \| | | | |  __  | / /\ \ | |     
 | |  | (_) | | |_| | |  | |/ ____ \| |____ 
 |_|   \___/|_|\__, |_|  |_/_/    \_\______|
                __/ |                       
               |___/                        "#
    );
    display_info!("Platform ABI", "{}", env!("BUILD_ABI"));
    display_info!("Platform Architecture", "{}", env!("BUILD_TARGET"));
}

/// Display the information before entering kernel
pub(crate) fn display_end() {
    display_info!();
    display_info!("Kernel Offset", "{:#p}", KERNEL_OFFSET as *const u8);
    display_info!("Kernel EntryPoint", "{:#p}", super::__polyhal_real_entry as *const u8);
    display_info!();
}


/// Display Platform Information with specified format
/// display_info!("item name", "{}", "format");
/// The output format like below:
/// item name             : format
#[macro_export]
macro_rules! display_info{
    () => {
        $crate::console::_print(format_args!("\n"))
    };
    ($fmt:literal) => {
        $crate::console::_print(format_args!("{}\n", $fmt));
    };
    ($item:expr,$fmt: expr $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!("{:<26}: {}\n", $item, format_args!($fmt $(, $($arg)+)?)))
    };
}
