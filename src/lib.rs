#![feature(no_std, lang_items, asm)]
#![feature(const_fn, unique, core_str_ext)]
#![no_std]

extern crate rlibc;
extern crate spin;

#[macro_use]
mod vga;

mod cpuio;
mod multiboot2;

#[no_mangle]
pub extern fn rust_main(multiboot_addr: usize) {
    // ATTENTION: we have a very small stack and no guard page
    vga::clear_screen();
    
    let boot_info = unsafe { multiboot2::load(multiboot_addr) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    
    println!("Memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("  start 0x{:x}, length: 0x{:x}", area.base_addr, area.length);
    }
    
    

    loop{}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern fn panic_fmt() -> ! {loop{}}
