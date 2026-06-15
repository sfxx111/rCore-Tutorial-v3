OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
        
        /* 核心修改：在 .text 段末尾强制一页（4096字节）对齐，并导出 strampoline 地址 */
        . = ALIGN(4096);
        strampoline = .;
        *(.text.trampoline)
        . = ALIGN(4096);
    }

    etext = .;
    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    edata = .;
    .bss : {
        *(.bss.stack)
        sbss_with_stack = .;
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    ebss = .;
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
