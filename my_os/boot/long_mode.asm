%ifndef KERNEL_SECTORS
%define KERNEL_SECTORS 128
%endif

%ifndef KERNEL_LBA
%define KERNEL_LBA 33
%endif

KERNEL_LOAD_REAL equ 0x10000
KERNEL_LOAD_HIGH equ 0x100000
KERNEL_BYTES equ KERNEL_SECTORS * 512

[org 0x8000]
[bits 16]

stage2_start:
    cli
    mov [boot_drive], dl

    mov si, stage2_message
    call print_string

    call enable_a20
    call load_kernel
    call enter_protected_mode

.hang:
    hlt
    jmp .hang

enable_a20:
    in al, 0x92
    or al, 0x02
    out 0x92, al
    ret

load_kernel:
    mov word [kernel_packet_segment], KERNEL_LOAD_REAL >> 4
    mov dword [kernel_packet_lba], KERNEL_LBA
    mov dword [kernel_packet_lba + 4], 0
    mov cx, KERNEL_SECTORS

.next_sector:
    test cx, cx
    jz .done

    mov ah, 0x42
    mov dl, [boot_drive]
    mov si, kernel_packet
    int 0x13
    jc disk_error

    add word [kernel_packet_segment], 0x20
    add dword [kernel_packet_lba], 1
    dec cx
    jmp .next_sector

.done:
    ret

enter_protected_mode:
    cli
    lgdt [gdt_descriptor]
    mov eax, cr0
    or eax, 0x1
    mov cr0, eax
    jmp CODE32_SELECTOR:protected_mode_start

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

stage2_message:
    db "Stage 2: loading kernel", 13, 10, 0

disk_error_message:
    db "Stage 2 disk read failed", 13, 10, 0

boot_drive:
    db 0

kernel_packet:
    db 0x10
    db 0
    dw 1
    dw 0
kernel_packet_segment:
    dw KERNEL_LOAD_REAL >> 4
kernel_packet_lba:
    dq KERNEL_LBA

align 8
gdt_start:
    dq 0x0000000000000000
gdt_code32:
    dq 0x00cf9a000000ffff
gdt_data:
    dq 0x00cf92000000ffff
gdt_code64:
    dq 0x00af9a000000ffff
gdt_end:

gdt_descriptor:
    dw gdt_end - gdt_start - 1
    dd gdt_start

CODE32_SELECTOR equ gdt_code32 - gdt_start
DATA_SELECTOR equ gdt_data - gdt_start
CODE64_SELECTOR equ gdt_code64 - gdt_start

[bits 32]
protected_mode_start:
    mov ax, DATA_SELECTOR
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x90000

    cld
    mov esi, KERNEL_LOAD_REAL
    mov edi, KERNEL_LOAD_HIGH
    mov ecx, KERNEL_BYTES
    rep movsb

    call setup_page_tables
    call enable_long_mode

    lgdt [gdt_descriptor]
    jmp CODE64_SELECTOR:long_mode_start

setup_page_tables:
    mov edi, pml4_table
    xor eax, eax
    mov ecx, (4096 * 3) / 4
    rep stosd

    mov eax, pdpt_table
    or eax, 0x3
    mov [pml4_table], eax
    mov dword [pml4_table + 4], 0

    mov eax, pd_table
    or eax, 0x3
    mov [pdpt_table], eax
    mov dword [pdpt_table + 4], 0

    mov edi, pd_table
    mov eax, 0x83
    mov ecx, 512
.map_next:
    mov [edi], eax
    mov dword [edi + 4], 0
    add eax, 0x200000
    add edi, 8
    loop .map_next
    ret

enable_long_mode:
    mov eax, pml4_table
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xc0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax
    ret

[bits 64]
long_mode_start:
    mov ax, DATA_SELECTOR
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax
    mov rsp, 0x90000

    mov rax, KERNEL_LOAD_HIGH
    call rax

.halt:
    hlt
    jmp .halt

align 4096
pml4_table:
    times 4096 db 0
pdpt_table:
    times 4096 db 0
pd_table:
    times 4096 db 0
