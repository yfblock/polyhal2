//! polyhal2 Boot Crate
//!
//! This crate provides a environment to help you
//! boot your kernel.
#![no_std]
// #![deny(warnings)]
#![deny(missing_docs)]
#![feature(linkage)]
#![feature(used_with_arg)]
#![feature(naked_functions)]

use core::slice::Iter;

/// EntryPoint per architecture
mod entry;

/// Input and output function
pub mod console;
/// The helpful macros.
pub mod macros;

mod panic;

unsafe extern "Rust" {
    /// The real entry of the program
    pub fn __polyhal_real_entry(hart_id: usize);
    /// The start symbol of the init section
    pub fn __start_ph_init();
    /// The stop symbol of the init section
    pub fn __stop_ph_init();
    /// Put a charactor to console
    #[linkage = "extern_weak"]
    pub fn __polyhal_putchar(c: u8);
}

/// PolyHAL's Initialize Wrapper
///
/// This struct contians' constructor function
/// and its priority.
/// The lower the priority, the earlier it will be called
pub struct PHInitWrap {
    /// The priority of the init function
    pub _priority: usize,
    /// The Initialize function
    pub func: fn(),
}

/// Polyhal Constructor placeholder
#[used(linker)]
#[unsafe(link_section = "ph_init")]
static PH_INIT_ARR: [fn(); 0] = [];

/// Get a iterator of the polyhal init section.
///
/// The item of the iterator is function reference.
///
/// ## Demo
///
/// ```rust
/// // Call all initialize function.
/// ph_init_iter().for_each(|f| f());
/// ```
fn ph_init_iter<'a>() -> Iter<'a, PHInitWrap> {
    let len = (__stop_ph_init as usize - __start_ph_init as usize) / size_of::<PHInitWrap>();
    unsafe { core::slice::from_raw_parts_mut(__start_ph_init as *mut PHInitWrap, len).iter() }
}

/// Weak function
/// Put a character to console
#[linkage = "weak"]
#[unsafe(export_name = "__polyhal_putchar")]
fn putc(_c: u8) {}
