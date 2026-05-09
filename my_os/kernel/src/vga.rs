use core::fmt::{self, Write};

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const COLOR: u8 = 0x0f;

pub struct Writer {
    row: usize,
    col: usize,
}

static mut WRITER: Writer = Writer { row: 0, col: 0 };

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        unsafe {
            crate::io::outb(0xe9, byte);
        }

        match byte {
            b'\n' => self.new_line(),
            8 => self.backspace(),
            byte => {
                if self.col >= WIDTH {
                    self.new_line();
                }
                let offset = (self.row * WIDTH + self.col) * 2;
                unsafe {
                    VGA_BUFFER.add(offset).write_volatile(byte);
                    VGA_BUFFER.add(offset + 1).write_volatile(COLOR);
                }
                self.col += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.col = 0;
        if self.row + 1 >= HEIGHT {
            self.scroll();
        } else {
            self.row += 1;
        }
    }

    fn scroll(&mut self) {
        for row in 1..HEIGHT {
            for col in 0..WIDTH {
                let from = (row * WIDTH + col) * 2;
                let to = ((row - 1) * WIDTH + col) * 2;
                unsafe {
                    let ch = VGA_BUFFER.add(from).read_volatile();
                    let color = VGA_BUFFER.add(from + 1).read_volatile();
                    VGA_BUFFER.add(to).write_volatile(ch);
                    VGA_BUFFER.add(to + 1).write_volatile(color);
                }
            }
        }
        self.clear_row(HEIGHT - 1);
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..WIDTH {
            let offset = (row * WIDTH + col) * 2;
            unsafe {
                VGA_BUFFER.add(offset).write_volatile(b' ');
                VGA_BUFFER.add(offset + 1).write_volatile(COLOR);
            }
        }
    }

    fn backspace(&mut self) {
        if self.col == 0 {
            return;
        }
        self.col -= 1;
        let offset = (self.row * WIDTH + self.col) * 2;
        unsafe {
            VGA_BUFFER.add(offset).write_volatile(b' ');
            VGA_BUFFER.add(offset + 1).write_volatile(COLOR);
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub fn clear_screen() {
    unsafe {
        for row in 0..HEIGHT {
            WRITER.clear_row(row);
        }
        WRITER.row = 0;
        WRITER.col = 0;
    }
}

pub fn backspace() {
    unsafe {
        WRITER.backspace();
    }
}

pub fn _print(args: fmt::Arguments) {
    unsafe {
        WRITER.write_fmt(args).ok();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::vga::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($fmt:expr) => {
        $crate::print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::print!(concat!($fmt, "\n"), $($arg)*)
    };
}
