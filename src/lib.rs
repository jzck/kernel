//! project hosted on [github](https://github.com/jzck/kernel)

#![no_std]
#![feature(lang_items)]
#![feature(const_fn)]
#![feature(ptr_internals)]
#![feature(asm)]

extern crate rlibc;
extern crate multiboot2;        //slightly modified fork from official 0.3.2
#[macro_use] extern crate bitflags;

/// 80x25 screen and simplistic terminal driver
#[macro_use] pub mod vga;
/// kernel init and environment
pub mod context;
/// PS/2 detection and processing
pub mod keyboard;
/// simplisitc kernel commands
pub mod console;
/// wrappers around the x86-family I/O instructions.
pub mod cpuio;
/// ACPI self-content module
pub mod acpi;
/// physical frame allocator + paging module
pub mod memory;
/// a few x86 instruction wrappers
pub mod x86;

#[no_mangle]
pub extern fn kmain(multiboot_info_addr: usize) -> ! {
    context::init(multiboot_info_addr);
    loop {}
    acpi::init().unwrap();
    loop { keyboard::kbd_callback(); }
}

#[lang = "eh_personality"] #[no_mangle]
pub extern fn eh_personality() {

}

#[lang = "panic_fmt"] #[no_mangle]
pub extern fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32)
-> ! {
    println!("PANIC: {}", fmt);
    println!("FILE: {}", file);
    println!("LINE: {}", line);
    flush!();
    loop {}

}

