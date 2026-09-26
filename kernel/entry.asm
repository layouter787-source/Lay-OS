; Lay OS kernel entry
bits 64
org 0x1000

global kernel_entry
extern lay_kernel_main

kernel_entry:
    cli
    mov rsp, 0x90000
    call lay_kernel_main

.halt:
    hlt
    jmp .halt
