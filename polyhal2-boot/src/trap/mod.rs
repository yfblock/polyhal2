//! Trap Handler
//!
//! Default Trap Handler for PolyHAL Boot
//! Just panic when trap happened.
//!

#[cfg(target_arch = "aarch64")]
pub(crate) mod aarch64;
#[cfg(target_arch = "loongarch64")]
pub(crate) mod loongarch64;
#[cfg(target_arch = "riscv64")]
pub(crate) mod riscv64;
#[cfg(target_arch = "x86_64")]
pub(crate) mod x86_64;
