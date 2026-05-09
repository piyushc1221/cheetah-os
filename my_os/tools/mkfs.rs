use std::env;
use std::fs::File;
use std::io::{self, Write};

const BLOCK_SIZE: usize = 512;
const MAX_FILES: usize = 16;
const ENTRY_SIZE: usize = 64;
const TABLE_SECTORS: usize = 2;
const DATA_BLOCKS: usize = 16;
const MAGIC: &[u8; 8] = b"TINYFS1\0";

#[derive(Clone)]
struct FileSeed {
    name: &'static str,
    data: &'static [u8],
}

fn main() -> io::Result<()> {
    let output = env::args().nth(1).unwrap_or_else(|| "disk/fs.img".to_string());
    let seeds = [
        FileSeed {
            name: "readme.txt",
            data: b"Welcome to tiny64. Try: help, ls, cat readme.txt, write note hello\n",
        },
        FileSeed {
            name: "hello.txt",
            data: b"Hello from the tiny educational filesystem.\n",
        },
    ];

    let mut image = vec![0u8; (1 + TABLE_SECTORS + DATA_BLOCKS) * BLOCK_SIZE];
    write_superblock(&mut image[0..BLOCK_SIZE]);

    for (index, seed) in seeds.iter().enumerate() {
        write_entry(&mut image, index, seed.name, index as u32, seed.data.len() as u32);
        let data_start = (1 + TABLE_SECTORS + index) * BLOCK_SIZE;
        image[data_start..data_start + seed.data.len()].copy_from_slice(seed.data);
    }

    let mut file = File::create(output)?;
    file.write_all(&image)?;
    Ok(())
}

fn write_superblock(block: &mut [u8]) {
    block[0..8].copy_from_slice(MAGIC);
    block[8..12].copy_from_slice(&(BLOCK_SIZE as u32).to_le_bytes());
    block[12..16].copy_from_slice(&(MAX_FILES as u32).to_le_bytes());
    block[16..20].copy_from_slice(&(TABLE_SECTORS as u32).to_le_bytes());
    block[20..24].copy_from_slice(&((2048 + 1 + TABLE_SECTORS) as u32).to_le_bytes());
}

fn write_entry(image: &mut [u8], index: usize, name: &str, block: u32, size: u32) {
    let table_start = BLOCK_SIZE;
    let entry_start = table_start + index * ENTRY_SIZE;
    let entry = &mut image[entry_start..entry_start + ENTRY_SIZE];

    entry[0..name.len()].copy_from_slice(name.as_bytes());
    entry[32..36].copy_from_slice(&block.to_le_bytes());
    entry[36..40].copy_from_slice(&size.to_le_bytes());
    entry[40] = 1;
}
