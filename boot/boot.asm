; Lay OS x86_64 BIOS loader.
; Kernel is loaded at 0x20000, safely away from the BIOS boot sector at 0x7C00.
; A fixed 1024-sector development window is read in <=127-sector transfers.
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
    mov word [remaining], 1024
    mov dword [dest_phys], 0x20000
    mov dword [current_lba], 1
    mov dword [current_lba+4], 0

load_loop:
    mov ax, [remaining]
    cmp ax, 127
    jbe .count_ready
    mov ax, 127
.count_ready:
    mov [dap_count], ax

    mov eax, [dest_phys]
    shr eax, 4
    mov [dap_segment], ax

    mov eax, [current_lba]
    mov [dap_lba], eax
    mov eax, [current_lba+4]
    mov [dap_lba+4], eax

    mov si, dap
    mov ah, 0x42
    mov dl, [boot_drive]
    int 0x13
    jc disk_error

    mov dx, [dap_count]
    sub [remaining], dx

    movzx eax, word [dap_count]
    add dword [current_lba], eax
    adc dword [current_lba+4], 0

    shl eax, 9
    add dword [dest_phys], eax

    cmp word [remaining], 0
    jne load_loop

    call detect_memory

    in al, 0x92
    or al, 0x02
    out 0x92, al

    lgdt [gdt_descriptor]

    mov eax, cr0
    or eax, 0x01
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

; Detects available RAM via BIOS INT 0x15, EAX=0xE820 while still in real
; mode (DS=ES=0 from boot start, so linear == physical for these writes).
; Writes:
;   0x8FF0 (word)  -> number of entries found (0 if unsupported/failed)
;   0x9000 (bytes) -> up to 64 raw 24-byte E820 entries, back to back
; See docs/MEMORY_MAP.md for the full contract read by kernel/memory.rs.
detect_memory:
    mov di, 0x9000
    xor ebx, ebx
    xor bp, bp
    mov edx, 0x0534D4150
.e820lp:
    mov eax, 0xe820
    mov ecx, 24
    int 0x15
    jc .e820done
    cmp eax, 0x0534D4150
    jne .e820done
    cmp cl, 20
    jbe .e820ok
    test byte [di + 20], 1
    je .e820skip
.e820ok:
    mov ecx, [di + 8]
    or ecx, [di + 12]
    jz .e820skip
    inc bp
    add di, 24
    cmp bp, 64
    jae .e820done
.e820skip:
    test ebx, ebx
    jnz .e820lp
.e820done:
    mov [0x8FF0], bp
    ret

bits 32
protected_mode:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; Page tables live at 1 MiB, above the kernel load window.
    mov edi, 0x100000
    xor eax, eax
    mov ecx, 0x3000 / 4
    rep stosd

    mov dword [0x100000], 0x101000 | 0x7
    mov dword [0x101000], 0x102000 | 0x7

    mov edi, 0x102000
    mov eax, 0x00000083
    mov ecx, 512
.map_pd:
    mov [edi], eax
    add eax, 0x00200000
    add edi, 8
    loop .map_pd

    mov eax, 0x100000
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
    jmp 0x20000

boot_drive db 0
remaining dw 0
dest_phys dd 0
current_lba dq 0

dap:
    db 0x10, 0
dap_count:
    dw 127
    dw 0
dap_segment:
    dw 0
dap_lba:
    dq 1

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
