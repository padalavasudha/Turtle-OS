// Exception Handling for ARM64 Kernel
//
// When CPU exceptions occur (interrupts, traps, faults), the exception
// vector table routes them here. We save CPU state, handle the exception,
// and restore.

use core::arch::asm;

/// CPU Context - All registers saved during exception
/// This is what we save when an exception occurs
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExceptionContext {
    // General purpose registers (x0-x30)
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub x29: u64,
    pub x30: u64,
    
    // Special registers
    pub sp: u64,              // Stack pointer
    pub pc: u64,              // Program counter (where exception happened)
    pub pstate: u64,          // CPU flags
    pub exception_type: u64,  // Which exception (our custom field)
}

/// Exception types
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExceptionType {
    Synchronous,     // CPU-caused (syscall, undefined instr, etc)
    Irq,            // Interrupt
    Fiq,            // Fast interrupt
    SError,         // System error
    Unknown,
}

// ============================================================
// EXCEPTION VECTOR TABLE
// ============================================================

#[repr(C, align(2048))]
pub struct ExceptionVectors {
    pub vectors: [u64; 256],
}

#[link_section = ".vectors"]
#[no_mangle]
pub static EXCEPTION_VECTORS: ExceptionVectors = ExceptionVectors {
    vectors: [0; 256],
};

// ============================================================
// EXCEPTION HANDLERS (Assembly Entry Points)
// ============================================================

/// Synchronous exception handler (syscalls, undefined instructions, etc)
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn synchronous_exception_handler() -> ! {
    core::arch::naked_asm!(
        // Save all general purpose registers
        "sub sp, sp, #272",         // Allocate space for context (34 registers * 8 bytes)
        
        // Save x0-x30
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        
        // Save special registers to context
        "mrs x0, elr_el1",          // Exception Link Register (return address)
        "str x0, [sp, #248]",       // pc
        
        "mrs x0, spsr_el1",         // Saved Program State Register
        "str x0, [sp, #256]",       // pstate
        
        "mov x0, #1",               // Synchronous exception type
        "str x0, [sp, #264]",       // exception_type
        
        // Call Rust handler: handle_synchronous_exception(sp)
        "mov x0, sp",               // x0 = pointer to ExceptionContext
        "bl handle_synchronous_exception",
        
        // Restore all registers
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        
        // Restore special registers
        "ldr x0, [sp, #248]",
        "msr elr_el1, x0",
        
        "ldr x0, [sp, #256]",
        "msr spsr_el1, x0",
        
        "add sp, sp, #272",         // Deallocate space
        
        // Return from exception
        "eret",
    );
}

/// IRQ handler
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn irq_exception_handler() -> ! {
    core::arch::naked_asm!(
        // Save context (same as synchronous)
        "sub sp, sp, #272",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        
        "mrs x0, elr_el1",
        "str x0, [sp, #248]",
        
        "mrs x0, spsr_el1",
        "str x0, [sp, #256]",
        
        "mov x0, #2",               // IRQ exception type
        "str x0, [sp, #264]",
        
        // Call handler
        "mov x0, sp",
        "bl handle_irq_exception",
        
        // Restore (identical to synchronous)
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        
        "ldr x0, [sp, #248]",
        "msr elr_el1, x0",
        
        "ldr x0, [sp, #256]",
        "msr spsr_el1, x0",
        
        "add sp, sp, #272",
        "eret",
    );
}

/// FIQ handler
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn fiq_exception_handler() -> ! {
    core::arch::naked_asm!(
        "sub sp, sp, #272",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        
        "mrs x0, elr_el1",
        "str x0, [sp, #248]",
        
        "mrs x0, spsr_el1",
        "str x0, [sp, #256]",
        
        "mov x0, #3",               // FIQ exception type
        "str x0, [sp, #264]",
        
        "mov x0, sp",
        "bl handle_fiq_exception",
        
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        
        "ldr x0, [sp, #248]",
        "msr elr_el1, x0",
        
        "ldr x0, [sp, #256]",
        "msr spsr_el1, x0",
        
        "add sp, sp, #272",
        "eret",
    );
}

