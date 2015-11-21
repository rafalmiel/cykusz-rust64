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
    
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    println!("Kernel sections:");
    for section in elf_sections_tag.sections() {
        println!("  addr: 0x{:x}, size 0x{:x}, flags: 0x{:x}",
            section.addr, section.size, section.flags);
    }
    
    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();
    
    let multiboot_start = multiboot_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);
    
    println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}", 
        kernel_start, kernel_end);
    println!("multiboot_start: 0x{:x}, multiboot_end: 0x{:x}", 
        multiboot_start, multiboot_end);
    
    //panic!("HEHE");

    loop{}
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    
    loop{}
}
