use loongArch64::register::euen;
use polyhal2_core::addr::PhysAddr;

use crate::{
    console::{display_basic, display_end},
    display_info,
};

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn _start() -> ! {
    unsafe {
        core::arch::naked_asm!("
            ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
            lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
            csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
            ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
            lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
            csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1
        ",
        // Enable Paging Mode
        // TODO: Enable if need to enable paging mode
        "
            b           1f
            # Enable PG 
            li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
            csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
            li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
            csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
            li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
            csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN
        ",
        // Init Stack and jump to main function
        "1:
            la.global   $sp, bstack_top

            csrrd       $a0, 0x20           # cpuid
            la.global   $t0, {entry}
            jirl        $zero,$t0,0
        ",
            entry = sym rust_tmp_main,
        )
    }
}

/// enter low cost area, loop until shutdown.
pub fn hlt_forever() -> ! {
    loop {
        unsafe { loongArch64::asm::idle() };
    }
}

/// Rust temporary entry point
///
/// This function will be called after assembly boot stage.
pub fn rust_tmp_main(hart_id: usize) {
    // Initialize CPU Configuration.
    init_cpu();
    crate::trap::loongarch64::init();

    crate::ph_init_iter().for_each(|phw| (phw.func)());
    // FIXME: Make this statement more efficient
    polyhal2_device::init_dtb(PhysAddr::new(0x100000));

    // Display Information.
    display_basic();
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_end();

    super::call_rust_main(hart_id);
}

/// Initialize CPU Configuration.
fn init_cpu() {
    // Enable floating point
    euen::set_fpe(true);

    // TODO: Init trap if needed.
}
