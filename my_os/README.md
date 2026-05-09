git rm --cached my_os
my_os/
├── boot/
│   ├── boot.asm
│   └── long_mode.asm
├── kernel/
│   ├── .cargo/
│   │   └── config.toml
│   ├── Cargo.toml
│   └── src/
│       ├── allocator.rs
│       ├── fs/
│       │   ├── disk.rs
│       │   ├── format.rs
│       │   ├── inode.rs
│       │   └── mod.rs
│       ├── interrupts.rs
│       ├── interrupt_stubs.asm
│       ├── io.rs
│       ├── keyboard.rs
│       ├── main.rs
│       ├── memory.rs
│       ├── shell.rs
│       └── vga.rs
├── linker.ld
├── Makefile
├── build.sh
├── run.sh
├── disk/
│   └── os.img
└── tools/
    └── mkfs.rs
```

## macOS Tools

Install Homebrew if needed:

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Install the OS tools:

```sh
brew install nasm qemu
rustup target add x86_64-unknown-none
```

`nasm` assembles boot code, `qemu-system-x86_64` runs the raw disk image, Rustup provides the bare-metal target, and Rust's bundled `rust-lld` emits the kernel as a flat binary using the linker script.

## Build

```sh
cd my_os
chmod +x build.sh run.sh
make build
```

Expected output:

```text
built disk/os.img
stage2 sectors: 32
kernel sectors: <number>
tinyfs starts at LBA: 2048
```

## Run

```sh
make run
```

QEMU opens a VGA window. Click the window and type:

```text
help
ls
cat readme.txt
write note.txt hello from tiny64
cat note.txt
clear
```

On macOS, make sure the QEMU VGA window is focused before typing. Click inside the black VGA area once. If the mouse is captured, press `control` + `option` + `g` to release it.

`make run` rebuilds `disk/os.img`, so it resets TinyFS to the starter files. To boot the existing disk image without formatting it again, use:

```sh
make run-only
```

## TinyFS Layout

The disk image is raw 512-byte sectors. TinyFS starts at LBA 2048 so the bootloader and kernel have room before it.

```text
LBA 0       boot sector
LBA 1-32    stage 2 loader
LBA 33..    kernel binary sectors
LBA 2048    TinyFS superblock
LBA 2049    file table sector 0
LBA 2050    file table sector 1
LBA 2051    data block for file slot 0
LBA 2052    data block for file slot 1
...
LBA 2066    data block for file slot 15
```

Each file table entry is 64 bytes:

```text
bytes 0..31    zero-terminated file name
bytes 32..35   start block, little endian
bytes 36..39   file size, little endian
byte  40       used flag, 1 means occupied
bytes 41..63   reserved
```

This filesystem is deliberately tiny: 16 files, one 512-byte block per file, no directories, no free bitmap. That keeps read/write support visible instead of hiding it behind layers.

## Why Each File Exists

`boot/boot.asm` is the 512-byte BIOS boot sector. BIOS loads it at physical address `0x7c00`; it uses BIOS interrupt `0x13` to read the second-stage loader from disk.

`boot/long_mode.asm` is the second-stage loader. It enables A20, reads kernel sectors while BIOS calls are still legal, switches to protected mode, copies the kernel to `0x100000`, builds identity page tables, enables long mode, and calls Rust.

`linker.ld` places the Rust kernel at physical `1M`, matching the loader jump address.

`kernel/.cargo/config.toml` tells Cargo to build for `x86_64-unknown-none` and to pass the linker script plus the NASM interrupt object to the linker.

`kernel/src/main.rs` is the Rust kernel entry point. It initializes VGA, memory notes, the allocator, TinyFS, interrupts, and then starts the shell.

`kernel/src/vga.rs` writes directly to VGA text memory at `0xb8000`.

`kernel/src/io.rs` wraps x86 port I/O instructions such as `in`, `out`, and `hlt`.

`kernel/src/interrupts.rs` builds and loads the IDT, remaps the PIC, unmasks keyboard IRQ1, and exposes Rust handlers called by assembly stubs.

`kernel/src/interrupt_stubs.asm` contains interrupt entry code because stable Rust does not use the special `x86-interrupt` ABI here.

`kernel/src/keyboard.rs` translates PS/2 set-1 scancodes into ASCII and stores them in a small ring buffer.

`kernel/src/memory.rs` records the memory assumptions established by the loader.

`kernel/src/allocator.rs` demonstrates a simple bump allocator without pulling in the Rust standard library.

`kernel/src/fs/format.rs` defines the TinyFS on-disk constants and superblock format.

`kernel/src/fs/inode.rs` parses and writes fixed-size file table entries.

`kernel/src/fs/disk.rs` reads and writes sectors using ATA PIO ports on QEMU's IDE disk.

`kernel/src/fs/mod.rs` mounts TinyFS and implements `list`, `read_file`, and `write_file`.

`kernel/src/shell.rs` is the terminal loop that connects keyboard input, VGA output, and TinyFS commands.

`tools/mkfs.rs` is a host-side formatter that creates the initial TinyFS image.

`build.sh` assembles boot code, builds Rust, creates the filesystem, and lays out the final raw disk image.

`run.sh` starts QEMU with the raw disk attached as an IDE drive.

## Common Errors

`nasm: command not found`: run `brew install nasm`.

`qemu-system-x86_64: command not found`: run `brew install qemu`.

`can't find crate for core`: run `rustup target add x86_64-unknown-none`.

QEMU boots but the shell does not type: click inside the QEMU VGA window so it captures keyboard input.

`kernel is ... sectors, but only ... fit`: the kernel grew too large for the educational fixed layout. Reduce debug output or move `FS_START_LBA` higher in `build.sh` and `kernel/src/fs/format.rs`.

# cheetah-os
1.  ⚡ Cheetah OS — a lightning-fast OS built in Rust for speed, safety, and control.  2.  Cheetah OS is a fast, minimal Rust-based OS focused on performance and safety.  3.  Blazing fast Rust OS built for performance, safety, and low-level control.  4.  Cheetah OS — fast, minimal, Rust-powered system with modern design.
 ccd17a4dac57d6b9d2e51368257780d7337f46e9
