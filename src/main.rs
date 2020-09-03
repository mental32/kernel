#![no_main]
#![no_std]
#![feature(lang_items)]

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

extern "C" fn kinit() {
    loop {}
}
