use crate::{fs, io, keyboard, vga};

const LINE_SIZE: usize = 128;

pub fn run() -> ! {
    crate::println!("type 'help' for commands");

    loop {
        crate::print!("> ");
        let mut line = [0u8; LINE_SIZE];
        let len = read_line(&mut line);
        execute(&line[..len]);
    }
}

fn read_line(buffer: &mut [u8; LINE_SIZE]) -> usize {
    let mut len = 0;

    loop {
        if let Some(ch) = keyboard::read_char() {
            match ch {
                b'\n' => {
                    crate::println!();
                    return len;
                }
                8 => {
                    if len > 0 {
                        len -= 1;
                        vga::backspace();
                    }
                }
                byte if byte.is_ascii_graphic() || byte == b' ' => {
                    if len + 1 < LINE_SIZE {
                        buffer[len] = byte;
                        len += 1;
                        crate::print!("{}", byte as char);
                    }
                }
                _ => {}
            }
        } else {
            io::halt();
        }
    }
}

fn execute(line: &[u8]) {
    let line = trim(line);
    if line.is_empty() {
        return;
    }

    let (command, rest) = split_word(line);

    if equals(command, b"help") {
        crate::println!("help clear echo ls cat write");
        crate::println!("write usage: write filename text to store");
    } else if equals(command, b"clear") {
        vga::clear_screen();
    } else if equals(command, b"echo") {
        print_bytes(trim_left(rest));
        crate::println!();
    } else if equals(command, b"ls") {
        fs::list();
    } else if equals(command, b"cat") {
        cat(trim_left(rest));
    } else if equals(command, b"write") {
        write(trim_left(rest));
    } else {
        crate::println!("unknown command");
    }
}

fn cat(args: &[u8]) {
    let (name, _) = split_word(args);
    if name.is_empty() {
        crate::println!("usage: cat filename");
        return;
    }

    let mut buffer = [0u8; fs::format::BLOCK_SIZE];
    match fs::read_file(as_str(name), &mut buffer) {
        Ok(size) => {
            print_bytes(&buffer[0..size]);
            crate::println!();
        }
        Err(_) => crate::println!("cat: file not found"),
    }
}

fn write(args: &[u8]) {
    let (name, rest) = split_word(args);
    let data = trim_left(rest);

    if name.is_empty() || data.is_empty() {
        crate::println!("usage: write filename text to store");
        return;
    }

    match fs::write_file(as_str(name), data) {
        Ok(()) => crate::println!("written"),
        Err(fs::FsError::FileTooLarge) => crate::println!("write: file too large"),
        Err(fs::FsError::NoSpace) => crate::println!("write: no file slots left"),
        Err(_) => crate::println!("write: failed"),
    }
}

fn split_word(line: &[u8]) -> (&[u8], &[u8]) {
    for index in 0..line.len() {
        if line[index] == b' ' {
            return (&line[0..index], &line[index..]);
        }
    }
    (line, &[])
}

fn trim(line: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = line.len();

    while start < end && line[start] == b' ' {
        start += 1;
    }
    while end > start && line[end - 1] == b' ' {
        end -= 1;
    }

    &line[start..end]
}

fn trim_left(line: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < line.len() && line[start] == b' ' {
        start += 1;
    }
    &line[start..]
}

fn equals(left: &[u8], right: &[u8]) -> bool {
    left == right
}

fn as_str(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("")
}

fn print_bytes(bytes: &[u8]) {
    for byte in bytes.iter().copied() {
        crate::print!("{}", byte as char);
    }
}
