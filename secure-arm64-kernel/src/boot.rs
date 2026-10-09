#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    core::arch::naked_asm!(
        // Disable interrupts
        "msr daifset, #15",
        
        // Set up stack pointer at 16KB
        "movz x9, #0x4000",
        "movk x9, #0x8, lsl #16",
        "mov sp, x9",
        
        // Zero BSS section
        "ldr x0, =__bss_start",
        "ldr x1, =__bss_end",
        "2:",
        "cmp x0, x1",
        "beq 3f",
        "str xzr, [x0], #8",
        "b 2b",
        "3:",
        
        // Set exception vectors (use correct label: __vectors)
        "ldr x0, =__vectors",
        "msr vbar_el1, x0",
        "dsb sy",
        "isb",
        
        // Call Rust kernel_main
        "bl kernel_main",
        
        // Hang if we return
        "4:",
        "wfi",
        "b 4b",
    );
}

#[link_section = ".vectors"]
#[no_mangle]
pub static __vectors: [u64; 32] = [0; 32];
