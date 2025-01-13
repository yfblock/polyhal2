#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "loongarch64.rs")]
mod imp;

use core::arch::global_asm;
use polyhal2_core::consts::PAGE_SIZE;

pub use imp::hlt_forever;

fn call_rust_main(hart_id: usize) -> ! {
    // Call rust main function.
    unsafe { crate::__polyhal_real_entry(hart_id) };
    hlt_forever()
}

// Map all memory to the page using 1GB Huge Page.
global_asm!(
    "
    .section .data
    .p2align 12
    .global boot_page
    boot_page:
        .fill {PAGE_SIZE} * {BOOT_PAGES}
", PAGE_SIZE = const PAGE_SIZE, BOOT_PAGES = const (get_boot_pages()) );

/// Get the boot pages number
const fn get_boot_pages() -> usize {
    match polyhal2_pagetable::VSpace::PAGE_LEVEL {
        // X86_64 will use 2 boot pages
        4 => 2,
        // riscv64 and aarch64 will use 1 boot pages
        3 => 1,
        _ => panic!("Unsupported page level"),
    }
}
