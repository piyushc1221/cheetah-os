use crate::{io, keyboard};
use core::arch::asm;

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct IdtEntry {
    offset_low: u16,
    selector: u16,
    options: u16,
    offset_mid: u16,
    offset_high: u32,
    reserved: u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            options: 0,
            offset_mid: 0,
            offset_high: 0,
            reserved: 0,
        }
    }

    fn set_handler(&mut self, handler: u64) {
        self.offset_low = handler as u16;
        self.selector = 0x18;
        self.options = 0x8e00;
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.reserved = 0;
    }
}

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::missing(); 256];

extern "C" {
    fn isr_default_stub();
    fn isr_error_stub();
    fn irq_keyboard_stub();
}

pub fn init() {
    unsafe {
        for entry in IDT.iter_mut() {
            entry.set_handler(isr_default_stub as *const () as u64);
        }

        for vector in [8usize, 10, 11, 12, 13, 14, 17, 21, 29, 30] {
            IDT[vector].set_handler(isr_error_stub as *const () as u64);
        }

        IDT[33].set_handler(irq_keyboard_stub as *const () as u64);

        let pointer = IdtPointer {
            limit: core::mem::size_of::<[IdtEntry; 256]>() as u16 - 1,
            base: core::ptr::addr_of!(IDT) as u64,
        };
        asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack));

        remap_pic();
        io::outb(0x21, 0xfd);
        io::outb(0xa1, 0xff);
        asm!("sti", options(nomem, nostack, preserves_flags));
    }
}

unsafe fn remap_pic() {
    io::outb(0x20, 0x11);
    io::io_wait();
    io::outb(0xa0, 0x11);
    io::io_wait();

    io::outb(0x21, 0x20);
    io::io_wait();
    io::outb(0xa1, 0x28);
    io::io_wait();

    io::outb(0x21, 0x04);
    io::io_wait();
    io::outb(0xa1, 0x02);
    io::io_wait();

    io::outb(0x21, 0x01);
    io::io_wait();
    io::outb(0xa1, 0x01);
    io::io_wait();
}

#[no_mangle]
pub extern "C" fn rust_interrupt_handler() {
    unsafe {
        io::outb(0x20, 0x20);
    }
}

#[no_mangle]
pub extern "C" fn rust_keyboard_interrupt() {
    unsafe {
        let scancode = io::inb(0x60);
        keyboard::handle_scancode(scancode);
        io::outb(0x20, 0x20);
    }
}
