ENTRY(_start)

SECTIONS
{
    /* Dizemos ao chip para carregar o UniqLoader no endereço 1MB da RAM */
    . = 1M;

    .text :
    {
        /* Garante que o código de inicialização seja o primeiro byte */
        *(.text .text.*)
    }

    .rodata :
    {
        *(.rodata .rodata.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }
}