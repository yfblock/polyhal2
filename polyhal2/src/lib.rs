//! The collection of the polyhal2's plugins
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

pub use polyhal2_base as base;
#[cfg(feature = "boot")]
pub use polyhal2_boot as boot;
#[cfg(feature = "pagetable")]
pub use polyhal2_pagetable as pagetable;
