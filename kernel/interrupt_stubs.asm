; Hardware interrupt stubs for Lay OS.
bits 64

global irq0_stub
global irq1_stub
global exception_stub
global syscall_stub

extern timer_handler
extern keyboard_handler
extern exception_handler
extern syscall_handler

irq0_stub:
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    mov rdi, rsp
    call timer_handler
    mov rsp, rax
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
    iretq

irq1_stub:
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    call keyboard_handler
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
    iretq

exception_stub:
    call exception_handler
    ud2

; Software syscall entry. The same register frame is used by the kernel ABI.
syscall_stub:
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    mov rdi, rsp
    call syscall_handler
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax
    iretq
