use polyhal2_core::{addr::PhysAddr, bit, consts::KERNEL_OFFSET};
use riscv::{
    asm::wfi,
    register::{sie, sstatus},
};

use crate::{
    console::{display_basic, display_end},
    display_info,
};

/// Assembly Entry Function
///
/// Initialize Stack, Page Table and call rust entry.
#[naked]
#[unsafe(export_name = "_start")]
unsafe extern "C" fn _start() -> ! {
    unsafe {
        core::arch::naked_asm!(
            // 1. Set Stack Pointer.
            // sp = bootstack + (hartid + 1) * 0x10000
            "
                mv      t2, a0
                mv      t3, a1
                la      sp, bstack_top
                la      a0, boot_page
                call    {init_vspace}
                li      s0, {offset}   // add virtual address
                or      sp, sp, s0
            ",
            // 2. Open Paging Mode
            // satp = (8 << 60) | PPN(page_table)
            "
                la      t0, boot_page
                srli    t0, t0, 12
                li      t1, 8 << 60
                or      t0, t0, t1
                csrw    satp, t0
                sfence.vma
            ",
            // 3. Call rust_main function.
            "
                mv      a0, t2
                mv      a1, t3
                la      a2, {entry}
                or      a2, a2, s0
                jalr    a2                      // call rust_main
            ",
            entry = sym rust_main,
            init_vspace = sym init_vspace,
            offset = const KERNEL_OFFSET,
        )
    }
}

/// Assembly Entry Function
///
/// Initialize Page Information. Call rust_secondary_main entry function.
#[naked]
#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn secondary_start() -> ! {
    unsafe {
        core::arch::naked_asm!(
            // 1. Set Stack Pointer.
            // sp = a1(given Stack Pointer.)
            "
                mv      s6, a0
                mv      sp, a1
    
                li      s0, {offset}   // add virtual address
                or      sp, sp, s0
            ",
            // 2. Call Paging Mode
            // satp = (8 << 60) | PPN(page_table)
            "
                la      t0, boot_page
                srli    t0, t0, 12
                li      t1, 8 << 60
                or      t0, t0, t1
                csrw    satp, t0
                sfence.vma
            ", 
            // 3. Call secondary_entry
            "
                la      a2, {entry}
                or      a2, a2, s0
                mv      a0, s6
                jalr    a2                      // call rust_main
            ",
            entry = sym rust_secondary_main,
            offset = const KERNEL_OFFSET,
        );
    }
}

pub(crate) fn rust_main(hartid: usize, dtb: usize) {
    // Initialize CPU Configuration.
    init_cpu();
    crate::trap::riscv64::init();

    crate::ph_init_iter().for_each(|phw| (phw.func)());
    display_info!("DTB PTR", "{:#X}", dtb);
    polyhal2_device::init_dtb(PhysAddr::new(dtb));
    // Display Information.
    display_basic();
    display_info!();
    display_info!("Boot HART ID", "{}", hartid);
    display_end();

    super::call_rust_main(hartid);
}

/// Secondary Main function Entry.
///
/// Supports MultiCore, Boot in this function.
pub(crate) extern "C" fn rust_secondary_main(hartid: usize) {
    // TODO: Get the hart_id and device_tree for the specified device.
    // let (hartid, _device_tree) = boards::init_device(hartid, 0);

    // Initialize CPU Configuration.
    init_cpu();

    super::call_rust_main(hartid);
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe fn init_vspace(vspace: *mut usize) {
    for i in 0..0x100 {
        // FLAGS: Valid | Read | Write | eXecute | Access | Dirty
        const FLAGS: usize = bit!(0) | bit!(1) | bit!(2) | bit!(3) | bit!(6) | bit!(7);
        *vspace.add(i) = ((i * 0x4000_0000) >> 2) | FLAGS;
    }
}

#[inline]
fn init_cpu() {
    unsafe {
        // Enable SUM for access user memory directly.
        // TODO: Call set_sum() for riscv version up than 1.0, Close when below 1.0
        sstatus::set_sum();
        // Open float point support.
        sstatus::set_fs(sstatus::FS::Dirty);
        sie::set_sext();
        sie::set_ssoft();
    }
}

/// enter low cost area, loop until shutdown.
pub fn hlt_forever() -> ! {
    loop {
        wfi();
    }
}
