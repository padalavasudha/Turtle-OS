// Week 7: Rootkit Detection System
// Detects unauthorized kernel modifications and syscall hooks

use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RootkitType {
    SyscallHook,
    InterruptHook,
    MemoryHook,
    CapabilityBypass,
    ExceptionHandler,
}

#[derive(Debug, Copy, Clone)]
pub struct RootkitSignature {
    pub sig_type: RootkitType,
    pub address: u64,
    pub expected_value: u64,
    pub actual_value: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DetectionResult {
    Clean,
    Suspicious,
    Modified,
    Infected,
    Critical,
}

pub struct RootkitDetector {
    syscall_table: u64,
    exception_handlers: [u64; 32],
    memory_integrity_baseline: u64,
    detections: [Option<RootkitSignature>; 8],
    detection_count: usize,
    scans_performed: usize,
    threats_detected: usize,
}

impl RootkitDetector {
    pub fn new(syscall_table: u64) -> Self {
        RootkitDetector {
            syscall_table,
            exception_handlers: [0; 32],
            memory_integrity_baseline: 0,
            detections: [None; 8],
            detection_count: 0,
            scans_performed: 0,
            threats_detected: 0,
        }
    }

    pub fn set_baseline(&mut self, baseline: u64) {
        self.memory_integrity_baseline = baseline;
    }

    pub fn scan_syscalls(&mut self) -> DetectionResult {
        self.scans_performed += 1;
        
        // Read syscall table and check for hooks
        let syscall_ptr = self.syscall_table as *const u64;
        unsafe {
            for i in 0..8 {
                let entry = core::ptr::read_volatile(syscall_ptr.add(i));
                // Simple check: syscalls should point to kernel space
                if entry > 0xFFFF000000000000 || entry == 0 {
                    self.threats_detected += 1;
                    return DetectionResult::Infected;
                }
            }
        }
        
        DetectionResult::Clean
    }

    pub fn scan_exceptions(&mut self) -> DetectionResult {
        self.scans_performed += 1;
        
        // Read VBAR_EL1 exception vector base
        let vbar: u64;
        unsafe {
            core::arch::asm!("mrs {}, vbar_el1", out(reg) vbar);
        }
        
        // Check if exception handlers were modified
        let handler_ptr = vbar as *const u64;
        unsafe {
            for i in 0..4 {
                let handler = core::ptr::read_volatile(handler_ptr.add(i));
                if handler == 0 || handler > 0xFFFF000000000000 {
                    self.threats_detected += 1;
                    return DetectionResult::Suspicious;
                }
            }
        }
        
        DetectionResult::Clean
    }

    pub fn scan_memory_integrity(&mut self) -> DetectionResult {
        self.scans_performed += 1;
        
        // Verify kernel text section hasn't been modified
        let kernel_text_start: u64 = 0x80000;
        let kernel_text_end: u64 = 0x100000;
        
        let mut hash = 0xcbf29ce484222325u64;
        let mut addr = kernel_text_start;
        
        while addr < kernel_text_end && addr < kernel_text_start + 512 {
            let byte = unsafe { core::ptr::read_volatile(addr as *const u8) };
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            addr += 1;
        }
        
        if hash == self.memory_integrity_baseline || self.memory_integrity_baseline == 0 {
            DetectionResult::Clean
        } else {
            self.threats_detected += 1;
            DetectionResult::Modified
        }
    }

    pub fn full_scan(&mut self) -> DetectionResult {
        let syscall_result = self.scan_syscalls();
        let exception_result = self.scan_exceptions();
        let memory_result = self.scan_memory_integrity();
        
        // Return worst result
        if syscall_result == DetectionResult::Infected 
            || exception_result == DetectionResult::Infected 
            || memory_result == DetectionResult::Infected {
            DetectionResult::Infected
        } else if syscall_result == DetectionResult::Suspicious 
            || exception_result == DetectionResult::Suspicious 
            || memory_result == DetectionResult::Suspicious {
            DetectionResult::Suspicious
        } else if syscall_result == DetectionResult::Modified
            || exception_result == DetectionResult::Modified
            || memory_result == DetectionResult::Modified {
            DetectionResult::Modified
        } else {
            DetectionResult::Clean
        }
    }

    pub fn scans_performed(&self) -> usize {
        self.scans_performed
    }

    pub fn threats_detected(&self) -> usize {
        self.threats_detected
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RootkitError {
    InvalidAddress,
    ScanFailed,
    TableNotInitialized,
}

impl fmt::Display for RootkitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RootkitError::InvalidAddress => write!(f, "Invalid address"),
            RootkitError::ScanFailed => write!(f, "Scan failed"),
            RootkitError::TableNotInitialized => write!(f, "Table not initialized"),
        }
    }
}
