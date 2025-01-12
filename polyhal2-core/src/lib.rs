//! This is a basic crate only contains defination
//! and macros.
//! It provides support for upper level crate.
//!
//!

#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]
#![allow(unsafe_op_in_unsafe_fn)]

/// addr Module, contains address and page type
/// physical and virtual version exists
pub mod addr;
/// It contains the constant value
/// Some consts will be initialized when compiling
/// using (const fn) from_str_radix by passing env.
pub mod consts;
/// Provide LazyInit for initializing once.
pub mod lazy_init;
/// It contains macros like declare_env_var and so on.
pub mod macros;
