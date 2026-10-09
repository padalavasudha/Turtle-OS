/* ARM64 Bare-Metal Linker Script for QEMU */

ENTRY(_start)

MEMORY
{
    /* QEMU virt machine memory layout:
       0x00000000 - System ROM (QEMU internals)
       0x08000000 - UART0 (serial output)
       0x09000000 - UART0 (backup address)
       0x40000000 - Flash/PLIC
       0x80000000 - RAM (where our kernel loads)
       
       We load our kernel at 0x80000 (within the RAM region)
    */
    ram : ORIGIN = 0x80000, LENGTH = 256M
}

SECTIONS
{
    /* Kernel code section - starts at ORIGIN (0x80000) */
    .text :
    {
        /* Symbol for start of text section */
        __text_start = .;
        
        /* Boot code MUST be first - _start needs to be at 0x80000 */
        *(.text.boot)
        
        /* Then all other code */
        *(.text*)
        
        /* Read-only data (constants, strings) */
        *(.rodata*)
        
        /* Symbol for end of text section */
        __text_end = .;
    } > ram
    
    /* Initialized data section (global variables with initial values) */
    .data :
    {
        __data_start = .;
        *(.data*)
        __data_end = .;
    } > ram
    
    /* Uninitialized data section (global variables, zeroed at boot) */
    .bss :
    {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > ram
    
    /* Discard unused sections */
    /DISCARD/ :
    {
        *(.note.GNU-stack)
        *(.gnu_debuglink)
        *(.gnu_debugdata)
    }
}

/* Security: Ensure no executable stack (extra safety) */
PHDRS
{
    text PT_LOAD FLAGS(5);  /* R-X (read, execute) */
    data PT_LOAD FLAGS(6);  /* RW- (read, write) */
}