/// SError handler
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "C" fn serror_exception_handler() -> ! {
    core::arch::naked_asm!(
        "sub sp, sp, #272",
        "stp x0, x1, [sp, #0]",
        "stp x2, x3, [sp, #16]",
        "stp x4, x5, [sp, #32]",
        "stp x6, x7, [sp, #48]",
        "stp x8, x9, [sp, #64]",
        "stp x10, x11, [sp, #80]",
        "stp x12, x13, [sp, #96]",
        "stp x14, x15, [sp, #112]",
        "stp x16, x17, [sp, #128]",
        "stp x18, x19, [sp, #144]",
        "stp x20, x21, [sp, #160]",
        "stp x22, x23, [sp, #176]",
        "stp x24, x25, [sp, #192]",
        "stp x26, x27, [sp, #208]",
        "stp x28, x29, [sp, #224]",
        "str x30, [sp, #240]",
        
        "mrs x0, elr_el1",
        "str x0, [sp, #248]",
        
        "mrs x0, spsr_el1",
        "str x0, [sp, #256]",
        
        "mov x0, #4",               // SError exception type
        "str x0, [sp, #264]",
        
        "mov x0, sp",
        "bl handle_serror_exception",
        
        "ldp x0, x1, [sp, #0]",
        "ldp x2, x3, [sp, #16]",
        "ldp x4, x5, [sp, #32]",
        "ldp x6, x7, [sp, #48]",
        "ldp x8, x9, [sp, #64]",
        "ldp x10, x11, [sp, #80]",
        "ldp x12, x13, [sp, #96]",
        "ldp x14, x15, [sp, #112]",
        "ldp x16, x17, [sp, #128]",
        "ldp x18, x19, [sp, #144]",
        "ldp x20, x21, [sp, #160]",
        "ldp x22, x23, [sp, #176]",
        "ldp x24, x25, [sp, #192]",
        "ldp x26, x27, [sp, #208]",
        "ldp x28, x29, [sp, #224]",
        "ldr x30, [sp, #240]",
        
        "ldr x0, [sp, #248]",
        "msr elr_el1, x0",
        
        "ldr x0, [sp, #256]",
        "msr spsr_el1, x0",
        
        "add sp, sp, #272",
        "eret",
    );
}

// ============================================================
// HANDLER FUNCTIONS (Rust handlers called by assembly)
// ============================================================

/// Handle synchronous exception (syscalls, undefined instructions, etc)
#[no_mangle]
pub extern "C" fn handle_synchronous_exception(ctx: &mut ExceptionContext) {
    uart_write_string("\n[EXCEPTION] Synchronous Exception\n");
    uart_write_string("  PC: 0x");
    uart_write_hex(ctx.pc);
    uart_write_string("\n");
    uart_write_string("  X0: 0x");
    uart_write_hex(ctx.x0);
    uart_write_string("\n");
}

/// Handle interrupt (IRQ)
#[no_mangle]
pub extern "C" fn handle_irq_exception(ctx: &mut ExceptionContext) {
    uart_write_string("\n[EXCEPTION] IRQ\n");
    uart_write_string("  PC: 0x");
    uart_write_hex(ctx.pc);
    uart_write_string("\n");
}

/// Handle fast interrupt (FIQ)
#[no_mangle]
pub extern "C" fn handle_fiq_exception(ctx: &mut ExceptionContext) {
    uart_write_string("\n[EXCEPTION] FIQ\n");
    uart_write_string("  PC: 0x");
    uart_write_hex(ctx.pc);
    uart_write_string("\n");
}

/// Handle system error (SError)
#[no_mangle]
pub extern "C" fn handle_serror_exception(ctx: &mut ExceptionContext) {
    uart_write_string("\n[EXCEPTION] SError (System Error)\n");
    uart_write_string("  PC: 0x");
    uart_write_hex(ctx.pc);
    uart_write_string("\n");
    uart_write_string("  FATAL - Entering idle loop\n");
    loop {}
}

// ============================================================
// UART HELPERS
// ============================================================

const UART0: *mut u8 = 0x09000000 as *mut u8;

unsafe fn uart_write_char(c: u8) {
    core::ptr::write_volatile(UART0, c);
}

fn uart_write_string(s: &str) {
    for byte in s.bytes() {
        unsafe {
            uart_write_char(byte);
        }
    }
}

fn uart_write_hex(n: u64) {
    let hex_digits = "0123456789abcdef";
    let mut printed = false;
    
    for i in (0..16).rev() {
        let digit = ((n >> (i * 4)) & 0xF) as usize;
        if digit != 0 || printed || i == 0 {
            unsafe {
                uart_write_char(hex_digits.as_bytes()[digit]);
            }
            printed = true;
        }
    }
}

// ============================================================
// INITIALIZE EXCEPTION VECTORS
// ============================================================

pub fn init_exception_vectors() {
    unsafe {
        // Set VBAR_EL1 to point to our exception vector table
        asm!(
            "ldr x0, =EXCEPTION_VECTORS",
            "msr vbar_el1, x0",
            "dsb sy",
            "isb",
        );
    }
}
