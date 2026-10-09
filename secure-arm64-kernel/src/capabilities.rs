use core::fmt;

// ============================================================
// CAPABILITY RIGHTS
// ============================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CapabilityRights {
    Read,
    Write,
    Execute,
    Delegate,
    Revoke,
}

impl fmt::Display for CapabilityRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityRights::Read => write!(f, "READ"),
            CapabilityRights::Write => write!(f, "WRITE"),
            CapabilityRights::Execute => write!(f, "EXECUTE"),
            CapabilityRights::Delegate => write!(f, "DELEGATE"),
            CapabilityRights::Revoke => write!(f, "REVOKE"),
        }
    }
}

// ============================================================
// RESOURCE TYPES
// ============================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResourceType {
    Memory(u32),
    File(u32),
    Task(u32),
    Device(u32),
    Network(u32),
    Uart,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Memory(id) => write!(f, "Memory({})", id),
            ResourceType::File(id) => write!(f, "File({})", id),
            ResourceType::Task(id) => write!(f, "Task({})", id),
            ResourceType::Device(id) => write!(f, "Device({})", id),
            ResourceType::Network(id) => write!(f, "Network({})", id),
            ResourceType::Uart => write!(f, "UART"),
        }
    }
}

// ============================================================
// AUDIT LOG
// ============================================================

#[derive(Debug, Copy, Clone)]
pub struct AuditLogEntry {
    pub operation: AuditOperation,
    pub task_id: u32,
    pub success: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AuditOperation {
    Grant,
    Revoke,
    Check,
    Delegate,
}

impl fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditOperation::Grant => write!(f, "GRANT"),
            AuditOperation::Revoke => write!(f, "REVOKE"),
            AuditOperation::Check => write!(f, "CHECK"),
            AuditOperation::Delegate => write!(f, "DELEGATE"),
        }
    }
}

// ============================================================
// CAPABILITY STRUCTURE
// ============================================================

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Capability {
    pub id: u64,
    pub rights: CapabilityRights,
    pub resource: ResourceType,
    pub holder: u32,
    pub delegatable: bool,
}

impl Capability {
    pub fn new(
        id: u64,
        rights: CapabilityRights,
        resource: ResourceType,
        holder: u32,
        delegatable: bool,
    ) -> Self {
        Capability {
            id,
            rights,
            resource,
            holder,
            delegatable,
        }
    }

    pub fn grants(&self, rights: CapabilityRights) -> bool {
        self.rights == rights
    }

    pub fn grants_access_to(&self, resource: ResourceType) -> bool {
        self.resource == resource
    }
}

// ============================================================
// CAPABILITY MANAGER (32 slots - minimal for boot)
// ============================================================

pub struct CapabilityManager {
    capabilities: [Option<Capability>; 32],
    capability_count: usize,
    next_id: u64,
    audit_count: usize,
}

impl CapabilityManager {
    pub fn new() -> Self {
        let mut caps = [None; 32];
        // Pre-initialize all slots to None explicitly
        let mut i = 0;
        while i < 32 {
            caps[i] = None;
            i += 1;
        }
        
        CapabilityManager {
            capabilities: caps,
            capability_count: 0,
            next_id: 1,
            audit_count: 0,
        }
    }

    fn log_audit(&mut self, _operation: AuditOperation) {
        self.audit_count += 1;
    }

    pub fn grant_capability(
        &mut self,
        rights: CapabilityRights,
        resource: ResourceType,
        holder: u32,
        delegatable: bool,
    ) -> Result<u64, CapabilityError> {
        if self.capability_count >= 32 {
            return Err(CapabilityError::CapabilityTableFull);
        }

        let cap_id = self.next_id;
        self.next_id += 1;

        let capability = Capability::new(cap_id, rights, resource, holder, delegatable);

        self.capabilities[self.capability_count] = Some(capability);
        self.capability_count += 1;

        self.log_audit(AuditOperation::Grant);

        Ok(cap_id)
    }

    pub fn revoke_capability(&mut self, cap_id: u64) -> Result<(), CapabilityError> {
        let mut i = 0;
        while i < self.capability_count {
            if let Some(cap) = self.capabilities[i] {
                if cap.id == cap_id {
                    self.log_audit(AuditOperation::Revoke);

                    self.capabilities[i] = self.capabilities[self.capability_count - 1];
                    self.capabilities[self.capability_count - 1] = None;
                    self.capability_count -= 1;
                    return Ok(());
                }
            }
            i += 1;
        }

        Err(CapabilityError::CapabilityNotFound)
    }

    pub fn delegate_capability(
        &mut self,
        cap_id: u64,
        from_task: u32,
        to_task: u32,
    ) -> Result<u64, CapabilityError> {
        let mut cap = None;
        let mut i = 0;
        while i < self.capability_count {
            if let Some(c) = self.capabilities[i] {
                if c.id == cap_id {
                    cap = Some(c);
                    break;
                }
            }
            i += 1;
        }

        let cap = cap.ok_or(CapabilityError::CapabilityNotFound)?;

        if cap.holder != from_task {
            return Err(CapabilityError::NotCapabilityHolder);
        }

        if !cap.delegatable {
            return Err(CapabilityError::NotDelegatable);
        }

        let new_cap = Capability::new(
            self.next_id,
            cap.rights,
            cap.resource,
            to_task,
            false,
        );

        self.next_id += 1;

        if self.capability_count >= 32 {
            return Err(CapabilityError::CapabilityTableFull);
        }

        self.capabilities[self.capability_count] = Some(new_cap);
        self.capability_count += 1;

        self.log_audit(AuditOperation::Delegate);

        Ok(new_cap.id)
    }

    pub fn check_capability(
        &mut self,
        task_id: u32,
        rights: CapabilityRights,
        resource: ResourceType,
    ) -> Result<(), CapabilityError> {
        let mut i = 0;
        while i < self.capability_count {
            if let Some(cap) = self.capabilities[i] {
                if cap.holder == task_id
                    && cap.grants(rights)
                    && cap.grants_access_to(resource)
                {
                    self.log_audit(AuditOperation::Check);
                    return Ok(());
                }
            }
            i += 1;
        }

        self.log_audit(AuditOperation::Check);
        Err(CapabilityError::AccessDenied)
    }

    pub fn audit_log_count(&self) -> usize {
        self.audit_count
    }

    pub fn capability_count(&self) -> usize {
        self.capability_count
    }

    pub fn count_task_capabilities(&self, task_id: u32) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < self.capability_count {
            if let Some(c) = self.capabilities[i] {
                if c.holder == task_id {
                    count += 1;
                }
            }
            i += 1;
        }
        count
    }
}

// ============================================================
// CAPABILITY ERRORS
// ============================================================

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CapabilityError {
    CapabilityNotFound,
    CapabilityTableFull,
    AccessDenied,
    NotCapabilityHolder,
    NotDelegatable,
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CapabilityError::CapabilityNotFound => write!(f, "Capability not found"),
            CapabilityError::CapabilityTableFull => write!(f, "Capability table full"),
            CapabilityError::AccessDenied => write!(f, "Access denied"),
            CapabilityError::NotCapabilityHolder => write!(f, "Not capability holder"),
            CapabilityError::NotDelegatable => write!(f, "Capability not delegatable"),
        }
    }
}
