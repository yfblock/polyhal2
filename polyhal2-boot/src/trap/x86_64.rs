use core::arch::global_asm;

use x86_64::{
    registers::model_specific::LStar,
    structures::idt::{Entry, HandlerFunc, InterruptDescriptorTable},
};

const NUM_INT: usize = 256;
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

global_asm!(
    r#"
.equ NUM_INT, {num_int}
.altmacro
.macro DEF_HANDLER, i
.Ltrap_handler_\i:
.if \i == 8 || (\i >= 10 && \i <= 14) || \i == 17
    # error code pushed by CPU
    pop     rsi
    mov     rdi, \i          # interrupt vector
    pop     rdx
    jmp     {trap_handler}
.else
    mov     rsi, 0           # fill in error code in TrapFrame
    mov     rdi, \i          # interrupt vector
    pop     rdx
    jmp     {trap_handler}
.endif
.endm

.macro DEF_TABLE_ENTRY, i
    .quad .Ltrap_handler_\i
.endm

.section .text
.code64
_trap_handlers:
.set i, 0
.rept NUM_INT
    DEF_HANDLER %i
    .set i, i + 1
.endr

.section .rodata
.global trap_handler_table
trap_handler_table:
.set i, 0
.rept NUM_INT
    DEF_TABLE_ENTRY %i
    .set i, i + 1
.endr
"#,
    trap_handler = sym trap_handler,
    num_int = const NUM_INT
);

unsafe extern "C" fn trap_handler(vector: usize, error_code: usize, rip: usize) -> ! {
    panic!(
        "Unhandled Trap @ ip: {:#x}, vector: {:#x?} error_code: {:#x}",
        rip, vector, error_code
    )
}

#[allow(static_mut_refs)]
pub(crate) fn init() {
    unsafe extern "C" {
        #[link_name = "trap_handler_table"]
        static ENTRIES: [extern "C" fn(); NUM_INT];
    }
    LStar::write(x86_64::VirtAddr::new(trap_handler as _));
    unsafe {
        let entries =
            core::slice::from_raw_parts_mut(&mut IDT as *mut _ as *mut Entry<HandlerFunc>, NUM_INT);
        for i in 0..NUM_INT {
            entries[i].set_handler_fn(core::mem::transmute(ENTRIES[i]));
        }
        IDT.load();
    };
}
