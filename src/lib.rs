#![feature(lang_items, asm, step_by)]
#![feature(const_fn, unique)]
#![feature(associated_type_defaults)]
#![no_std]
#![allow(dead_code)]

extern crate rlibc;
extern crate spin;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate x86;

#[macro_use]
mod vga;

mod multiboot2;
mod memory;
pub mod arch;

fn print_kernel_sections(boot_info: &multiboot2::BootInformation) {
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
    println!("Kernel sections:");
    for (idx, section) in elf_sections_tag.sections().enumerate() {
        println!("  {} addr: 0x{:x}, size 0x{:x}, flags: 0x{:x}",
                 idx,
                 section.addr,
                 section.size,
                 section.flags);
    }
}

fn print_memory_areas(memory_map_tag: &multiboot2::MemoryMapTag) {
    println!("Memory areas:");
    for area in memory_map_tag.memory_areas() {
        println!("  start 0x{:x}, length: 0x{:x}",
                 area.base_addr,
                 area.length);
    }
}

fn test_mapping() {
    unsafe {
        arch::mm::map_to(0xFFFFFFF800000000, 0x200000);

        let ptr: *mut u64 = 0xFFFFFFF800000000 as *mut u64;

        *ptr = 33;

        println!("val: {}", *ptr);

        arch::mm::unmap(0xFFFFFFF800000000);
    }
}

#[no_mangle]
pub extern "C" fn rust_main(multiboot_addr: usize) {
    vga::clear_screen();
    println!("test 0x{:x}", arch::mm::phys_to_kern(multiboot_addr));

    //let multiboot_addr = multiboot_addr + 0xffff_8000_0000_0000;

    let boot_info = unsafe { multiboot2::load(multiboot_addr) };
    let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");
    let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");

    print_memory_areas(memory_map_tag);

    print_kernel_sections(boot_info);

    let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
    let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

    let multiboot_start = multiboot_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    println!("kernel_start: 0x{:x}, kernel_end: 0x{:x}",
             kernel_start,
             kernel_end);
    println!("multiboot_start: 0x{:x}, multiboot_end: 0x{:x}",
             multiboot_start,
             multiboot_end);

    memory::init(kernel_start as usize,
                 kernel_end as usize,
                 multiboot_start as usize,
                 multiboot_end as usize,
                 memory_map_tag.memory_areas());

    arch::mm::init();
    arch::acpi::init();
    arch::interrupts::init();

    test_mapping();

    println!("KERNEL END");

    unsafe {
        loop {
            asm!("hlt");
        }
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[lang = "panic_fmt"]
extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);

    loop {}
}
