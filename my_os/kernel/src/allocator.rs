const HEAP_SIZE: usize = 64 * 1024;

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut NEXT: usize = 0;

pub fn init() {
    unsafe {
        NEXT = 0;
    }
    crate::println!("heap: simple bump allocator ready ({} KiB)", HEAP_SIZE / 1024);
}

pub fn alloc(size: usize, align: usize) -> Option<&'static mut [u8]> {
    if align == 0 || !align.is_power_of_two() {
        return None;
    }

    unsafe {
        let base = HEAP.as_mut_ptr() as usize;
        let current = align_up(base + NEXT, align);
        let next = current.checked_add(size)?;
        let used = next.checked_sub(base)?;

        if used > HEAP_SIZE {
            return None;
        }

        NEXT = used;
        Some(core::slice::from_raw_parts_mut(current as *mut u8, size))
    }
}

fn align_up(value: usize, align: usize) -> usize {
    (value + align - 1) & !(align - 1)
}
