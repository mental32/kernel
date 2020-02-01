#![feature(lang_items)]
#![no_std]

use vga::{VGAWriter, vprintln};


#[no_mangle]
pub extern "C" fn kmain(_multiboot_addr: usize) {
    let mut writer = VGAWriter::default();
    vprintln!(writer, "{}", "Hello, world! ~ fart-joke");
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle] 
pub extern fn eh_personality() {}
