use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(message: &PanicInfo) -> ! {
    log::error!("Panic Message: {}", message.message());
    crate::entry::hlt_forever()
}
