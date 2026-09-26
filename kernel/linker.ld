ENTRY(kernel_entry)

SECTIONS
{
    . = 0x20000;

    .text : ALIGN(16)
    {
        KEEP(*(.text.entry))
        *(.text*)
    }

    .rodata : ALIGN(16)
    {
        *(.rodata*)
    }

    .data : ALIGN(16)
    {
        *(.data*)
    }

    .bss : ALIGN(16)
    {
        *(COMMON)
        *(.bss*)
        . = ALIGN(16);
        kernel_boot_stack_start = .;
        . += 16384;
        kernel_boot_stack_end = .;
    }
}
