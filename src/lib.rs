#![feature(no_std, lang_items, asm)]
#![feature(const_fn, unique, core_str_ext)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod vga;

mod cpuio;

#[no_mangle]
pub extern fn rust_main() {
    // ATTENTION: we have a very small stack and no guard page
    vga::clear_screen();
    println!("Hello World{}", "!");
    print!("Hello World{}", "!");

    loop{}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {loop{}}
