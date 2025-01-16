#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::hlt_forever;
#[cfg(target_arch = "loongarch64")]
mod loongarch64;
#[cfg(target_arch = "loongarch64")]
pub use loongarch64::hlt_forever;
#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::hlt_forever;

use core::arch::global_asm;
use polyhal2_core::consts::PAGE_SIZE;

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
    #[cfg(not(target_arch = "x86_64"))]
    match polyhal2_pagetable::VSpace::PAGE_LEVEL {
        // use 2 physical pages.
        4 => 2,
        // riscv64 and aarch64 will use 1 boot pages
        3 => 1,
        _ => panic!("Unsupported page level"),
    }
    // x86_64 use 514 physical pages (for not 1G huge page compatiable)
    #[cfg(target_arch = "x86_64")]
    514
}
