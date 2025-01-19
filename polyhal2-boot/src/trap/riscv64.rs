use riscv::{
    ExceptionNumber, InterruptNumber,
    interrupt::supervisor::{Exception, Interrupt},
    register::{scause, sepc, sstatus, stvec},
};

unsafe extern "C" fn trap_handler() -> ! {
    unsafe { core::arch::asm!(".p2align 2") };
    match scause::read().cause() {
        scause::Trap::Interrupt(n) => panic!(
            "Unhandled Trap @ ip: {:#x}, scause: {:x?} {{ bits: {:#x} }}, sstatus: {:x?}",
            sepc::read(),
            Interrupt::from_number(n).unwrap(),
            n,
            sstatus::read()
        ),
        scause::Trap::Exception(n) => panic!(
            "Unhandled Trap @ ip: {:#x}, scause: {:x?} {{ bits: {:#x} }}, sstatus: {:x?}",
            sepc::read(),
            Exception::from_number(n).unwrap(),
            n,
            sstatus::read()
        ),
    }
}

pub(crate) fn init() {
    unsafe {
        stvec::write(trap_handler as _, stvec::TrapMode::Direct);
    }
}
