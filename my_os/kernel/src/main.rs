#![no_std]
#![no_main]
#![allow(static_mut_refs)]

mod allocator;
mod fs;
mod interrupts;
mod io;
mod keyboard;
mod memory;
mod shell;
mod vga;

use core::panic::PanicInfo;

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    vga::clear_screen();
    println!("tiny64: Rust kernel entered long mode");
    println!("-----------------------------------");

    memory::init();
    allocator::init();
    if let Some(bytes) = allocator::alloc(64, 8) {
        bytes[0] = 42;
        println!("heap: allocated 64-byte demo block");
    }

    match fs::init() {
        Ok(()) => println!("fs: tinyfs mounted at LBA {}", fs::FS_START_LBA),
        Err(_) => println!("fs: tinyfs mount failed"),
    }

    interrupts::init();
    println!("keyboard: IRQ1 enabled, polling fallback active");
    println!();
    shell::run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("kernel panic: {}", info);
    loop {
        io::halt();
    }
}
