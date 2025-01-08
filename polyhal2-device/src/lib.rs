//! This crate support parse device tree and pci
//! and get its details.
//! 
//! Additionally, it support init and assign pci 
//! device if it not initialized. (For loongarch64)
//! 
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

use polyhal2_base::consts::KERNEL_OFFSET;

/// Initialize with specific device tree binary
pub fn init_dtb(dtb_ptr: *const u8) -> Option<()> {
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb_ptr.add(KERNEL_OFFSET)).ok()? };
    // log::debug!("{:#x?}", fdt.mem);
    log::debug!("memory count: {}", fdt.memory_reservations().count());
    fdt.memory().regions().for_each(|mr| {
        log::debug!("memory: {:#x?}", mr);
    });
    fdt.memory_reservations().for_each(|m| {
        log::debug!("memory: {:#x?}", m)
    });
    log::debug!("{:#x?}", fdt.chosen().bootargs());
    if let Some(stdout) = fdt.chosen().stdout() {
        log::debug!("stdout: {:#x?}", stdout.name);
        log::debug!("stdout: {:#x?}", stdout.property("reg").unwrap().name);
    }

    Some(())
}
