use super::format::ENTRY_SIZE;

#[derive(Clone, Copy)]
pub struct FileEntry {
    pub name: [u8; 32],
    pub start_block: u32,
    pub size: u32,
    pub used: bool,
}

impl FileEntry {
    pub const fn empty() -> Self {
        Self {
            name: [0; 32],
            start_block: 0,
            size: 0,
            used: false,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut name = [0u8; 32];
        name.copy_from_slice(&bytes[0..32]);
        Self {
            name,
            start_block: read_u32(&bytes[32..36]),
            size: read_u32(&bytes[36..40]),
            used: bytes[40] == 1,
        }
    }

    pub fn write_bytes(&self, bytes: &mut [u8]) {
        bytes.fill(0);
        bytes[0..32].copy_from_slice(&self.name);
        bytes[32..36].copy_from_slice(&self.start_block.to_le_bytes());
        bytes[36..40].copy_from_slice(&self.size.to_le_bytes());
        bytes[40] = if self.used { 1 } else { 0 };
    }

    pub fn set_name(&mut self, name: &str) -> bool {
        let bytes = name.as_bytes();
        if bytes.is_empty() || bytes.len() > 31 {
            return false;
        }

        self.name = [0; 32];
        for (index, byte) in bytes.iter().enumerate() {
            self.name[index] = *byte;
        }
        true
    }

    pub fn name_equals(&self, name: &str) -> bool {
        let bytes = name.as_bytes();
        if bytes.len() != self.name_len() {
            return false;
        }
        &self.name[0..bytes.len()] == bytes
    }

    pub fn print_name(&self) {
        for byte in self.name.iter().copied() {
            if byte == 0 {
                break;
            }
            crate::print!("{}", byte as char);
        }
    }

    fn name_len(&self) -> usize {
        self.name.iter().position(|byte| *byte == 0).unwrap_or(32)
    }
}

pub fn entry_range(index: usize) -> core::ops::Range<usize> {
    let start = index * ENTRY_SIZE;
    start..start + ENTRY_SIZE
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
