#![no_std]
#![no_main]

#[panic_handler]
fn panic_h(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_kmain() -> ! {
    loop {}
}
