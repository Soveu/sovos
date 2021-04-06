ENTRY(_start)

SECTIONS {
  . = 0xffffffffc0000000;

  .text   : ALIGN(2M) { *(.text) }
  .rodata : ALIGN(2M) { *(.rodata) }
  .data   : ALIGN(2M) { *(.data) *(.bss) }
}
