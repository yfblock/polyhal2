#[cfg_attr(target_arch = "aarch64", path = "aarch64.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "loongarch64.rs")]
mod imp;

pub use imp::hlt_forever;

fn call_rust_main(hart_id: usize) -> ! {
    // Call rust main function.
    unsafe { crate::__polyhal_real_entry(hart_id) };

    hlt_forever()
}
