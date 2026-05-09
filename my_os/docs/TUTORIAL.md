# tiny64 Step-by-Step Tutorial

This document follows the project in build order. The complete source code lives in the repository files; this tutorial explains why each step exists, what to run, what to expect, and what usually goes wrong.

## Step 1: Install macOS Tools

Goal: get a native macOS toolchain without Docker.

Theory: BIOS boot code is assembled by NASM. The Rust kernel uses the `x86_64-unknown-none` target and Rust's bundled `rust-lld` linker. QEMU runs the raw disk image. Binutils is optional for inspection tools such as `gobjdump`, but the normal build does not need it.

Commands:

```sh
brew install nasm qemu
brew install binutils
rustup target add x86_64-unknown-none
```

Expected output: Homebrew installs `nasm` and `qemu-system-x86_64`; Rustup reports the target is installed.

Common errors:

- `nasm: command not found`: run `brew install nasm`.
- `qemu-system-x86_64: command not found`: run `brew install qemu`.
- `can't find crate for core`: run `rustup target add x86_64-unknown-none`.

## Step 2: Create the Project Tree

Goal: separate boot code, kernel code, host tools, and generated disk files.

Theory: boot code must be raw binary sectors, the kernel is built by Cargo, and the filesystem formatter is a normal host program.

File tree:

```text
my_os/
├── boot/
├── kernel/
├── tools/
├── disk/
├── build.sh
├── run.sh
├── Makefile
└── linker.ld
```

Commands:

```sh
mkdir -p my_os/boot my_os/kernel/src/fs my_os/tools my_os/disk
```

Expected output: directories exist. The repository already contains the final structure.

## Step 3: BIOS Boot Sector

Files: `boot/boot.asm`

Goal: make a valid 512-byte boot sector and load the second stage.

Theory: legacy BIOS loads sector 0 to physical `0x7c00` and jumps to it with the boot drive number in `DL`. The last two bytes must be `0xaa55`. Because 512 bytes is tiny, stage 1 only prints a message and uses BIOS interrupt `0x13` extended read function `0x42` to load stage 2 from LBA 1 to `0x8000`.

Reading guide:

- `[org 0x7c00]` makes labels match the BIOS load address.
- Segment registers are cleared so `DS:SI` and `ES:BX` point where expected.
- `stage2_packet` is a BIOS Disk Address Packet: size, sector count, destination address, and start LBA.
- `int 0x13` reads sectors from the boot disk.
- `times 510 - ($ - $$) db 0` pads the sector.
- `dw 0xaa55` marks it bootable.

Commands:

```sh
nasm -f bin -D STAGE2_SECTORS=32 boot/boot.asm -o build/boot.bin
```

Expected output: `build/boot.bin` is exactly 512 bytes.

Common errors:

- Boot loop or `not a bootable disk`: the `0xaa55` signature is missing or not at byte 510.
- Disk read failed: QEMU drive layout does not match the expected raw image.

## Step 4: Stage 2 and Long Mode

Files: `boot/long_mode.asm`

Goal: load the Rust kernel, enter protected mode, enable paging, enter 64-bit long mode, and jump to the kernel.

Theory: BIOS calls only work in real mode, so stage 2 reads all kernel sectors first. The kernel is initially read below 1 MiB at `0x10000`, then copied to `0x100000` after protected mode starts. Long mode requires PAE paging, `EFER.LME`, and `CR0.PG`. The loader identity maps the first 1 GiB using 2 MiB pages.

Reading guide:

- `enable_a20` opens access above 1 MiB.
- `load_kernel` reads one sector at a time with BIOS `int 0x13`.
- `enter_protected_mode` loads the GDT and sets `CR0.PE`.
- `protected_mode_start` copies the kernel to 1 MiB.
- `setup_page_tables` creates PML4, PDPT, and PD tables.
- `enable_long_mode` writes `CR3`, `CR4.PAE`, `EFER.LME`, and `CR0.PG`.
- `long_mode_start` sets a 64-bit stack and calls address `0x100000`.

Commands:

