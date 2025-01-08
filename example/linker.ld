OUTPUT_ARCH(aarch64)
ENTRY(_start)

/* BASE_ADDRESS = 0x00080000; */
BASE_ADDRESS = 0x40080000;
/* BASE_ADDRESS = 0xffffff8040080000; */
/* 0xffffff8000000000 */
SECTIONS
{
    /* Load the kernel at this address: "." means the current address */
    . = BASE_ADDRESS;
    _skernel = .;

    .text ALIGN(4K): {
        stext = .;
        *(.text._start );
        *(.text .text.*)
        etext = .;
    }

    .rodata ALIGN(4K): {
        srodata = .;
        *(.rodata .rodata.*)
        erodata = .;
    }

    .data ALIGN(4K): {
        _sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
        _edata = .;
    }
    
    _load_end = .;

    .bss ALIGN(4K): {
        *(.bss.bstack*)
        _sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        _ebss = .;
    }

    PROVIDE(end = .);
    /DISCARD/ : {
        *(.comment) *(.gnu*) *(.note*) *(.eh_frame*) *(.debug*)
    }
}