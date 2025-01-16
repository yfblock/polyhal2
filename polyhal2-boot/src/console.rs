use polyhal2_core::consts::KERNEL_OFFSET;

use crate::display_info;
use core::fmt::{Arguments, Write};

/// A Struct implement the Writer
pub struct WriterImpl;

impl Write for WriterImpl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        s.as_bytes()
            .iter()
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
    #[cfg(not(target_arch = "x86_64"))]
    display_info!(
        "Platform DTB Pointer",
        "{:#018x}",
        polyhal2_device::get_dtb_ptr().raw()
    );
}

/// Display the information before entering kernel
pub(crate) fn display_end() {
    unsafe extern "C" {
        fn bstack_top();
    }
    display_info!("Boot Stack Top", "{:#p}", bstack_top as *const u8);
    display_info!();
    display_info!("Kernel Offset", "{:#p}", KERNEL_OFFSET as *const u8);
    display_info!(
        "Kernel EntryPoint",
        "{:#p}",
        super::__polyhal_real_entry as *const u8
    );
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
