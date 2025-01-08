use core::arch::global_asm;

use aarch64_cpu::asm::{self, barrier};
use aarch64_cpu::registers::{
    CurrentEL, ReadWriteable, Readable, SPSel, Writeable, CNTHCTL_EL2, CNTVOFF_EL2, ELR_EL2, ELR_EL3, HCR_EL2, LR, MAIR_EL1, SCR_EL3, SCTLR_EL1, SPSR_EL2, SPSR_EL3, SP_EL0, SP_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1
};
use polyhal2_base::consts::KERNEL_OFFSET;
use polyhal2_pagetable::TLB;

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

            // mov     x0, sp
            // bl      {switch_to_el1}         // switch to EL1
            // adrp    x0, boot_page
            // bl      {init_mmu}              // setup MMU

            mov     x8, {KERNEL_OFFSET}     // set SP to the high address
            add     sp, sp, x8

            mov     x0, x19                 // call rust_entry(cpu_id, dtb)
            mov     x1, x20
            mov     x2, sp
            ldr     x8, ={entry}
            blr     x8
            b      .",
            switch_to_el1 = sym switch_to_el1,
            init_mmu = sym init_mmu,
            KERNEL_OFFSET = const polyhal2_base::consts::KERNEL_OFFSET,
            entry = sym rust_tmp_main,
        )
    }
}

/// Drop currentEL to el1
///
unsafe fn switch_to_el1(sp: u64) {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el < 2 {
        return;
    }
    if current_el == 3 {
        // Set EL2 to 64bit and enable the HVC instruction.
        SCR_EL3.write(
            SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
        );
        // Set the return address and exception level.
        SPSR_EL3.write(
            SPSR_EL3::M::EL1h
                + SPSR_EL3::D::Masked
                + SPSR_EL3::A::Masked
                + SPSR_EL3::I::Masked
                + SPSR_EL3::F::Masked,
        );
        ELR_EL3.set(LR.get());
    }
    // Disable EL1 timer traps and the timer offset.
    CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
    CNTVOFF_EL2.set(0);
    // Set EL1 to 64bit.
    HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
    // Set the return address and exception level.
    SPSR_EL2.write(
        SPSR_EL2::M::EL1h
            + SPSR_EL2::D::Masked
            + SPSR_EL2::A::Masked
            + SPSR_EL2::I::Masked
            + SPSR_EL2::F::Masked,
    );
    SP_EL1.set(sp);
    ELR_EL2.set(LR.get());
    asm::eret();
}

// Map all memory to the page using 1GB Huge Page.
global_asm!(
    "
    .section .data
    .p2align 12
    boot_page:
    .set    n, 0
    .rept   512
        // PTEFlags::VALID | PTEFlags::ATTR_INDX | PTEFlags::AF
        .quad n | (1 << 0) | (0b111 << 2) | (1 << 10)
    .set    n, n + 0x40000000
    .endr
"
);

unsafe fn init_mmu(mut root_paddr: u64) {
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
    if root_paddr > KERNEL_OFFSET as _ {
        root_paddr = root_paddr - KERNEL_OFFSET as u64;
    }
    TTBR0_EL1.set(root_paddr);
    TTBR1_EL1.set(root_paddr);
    // Flush the entire TLB
    TLB::flush_all();

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

/// Rust Temporary Entry
unsafe fn rust_tmp_main(hart_id: usize, dtb: *const u8, boot_stack: usize) {
    // Initialize all constructor functions.
    super::ph_init_iter().for_each(|phw| (phw.func)());
    polyhal2_device::init_dtb(dtb);
    display_basic();
    display_info!("Platform CurrentEL", "{}", CurrentEL.read(CurrentEL::EL));
    display_info!("Platfoem Device Tree", "{:#p}", dtb);
    display_info!();
    display_info!("LR value", "{:#x}", LR.get());
    display_info!("PAN Reg", "{:#x}", SCTLR_EL1.get());
    display_info!("Boot Stack Pointer", "{:#p}", boot_stack as *const u8);
    display_end();
    unsafe {
        super::__polyhal_real_entry(hart_id);
    }
}
