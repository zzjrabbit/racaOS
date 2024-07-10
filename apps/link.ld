ENTRY(_start)

USER_BEGIN = 0x120000000000;

SECTIONS {

  . = USER_BEGIN;

  .rodata ALIGN(4K):
  {
    *(.rodata .rodata.*)
  }

  .text ALIGN(4K):
  {
    *(.text .text.*)
  }

  .data ALIGN(4K):
  {
    *(.data .data.*)
    *(.sdata .sdata.*)
  }

  .got ALIGN(4K):
  {
    *(.got .got.*)
  }

  .sbss : 
  {
    *(.sbss)
    *(.sbss.*)
    *(.dynsbss)
    *(.scommon)
  }

  .bss :
  {
    *(.bss)
    *(.bss.*)
    *(.dynbss)
  }
}
