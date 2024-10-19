ENTRY(init)

SECTIONS {
    .text : {
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)

        KEEP(*(.info))
    }

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
}