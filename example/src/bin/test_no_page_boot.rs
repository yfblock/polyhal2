#![no_std]
#![no_main]
#![feature(used_with_arg)]

use log::LevelFilter;
use polyhal2_debug::println;
use test_boot::log_impl::LogImpl;
extern crate polyhal2_debug;
extern crate test_boot;

polyhal2::boot::ph_ctor!(INIT_LOG, || {
    log::set_logger(&LogImpl).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Debug,
    });
});

fn main(_hart_id: usize) {
    println!("Entering kernel ...");
    println!("Hello World!");
    log::debug!("Test kernel Logging");
}

// Specific a boot function and the size of the boot_stack
polyhal2::boot::entry_point!(main, 0x5000);
