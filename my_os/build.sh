#!/usr/bin/env sh
set -eu

STAGE2_SECTORS=32
KERNEL_LBA=$((1 + STAGE2_SECTORS))
FS_START_LBA=2048

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BUILD_DIR="$ROOT_DIR/build"
DISK_DIR="$ROOT_DIR/disk"
KERNEL_BIN="$ROOT_DIR/kernel/target/x86_64-unknown-none/release/kernel"

mkdir -p "$BUILD_DIR" "$DISK_DIR"

file_size() {
    if stat -f%z "$1" >/dev/null 2>&1; then
        stat -f%z "$1"
    else
        stat -c%s "$1"
    fi
}

pad_to_size() {
    file=$1
    target_size=$2
    current_size=$(file_size "$file")
    if [ "$current_size" -gt "$target_size" ]; then
        echo "$file is larger than $target_size bytes" >&2
        exit 1
    fi
    pad=$((target_size - current_size))
    if [ "$pad" -gt 0 ]; then
        dd if=/dev/zero bs=1 count="$pad" >> "$file" 2>/dev/null
    fi
}

pad_sectors_to_image() {
    image=$1
    sectors=$2
    if [ "$sectors" -gt 0 ]; then
        dd if=/dev/zero bs=512 count="$sectors" >> "$image" 2>/dev/null
    fi
}

nasm -f bin -D STAGE2_SECTORS="$STAGE2_SECTORS" "$ROOT_DIR/boot/boot.asm" -o "$BUILD_DIR/boot.bin"
nasm -f elf64 "$ROOT_DIR/kernel/src/interrupt_stubs.asm" -o "$BUILD_DIR/interrupt_stubs.o"

(cd "$ROOT_DIR/kernel" && cargo build --release)
cp "$KERNEL_BIN" "$BUILD_DIR/kernel.bin"

kernel_size=$(file_size "$BUILD_DIR/kernel.bin")
KERNEL_SECTORS=$(((kernel_size + 511) / 512))
max_kernel_sectors=$((FS_START_LBA - KERNEL_LBA))

if [ "$KERNEL_SECTORS" -gt "$max_kernel_sectors" ]; then
    echo "kernel is $KERNEL_SECTORS sectors, but only $max_kernel_sectors fit before tinyfs" >&2
    exit 1
fi

nasm -f bin \
    -D KERNEL_LBA="$KERNEL_LBA" \
    -D KERNEL_SECTORS="$KERNEL_SECTORS" \
    "$ROOT_DIR/boot/long_mode.asm" \
    -o "$BUILD_DIR/stage2.bin"

cp "$BUILD_DIR/stage2.bin" "$BUILD_DIR/stage2.pad"
pad_to_size "$BUILD_DIR/stage2.pad" $((STAGE2_SECTORS * 512))

cp "$BUILD_DIR/kernel.bin" "$BUILD_DIR/kernel.pad"
pad_to_size "$BUILD_DIR/kernel.pad" $((KERNEL_SECTORS * 512))

rustc "$ROOT_DIR/tools/mkfs.rs" -o "$BUILD_DIR/mkfs"
"$BUILD_DIR/mkfs" "$BUILD_DIR/fs.img"

IMAGE="$DISK_DIR/os.img"
cat "$BUILD_DIR/boot.bin" "$BUILD_DIR/stage2.pad" "$BUILD_DIR/kernel.pad" > "$IMAGE"

used_sectors=$((1 + STAGE2_SECTORS + KERNEL_SECTORS))
pad_before_fs=$((FS_START_LBA - used_sectors))
pad_sectors_to_image "$IMAGE" "$pad_before_fs"
cat "$BUILD_DIR/fs.img" >> "$IMAGE"

echo "built disk/os.img"
echo "stage2 sectors: $STAGE2_SECTORS"
echo "kernel sectors: $KERNEL_SECTORS"
echo "tinyfs starts at LBA: $FS_START_LBA"
