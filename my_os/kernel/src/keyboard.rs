const QUEUE_SIZE: usize = 256;

static mut QUEUE: [u8; QUEUE_SIZE] = [0; QUEUE_SIZE];
static mut HEAD: usize = 0;
static mut TAIL: usize = 0;
static mut SHIFT: bool = false;

pub fn handle_scancode(scancode: u8) {
    unsafe {
        match scancode {
            0x2a | 0x36 => SHIFT = true,
            0xaa | 0xb6 => SHIFT = false,
            0x1c => push(b'\n'),
            0x0e => push(8),
            code if code & 0x80 == 0 => {
                if let Some(ch) = translate(code, SHIFT) {
                    push(ch);
                }
            }
            _ => {}
        }
    }
}

pub fn read_char() -> Option<u8> {
    poll_controller();

    unsafe {
        if HEAD == TAIL {
            None
        } else {
            let ch = QUEUE[TAIL];
            TAIL = (TAIL + 1) % QUEUE_SIZE;
            Some(ch)
        }
    }
}

fn poll_controller() {
    unsafe {
        let status = crate::io::inb(0x64);
        if status & 0x01 != 0 {
            let scancode = crate::io::inb(0x60);
            handle_scancode(scancode);
        }
    }
}

unsafe fn push(ch: u8) {
    let next = (HEAD + 1) % QUEUE_SIZE;
    if next != TAIL {
        QUEUE[HEAD] = ch;
        HEAD = next;
    }
}

fn translate(scancode: u8, shift: bool) -> Option<u8> {
    let ch = match scancode {
        0x02 => if shift { b'!' } else { b'1' },
        0x03 => if shift { b'@' } else { b'2' },
        0x04 => if shift { b'#' } else { b'3' },
        0x05 => if shift { b'$' } else { b'4' },
        0x06 => if shift { b'%' } else { b'5' },
        0x07 => if shift { b'^' } else { b'6' },
        0x08 => if shift { b'&' } else { b'7' },
        0x09 => if shift { b'*' } else { b'8' },
        0x0a => if shift { b'(' } else { b'9' },
        0x0b => if shift { b')' } else { b'0' },
        0x0c => if shift { b'_' } else { b'-' },
        0x0d => if shift { b'+' } else { b'=' },
        0x10 => letter(b'q', shift),
        0x11 => letter(b'w', shift),
        0x12 => letter(b'e', shift),
        0x13 => letter(b'r', shift),
        0x14 => letter(b't', shift),
        0x15 => letter(b'y', shift),
        0x16 => letter(b'u', shift),
        0x17 => letter(b'i', shift),
        0x18 => letter(b'o', shift),
        0x19 => letter(b'p', shift),
        0x1a => if shift { b'{' } else { b'[' },
        0x1b => if shift { b'}' } else { b']' },
        0x1e => letter(b'a', shift),
        0x1f => letter(b's', shift),
        0x20 => letter(b'd', shift),
        0x21 => letter(b'f', shift),
        0x22 => letter(b'g', shift),
        0x23 => letter(b'h', shift),
        0x24 => letter(b'j', shift),
        0x25 => letter(b'k', shift),
        0x26 => letter(b'l', shift),
        0x27 => if shift { b':' } else { b';' },
        0x28 => if shift { b'"' } else { b'\'' },
        0x29 => if shift { b'~' } else { b'`' },
        0x2b => if shift { b'|' } else { b'\\' },
        0x2c => letter(b'z', shift),
        0x2d => letter(b'x', shift),
        0x2e => letter(b'c', shift),
        0x2f => letter(b'v', shift),
        0x30 => letter(b'b', shift),
        0x31 => letter(b'n', shift),
        0x32 => letter(b'm', shift),
        0x33 => if shift { b'<' } else { b',' },
        0x34 => if shift { b'>' } else { b'.' },
        0x35 => if shift { b'?' } else { b'/' },
        0x39 => b' ',
        _ => return None,
    };
    Some(ch)
}

fn letter(ch: u8, shift: bool) -> u8 {
    if shift {
        ch - 32
    } else {
        ch
    }
}
