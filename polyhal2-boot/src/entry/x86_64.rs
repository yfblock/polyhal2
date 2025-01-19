mod mb_entry;

use core::{arch::global_asm, slice};

use mb_entry::use_multiboot;
use polyhal2_core::consts::KERNEL_OFFSET;
use x86_64::registers::control::{Cr0Flags, Cr4Flags, EferFlags};

use crate::{
    console::{display_basic, display_end},
    display_info,
};

/// CR0 Registers introduction: https://wiki.osdev.org/CPU_Registers_x86-64#CR0
const CR0: u64 = Cr0Flags::PROTECTED_MODE_ENABLE.bits()
    | Cr0Flags::MONITOR_COPROCESSOR.bits()
    | Cr0Flags::NUMERIC_ERROR.bits()
    | Cr0Flags::WRITE_PROTECT.bits()
    | Cr0Flags::PAGING.bits();

/// CR4 registers introduction: https://wiki.osdev.org/CPU_Registers_x86-64#CR4
/// Physical Address Extension
const CR4: u64 = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits()
    // Page Global Enable
    | Cr4Flags::PAGE_GLOBAL.bits()
    // OS support for fxsave and fxrstor instructions
    | Cr4Flags::OSFXSR.bits()
    // Add Support for 2M Huge Page Support.
    | Cr4Flags::PAGE_SIZE_EXTENSION.bits()
    // XSAVE And Processor Extended States Enable
    // This bit should open if the processor was supported.
    // | Cr4Flags::OSXSAVE.bits()
    // OS Support for unmasked simd floating point exceptions
    | Cr4Flags::OSXMMEXCPT_ENABLE.bits();

const IA32_EFER_NUM: u32 = 0xC0000080;
const EFER: u64 = EferFlags::LONG_MODE_ENABLE.bits();
global_asm!(
    include_str!("x86_64/entry.S"),
    entry = sym rust_tmp_main,

    offset = const KERNEL_OFFSET,

    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const IA32_EFER_NUM,
    efer = const EFER,
);

global_asm!(
    include_str!("x86_64/multiboot.S"),
    mb_hdr_magic = const (mb_entry::MULTIBOOT_HEADER_MAGIC),
    mb_hdr_flags = const (mb_entry::MULTIBOOT_HEADER_FLAGS),
    graphic_mode = const 0,
    offset = const (polyhal2_core::consts::KERNEL_OFFSET),
);

fn rust_tmp_main(magic: usize, mboot_ptr: u64) {
    // Initialize CPU Configuration.
    init_page_table();

    crate::ph_init_iter().for_each(|phw| (phw.func)());
    let hart_id = match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as _,
        None => 0,
    };
    // Display Information.
    display_basic();
    display_info!("Platform Multiboot Magic", "{:#x?}", magic);
    display_info!("Platform Multiboot", "{:#018x}", mboot_ptr);

    if let Some(mboot) = use_multiboot(mboot_ptr) {
        if let Some(regions) = mboot.memory_regions() {
            regions.for_each(|rg| {
                display_info!(
                    "Platform Memory Region",
                    "{:#018x} - {:#018x} {:?}",
                    rg.base_address(),
                    rg.base_address() + rg.length(),
                    rg.memory_type()
                );
            });
        }
    }
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_end();

    super::call_rust_main(hart_id);
}

/// enter low cost area, loop until shutdown.
pub fn hlt_forever() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Initialize Boot Page Table
fn init_page_table() {
    unsafe extern "C" {
        fn boot_page();
    }
    const PDPT_FLAGS: u64 = 0x3; // P | W
    const PD_FLAGS: u64 = 0x83; // P | W | HUGE
    unsafe {
        let pdpt_ptr = boot_page as usize + 0x1000;
        let pd_ptr = pdpt_ptr + 0x1000;

        // Initialize PDPT range 0x0000_0000_4000_0000 - 0x0000_007f_ffff_ffff
        let pdpt = slice::from_raw_parts_mut(pdpt_ptr as *mut u64, 512);
        for (i, pdpt_item) in pdpt.iter_mut().enumerate().take(512).skip(1) {
            *pdpt_item = (pd_ptr - KERNEL_OFFSET + i * 0x1000) as u64 + PDPT_FLAGS;
        }

        // Initalize PDPT range 0x0000_0000_4000_0000 - 0x0000_007f_ffff_ffff
        let pd = slice::from_raw_parts_mut(pd_ptr as *mut u64, 512 * 512);
        for (i, pd_item) in pd.iter_mut().enumerate().skip(512) {
            *pd_item = i as u64 * 0x200000 + PD_FLAGS;
        }
    }
}
