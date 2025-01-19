use loongArch64::register::{ecfg, eentry, era, estat};

unsafe extern "C" fn trap_vector_base() {
    // Align function with 4096(2^12)
    unsafe { core::arch::asm!(".p2align 12") };
    panic!(
        "Unhandled Trap @ ip: {:#x}, estat: {:x?}{{ bits: {:#x} }}",
        era::read().raw(),
        estat::read().cause(),
        estat::read().raw()
    )
}

pub(crate) fn init() {
    ecfg::set_vs(0);
    eentry::set_eentry(trap_vector_base as usize);
}
