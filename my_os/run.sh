#!/usr/bin/env sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

qemu-system-x86_64 \
    -m 64M \
    -drive file="$ROOT_DIR/disk/os.img",format=raw,if=ide,index=0 \
    -display cocoa,show-cursor=on \
    -k en-us \
    -no-reboot \
    -no-shutdown
