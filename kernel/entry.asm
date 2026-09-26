bits 64

global kernel_entry
extern lay_kernel_main
extern kernel_boot_stack_end

kernel_entry:
    cli
    mov rsp, kernel_boot_stack_end
    cld
    and rsp, -16
    call lay_kernel_main

.halt:
    hlt
    jmp .halt
