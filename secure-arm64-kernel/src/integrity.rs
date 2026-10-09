// Week 6: Integrity Monitoring System
// Detects rootkits and kernel tampering

use core::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntegrityLevel {
    Critical,   // Kernel code
    High,       // System tables
    Medium,     // User data
    Low,        // Logs
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryRegion {
    pub start: u64,
    pub end: u64,
    pub level: IntegrityLevel,
    pub hash: u64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IntegrityStatus {
    Valid,
    Modified,
    Corrupted,
    Missing,
}

#[derive(Debug, Copy, Clone)]
pub struct IntegrityCheckResult {
    pub region_id: usize,
    pub status: IntegrityStatus,
    pub expected_hash: u64,
    pub actual_hash: u64,
}

pub struct IntegrityMonitor {
    regions: [Option<MemoryRegion>; 16],
    region_count: usize,
    checks_performed: usize,
    violations_detected: usize,
}

impl IntegrityMonitor {
    pub fn new() -> Self {
        IntegrityMonitor {
            regions: [None; 16],
            region_count: 0,
            checks_performed: 0,
            violations_detected: 0,
        }
    }

    pub fn register_region(
        &mut self,
        start: u64,
        end: u64,
        level: IntegrityLevel,
    ) -> Result<usize, IntegrityError> {
        if self.region_count >= 16 {
            return Err(IntegrityError::TooManyRegions);
        }

        if start >= end {
            return Err(IntegrityError::InvalidRegion);
        }

        let hash = self.compute_hash(start, end);
        let region = MemoryRegion {
            start,
            end,
            level,
            hash,
        };

        let region_id = self.region_count;
        self.regions[self.region_count] = Some(region);
        self.region_count += 1;

        Ok(region_id)
    }

    pub fn check_integrity(&mut self, region_id: usize) -> Result<IntegrityCheckResult, IntegrityError> {
        if region_id >= self.region_count {
            return Err(IntegrityError::RegionNotFound);
        }

        let region = self.regions[region_id].ok_or(IntegrityError::RegionNotFound)?;

        let actual_hash = self.compute_hash(region.start, region.end);
        self.checks_performed += 1;

        let status = if actual_hash == region.hash {
            IntegrityStatus::Valid
        } else {
            self.violations_detected += 1;
            IntegrityStatus::Modified
        };

        Ok(IntegrityCheckResult {
            region_id,
            status,
            expected_hash: region.hash,
            actual_hash,
        })
    }

    pub fn check_all(&mut self) -> usize {
        let mut violations = 0;
        for i in 0..self.region_count {
            if let Ok(result) = self.check_integrity(i) {
                if result.status != IntegrityStatus::Valid {
                    violations += 1;
                }
            }
        }
        violations
    }

    fn compute_hash(&self, start: u64, end: u64) -> u64 {
        // Simple FNV-1a hash for memory regions
        let mut hash = 0xcbf29ce484222325u64;
        let mut addr = start;
        
        while addr < end && addr < start + 256 {
            let byte = unsafe { core::ptr::read_volatile(addr as *const u8) };
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
            addr += 1;
        }
        
        hash
    }

    pub fn checks_performed(&self) -> usize {
        self.checks_performed
    }

    pub fn violations_detected(&self) -> usize {
        self.violations_detected
    }

    pub fn region_count(&self) -> usize {
        self.region_count
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IntegrityError {
    RegionNotFound,
    TooManyRegions,
    InvalidRegion,
}

impl fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntegrityError::RegionNotFound => write!(f, "Region not found"),
            IntegrityError::TooManyRegions => write!(f, "Too many regions"),
            IntegrityError::InvalidRegion => write!(f, "Invalid region"),
        }
    }
}
