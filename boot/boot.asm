; Lay OS — x86_64 boot sector
; First-stage BIOS loader for the initial development image.

bits 16
org 0x7C00

start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7C00

    mov [boot_drive], dl

    ; Development image reserves the first 128 sectors after the boot sector
    ; for the kernel image.
    mov si, dap
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    jc disk_error

    in al, 0x92
    or al, 00000010b
    out 0x92, al

    lgdt [gdt_descriptor]

    mov eax, cr0
    or eax, 1
    mov cr0, eax

    jmp 0x08:protected_mode

disk_error:
    mov si, error_message
.print:
    lodsb
    test al, al
    jz .halt
    mov ah, 0x0E
    int 0x10
    jmp .print
.halt:
    cli
    hlt
    jmp .halt

bits 32
protected_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov edi, 0x7000
    xor eax, eax
    mov ecx, 0x3000 / 4
    rep stosd

    mov dword [0x7000], 0x8000 | 0x3
    mov dword [0x8000], 0x9000 | 0x3

    mov edi, 0x9000
    mov eax, 0x83
    mov ecx, 512
.map_pd:
    mov [edi], eax
    add eax, 0x200000
    add edi, 8
    loop .map_pd

    mov eax, 0x7000
    mov cr3, eax

    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    lgdt [gdt64_descriptor]
    jmp 0x18:long_mode

bits 64
long_mode:
    mov ax, 0x20
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, 0x90000

    jmp 0x1000

boot_drive db 0

dap:
    db 0x10
    db 0
    dw 128
    dw 0x1000
    dw 0
    dd 1
    dd 0

error_message db "LAY BOOT: disk read failed", 0

align 8
gdt:
    dq 0x0000000000000000
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
    dq 0x00AF9A000000FFFF
    dq 0x00AF92000000FFFF

gdt_descriptor:
    dw gdt_descriptor - gdt - 1
    dd gdt

gdt64_descriptor:
    dw gdt64_descriptor - gdt - 1
    dq gdt

times 510-($-$$) db 0
dw 0xAA55
