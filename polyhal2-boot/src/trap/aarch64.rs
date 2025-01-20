use core::arch::global_asm;

use aarch64_cpu::registers::{ELR_EL1, ESR_EL1, Readable, VBAR_EL1, Writeable};

global_asm!("
.macro INVALID_EXCP, kind, source
.p2align 7
    mov     x0, \\kind
    mov     x1, \\source
    bl      {trap_handler}
.endm

.section .text
.p2align 12
exception_vector_base:
    // current EL, with SP_EL0
    INVALID_EXCP 0 0
    INVALID_EXCP 1 0
    INVALID_EXCP 2 0
    INVALID_EXCP 3 0

    // current EL, with SP_ELx
    INVALID_EXCP 0 1
    INVALID_EXCP 1 1
    INVALID_EXCP 2 1
    INVALID_EXCP 3 1
    ",
    trap_handler = sym trap_handler
);

unsafe extern "C" fn trap_handler(kind: usize, source: usize) -> ! {
    let esr = ESR_EL1.extract();
    let elr = ELR_EL1.extract();
    panic!(
        "Unhandled Trap @ SP_EL{source}, kind: {:#x}, esr: {:?} {{{:#x}}}, elr: {:#x}",
        kind,
        esr.read_as_enum::<ESR_EL1::EC::Value>(ESR_EL1::EC),
        esr.get(),
        elr.get()
    )
}

pub(crate) fn init() {
    unsafe extern "C" {
        fn exception_vector_base();
    }
    VBAR_EL1.set(exception_vector_base as _);
}