```sh
nasm -f bin -D KERNEL_LBA=33 -D KERNEL_SECTORS=31 boot/long_mode.asm -o build/stage2.bin
```

Expected output: `build/stage2.bin` fits inside 32 sectors.

Common errors:

- Triple fault after stage 2: GDT selector, page-table address, or kernel load address is wrong.
- Kernel overwrites TinyFS: kernel grew past LBA 2048; move `FS_START_LBA` higher.

## Step 5: Link the Rust Kernel

Files: `kernel/Cargo.toml`, `kernel/.cargo/config.toml`, `linker.ld`

Goal: build a `#![no_std]` kernel as a flat binary loaded at physical `0x100000`.

Theory: there is no OS underneath this kernel, so Rust cannot use `std`. The linker script places sections starting at 1 MiB. `_start` is pinned to `.text.entry` so it is the first byte in the flat binary, exactly where the bootloader jumps.

Reading guide:

- `Cargo.toml` sets `panic = "abort"` because unwinding needs OS/runtime support.
- `.cargo/config.toml` selects `x86_64-unknown-none`.
- `--oformat=binary` asks `rust-lld` to emit a flat kernel binary directly.
- `linker.ld` starts at `1M` and emits `.text.entry` before normal `.text`.

Commands:

```sh
cd kernel
cargo build --release
cd ..
```

Expected output: `kernel/target/x86_64-unknown-none/release/kernel` exists.

Common errors:

- `undefined symbol: irq_keyboard_stub`: build `build/interrupt_stubs.o` before Cargo.
- Kernel immediately panics or runs garbage: `_start` is not first in the flat binary.

## Step 6: VGA Text Output

Files: `kernel/src/vga.rs`, `kernel/src/io.rs`

Goal: print text without a graphics driver.

Theory: VGA text mode maps an 80x25 character buffer at physical `0xb8000`. Each screen cell is two bytes: ASCII character and color attribute.

Reading guide:

- `VGA_BUFFER` is a raw pointer to `0xb8000`.
- `write_volatile` prevents the compiler from removing memory-mapped I/O writes.
- `new_line` and `scroll` keep output readable.
- `print!` and `println!` macros route formatted Rust text into the VGA writer.
- Port `0xe9` mirrors output to QEMU debug console for headless testing.

Expected output after boot:

```text
tiny64: Rust kernel entered long mode
```

Common errors:

- Blank screen: long mode jump failed or VGA memory is not identity-mapped.

## Step 7: Interrupts and Keyboard

Files: `kernel/src/interrupts.rs`, `kernel/src/interrupt_stubs.asm`, `kernel/src/keyboard.rs`

Goal: receive keyboard input through IRQ1.

Theory: hardware interrupts enter through the Interrupt Descriptor Table. The old PIC is remapped so IRQs start at vectors 32..47 instead of colliding with CPU exceptions. The keyboard is IRQ1, so it becomes IDT vector 33.

Reading guide:

- `IdtEntry` matches the x86_64 IDT gate format.
- `remap_pic` sends initialization control words to ports `0x20`, `0x21`, `0xa0`, and `0xa1`.
- `irq_keyboard_stub` saves registers, calls Rust, restores registers, then `iretq`.
- `rust_keyboard_interrupt` reads port `0x60` and acknowledges the PIC.
- `keyboard.rs` translates PS/2 set-1 scancodes into ASCII.

Expected output: after boot, typing in QEMU appears at the shell prompt.

Common errors:

- No typing: click the QEMU window first.
- Repeating or frozen input: missing PIC end-of-interrupt command to port `0x20`.

## Step 8: Heap Basics

Files: `kernel/src/allocator.rs`, `kernel/src/memory.rs`

Goal: demonstrate dynamic memory without using `std` or `alloc`.

Theory: a bump allocator owns a fixed byte array and hands out aligned slices by moving a `NEXT` pointer forward. It cannot free memory, but it is simple and excellent for learning.

Reading guide:

- `HEAP` is 64 KiB of static storage.
- `alloc(size, align)` aligns the current pointer and checks capacity.
- `align_up` rounds an address to the next alignment boundary.

