//! This crate support parse device tree and pci
//! and get its details.
//!
//! Additionally, it support init and assign pci
//! device if it not initialized. (For loongarch64)
//!
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

use core::sync::atomic::{AtomicUsize, Ordering};

use polyhal2_core::addr::PhysAddr;

static DTB_PTR: AtomicUsize = AtomicUsize::new(0);

/// Initialize with specific device tree binary
pub fn init_dtb(dtb_ptr: PhysAddr) -> Option<()> {
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb_ptr.mapped_vaddr().get_ptr()).ok()? };

    // Initialize the device tree pointer
    DTB_PTR.store(dtb_ptr.raw(), Ordering::SeqCst);

    log::debug!("memory count: {}", fdt.memory_reservations().count());
    fdt.memory().regions().for_each(|mr| {
        log::debug!("memory: {:#x?}", mr);
    });
    fdt.memory_reservations()
        .for_each(|m| log::debug!("memory: {:#x?}", m));
    log::debug!("{:#x?}", fdt.chosen().bootargs());
    if let Some(stdout) = fdt.chosen().stdout() {
        log::debug!("stdout: {:#x?}", stdout.name);
        log::debug!("stdout: {:#x?}", stdout.property("reg").unwrap().name);
    }

    Some(())
}

/// Get the pointer to the device Address
pub fn get_dtb_ptr() -> PhysAddr {
    PhysAddr::new(DTB_PTR.load(Ordering::SeqCst))
}
