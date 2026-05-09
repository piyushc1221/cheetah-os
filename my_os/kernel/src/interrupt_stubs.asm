[bits 64]

global isr_default_stub
global isr_error_stub
global irq_keyboard_stub

extern rust_interrupt_handler
extern rust_keyboard_interrupt

%macro PUSH_REGS 0
    push rax
    push rbx
    push rcx
    push rdx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
%endmacro

%macro POP_REGS 0
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rdx
    pop rcx
    pop rbx
    pop rax
%endmacro

isr_default_stub:
    PUSH_REGS
    call rust_interrupt_handler
    POP_REGS
    iretq

isr_error_stub:
    PUSH_REGS
    call rust_interrupt_handler
    POP_REGS
    add rsp, 8
    iretq

irq_keyboard_stub:
    PUSH_REGS
    call rust_keyboard_interrupt
    POP_REGS
    iretq
