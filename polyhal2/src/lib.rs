//! The collection of the polyhal2's plugins
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

#[cfg(feature = "boot")]
pub use polyhal2_boot as boot;
pub use polyhal2_core as core;
#[cfg(feature = "pagetable")]
pub use polyhal2_pagetable as pagetable;