Expected output:

```text
heap: simple bump allocator ready (64 KiB)
heap: allocated 64-byte demo block
```

Common errors:

- Allocation fails: requested size exceeds the fixed heap.

## Step 9: TinyFS

Files: `kernel/src/fs/format.rs`, `kernel/src/fs/inode.rs`, `kernel/src/fs/disk.rs`, `kernel/src/fs/mod.rs`, `tools/mkfs.rs`

Goal: mount a small custom filesystem and read/write files from the shell.

Theory: TinyFS uses fixed 512-byte sectors. The kernel talks to QEMU's IDE disk through ATA PIO ports. The filesystem is intentionally fixed-size: 16 file slots and one 512-byte data block per file.

Layout:

```text
LBA 2048  superblock
LBA 2049  file table sector 0
LBA 2050  file table sector 1
LBA 2051  file slot 0 data
...
LBA 2066  file slot 15 data
```

Reading guide:

- `format.rs` defines constants shared by mount/read/write.
- `inode.rs` converts 64-byte table entries to Rust structs and back.
- `disk.rs` selects an LBA and uses ATA commands `0x20` read and `0x30` write.
- `mod.rs` loads the table, finds names, and updates sectors.
- `mkfs.rs` creates the first filesystem image on the host.

Commands:

```sh
rustc tools/mkfs.rs -o build/mkfs
build/mkfs build/fs.img
```

Expected output: `build/fs.img` contains `readme.txt` and `hello.txt`.

Common errors:

- `fs: tinyfs mount failed`: disk not attached as IDE or LBA constants do not match the image builder.
- `write: file too large`: TinyFS stores at most 512 bytes per file.

## Step 10: Shell

Files: `kernel/src/shell.rs`

Goal: provide a basic terminal that interacts with TinyFS.

Theory: the shell reads characters from the keyboard ring buffer, echoes them to VGA, parses one line, and dispatches simple commands. It avoids heap allocations by using fixed-size byte buffers.

Commands inside QEMU:

```text
help
ls
cat readme.txt
write note.txt hello from tiny64
cat note.txt
clear
```

Expected output:

```text
readme.txt  67 bytes
hello.txt  44 bytes
written
hello from tiny64
```

Common errors:

- `unknown command`: command parser only supports the six listed commands.
- `usage: write filename text to store`: both a name and content are required.

## Step 11: Build the Disk Image

Files: `build.sh`, `Makefile`

Goal: automate all assembly, Rust build, filesystem formatting, padding, and image layout.

Theory: a BIOS disk image is just bytes. The script concatenates sector 0, padded stage 2 sectors, padded kernel sectors, zero padding up to LBA 2048, then TinyFS.

Commands:

```sh
chmod +x build.sh run.sh
make build
```

Expected output:

```text
built disk/os.img
stage2 sectors: 32
kernel sectors: 31
tinyfs starts at LBA: 2048
```

Common errors:

- `stage2.bin is larger than 16384 bytes`: increase `STAGE2_SECTORS` in `build.sh`.
- `kernel is ... sectors`: move `FS_START_LBA` higher in both `build.sh` and `kernel/src/fs/format.rs`.

## Step 12: Run in QEMU

Files: `run.sh`

Goal: boot the raw disk image.

Theory: `-drive ...,if=ide,index=0` attaches the raw image to the primary IDE disk, matching the kernel's ATA PIO code.

Commands:

```sh
make run
```

Expected output: a QEMU VGA window opens with the tiny64 banner and a `>` prompt.

Headless verification command:

```sh
qemu-system-x86_64 \
  -m 64M \
  -drive file=disk/os.img,format=raw,if=ide,index=0 \
  -display none \
  -chardev file,id=dbg,path=build/qemu.log \
  -device isa-debugcon,iobase=0xe9,chardev=dbg \
  -no-reboot \
  -no-shutdown
```

Expected debug log:

```text
tiny64: Rust kernel entered long mode
fs: tinyfs mounted at LBA 2048
type 'help' for commands
>
```
