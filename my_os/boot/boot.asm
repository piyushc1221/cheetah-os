%ifndef STAGE2_SECTORS
%define STAGE2_SECTORS 32
%endif

[org 0x7c00]
[bits 16]

stage1_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00
    sti

    mov [boot_drive], dl
    mov si, boot_message
    call print_string

    mov ah, 0x42
    mov dl, [boot_drive]
    mov si, stage2_packet
    int 0x13
    jc disk_error

    mov dl, [boot_drive]
    jmp 0x0000:0x8000

disk_error:
    mov si, disk_error_message
    call print_string
    cli
.hang:
    hlt
    jmp .hang

print_string:
    lodsb
    test al, al
    jz .done
    mov ah, 0x0e
    mov bh, 0x00
    int 0x10
    jmp print_string
.done:
    ret

boot_message:
    db "Stage 1: loading stage 2", 13, 10, 0

disk_error_message:
    db "BIOS disk read failed", 13, 10, 0

boot_drive:
    db 0

stage2_packet:
    db 0x10
    db 0
    dw STAGE2_SECTORS
    dw 0x8000
    dw 0x0000
    dq 1

times 510 - ($ - $$) db 0
dw 0xaa55
