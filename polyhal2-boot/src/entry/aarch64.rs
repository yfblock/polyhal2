use aarch64_cpu::asm::barrier;
use aarch64_cpu::registers::{
    CurrentEL, MAIR_EL1, ReadWriteable, Readable, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1,
    Writeable,
};
use polyhal2_core::addr::{PhysAddr, VirtAddr};
use polyhal2_core::bit;
use polyhal2_core::consts::KERNEL_OFFSET;

use crate::console::{display_basic, display_end};
use crate::display_info;

/// The earliest entry point for the primary CPU.
#[naked]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn _start() -> ! {
    // PC = 0x8_0000
    // X0 = dtb
    unsafe {
        core::arch::naked_asm!("
            mrs     x19, mpidr_el1
            and     x19, x19, #0xffffff     // get current CPU id
            mov     x20, x0                 // save DTB pointer
            cbz     x19, 1f
            b       .
        1:
            adrp    x8, bstack_top
            add     x8, x8, :lo12:bstack_top
            mov     sp, x8
        ",
        // Enable Paging Mode
        "
            adrp    x0, boot_page
            bl      {init_mmu}              // setup MMU
            mov     x8, {KERNEL_OFFSET}     // set SP to the high address
            add     sp, sp, x8
        ",
        // Init boot Stack and call main function
        "
            mov     x0, x19                 // call rust_entry(cpu_id, dtb)
            mov     x1, x20
            ldr     x8, ={entry}
            blr     x8
        ",
            init_mmu = sym init_mmu,
            KERNEL_OFFSET = const polyhal2_core::consts::KERNEL_OFFSET,
            entry = sym rust_tmp_main,
        )
    }
}

/// enter low cost area, loop until shutdown.
pub fn hlt_forever() -> ! {
    loop {
        aarch64_cpu::asm::wfi();
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_mmu(mut root_paddr: usize) {
    MAIR_EL1.set(0x44_ff_04);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 39 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(25);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(25);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    if root_paddr > KERNEL_OFFSET {
        root_paddr -= KERNEL_OFFSET;
    }
    // Mapping all physical addresses.
    let ptr = VirtAddr::new(root_paddr).get_mut_ptr::<usize>();
    for idx in 0..512 {
        // FLAGS: VALID | AF(AccessFlag)
        const FLAGS: usize = bit!(0) | bit!(10);
        ptr.add(idx).write_volatile((idx * 0x4000_0000) | FLAGS);
    }

    TTBR0_EL1.set(root_paddr as _);
    TTBR1_EL1.set(root_paddr as _);
    // Flush the entire TLB
    core::arch::asm!("tlbi vmalle1; dsb sy; isb");

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

/// Rust Temporary Entry
unsafe fn rust_tmp_main(hart_id: usize, dtb: usize) {
    crate::trap::aarch64::init();
    // Initialize all constructor functions.
    crate::ph_init_iter().for_each(|phw| (phw.func)());
    polyhal2_device::init_dtb(PhysAddr::new(dtb));
    display_basic();
    display_info!("Platform CurrentEL", "{}", CurrentEL.read(CurrentEL::EL));
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_end();

    // Call rust main function.
    super::call_rust_main(hart_id);
}
