/// Definition the entry point of the program
///
/// ## Demo
///
/// fn main(hart) {
///     println!("This is main function");
/// }
///
/// entry_point!(main, 0x1000)
#[macro_export]
macro_rules! entry_point {
    ($entry:ident, $boot_stack:literal) => {
        #[unsafe(no_mangle)]
        unsafe fn __polyhal_real_entry(hart_id: usize) {
            $entry(hart_id as _);
        }
        /// Definition a boot stack through `global_asm!`
        core::arch::global_asm!(concat!(
            "
            .section .bss
            .global bstack_top
            bstack_bottom:
            .fill ",
            $boot_stack,
            "
            bstack_top:
        "
        ));
    };
}

/// Specific uart interface.
///
///
/// ## Demo
///
/// fn put_char(c: u8) {
///     todo!("putchar")
/// }
///
/// fn get_char() -> u8 {
///     todo!("getchar")
/// }
///
/// uart_interface!(put_char, get_char);
///
#[macro_export]
macro_rules! uart_interface {
    ($putc:expr, $getc: expr) => {
        #[unsafe(no_mangle)]
        fn __polyhal_putchar(c: u8) {
            $putc(c);
        }
    };
}

/// Definiation a constructer
///
/// This constructor will be called by polyhal when booting.
/// Please add `#![feature(used_with_arg)]` at the top of your `lib.rs` file.
///
/// ## Demo
///
/// ```rust
/// ph_ctor!(ctor_name, || {
///     // Ctor block
/// });
/// ```
#[macro_export]
macro_rules! ph_ctor {
    ($name:ident, $f:expr) => {
        #[used(linker)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = "ph_init")]
        static $name: $crate::PHInitWrap = $crate::PHInitWrap {
            _priority: 0,
            func: $f,
        };
    };
}
