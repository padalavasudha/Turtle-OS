use std::io::{self, Write, BufRead, BufReader};
use std::collections::HashMap;

const DUSTY_ROSE_LIGHT: &str = "\x1b[38;2;191;100;100m";
const DUSTY_ROSE_DARK: &str = "\x1b[38;2;160;75;75m";
const DUSTY_ROSE_BOLD: &str = "\x1b[1;38;2;191;100;100m";
const RESET: &str = "\x1b[0m";

const TURTLE_LOGO: &str = concat!(
    "          ########\n",
    "       ##############\n",
    "     ##################\n",
    "   ###  ############  ####\n",
    "  ##########################\n",
    "   ########################\n",
    "      ####  ####  ####\n",
    "      ###    ##    ###"
);

#[derive(Clone, Debug)]
struct CapabilityType {
    name: &'static str,
    description: &'static str,
    risk_level: RiskLevel,
    can_delegate: bool,
    requires_auth: bool,
}

#[derive(Clone, Debug, PartialEq)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    fn as_str(&self) -> &str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        }
    }
}

#[derive(Clone, Debug)]
struct IntegrityLevelDef {
    name: &'static str,
    description: &'static str,
    check_frequency: &'static str,
    priority: u8,
}

#[derive(Clone, Debug)]
struct RoleTypeDef {
    name: &'static str,
    description: &'static str,
    default_capabilities: &'static [&'static str],
    max_resources: u32,
    privilege_level: u8,
}

const CAPABILITY_TYPES: &[CapabilityType] = &[
    CapabilityType {
        name: "READ",
        description: "Read data from files, memory, and resources",
        risk_level: RiskLevel::Low,
        can_delegate: true,
        requires_auth: false,
    },
    CapabilityType {
        name: "WRITE",
        description: "Write and modify files and memory contents",
        risk_level: RiskLevel::Medium,
        can_delegate: true,
        requires_auth: true,
    },
    CapabilityType {
        name: "EXECUTE",
        description: "Execute code, syscalls, and kernel operations",
        risk_level: RiskLevel::High,
        can_delegate: false,
        requires_auth: true,
    },
    CapabilityType {
        name: "DELETE",
        description: "Delete files, resources, and data permanently",
        risk_level: RiskLevel::High,
        can_delegate: false,
        requires_auth: true,
    },
    CapabilityType {
        name: "DELEGATE",
        description: "Delegate capabilities to other tasks/roles",
        risk_level: RiskLevel::Critical,
        can_delegate: false,
        requires_auth: true,
    },
    CapabilityType {
        name: "ADMIN",
        description: "Full system administration and kernel access",
        risk_level: RiskLevel::Critical,
        can_delegate: false,
        requires_auth: true,
    },
];

const INTEGRITY_LEVELS: &[IntegrityLevelDef] = &[
    IntegrityLevelDef {
        name: "CRITICAL",
        description: "Kernel code, exception handlers, core data structures",
        check_frequency: "Every syscall (continuous)",
        priority: 1,
    },
    IntegrityLevelDef {
        name: "HIGH",
        description: "System libraries, device drivers, security services",
        check_frequency: "Every 10ms",
        priority: 2,
    },
    IntegrityLevelDef {
        name: "MEDIUM",
        description: "User applications, configuration files, shared libraries",
        check_frequency: "Every 100ms",
        priority: 3,
    },
    IntegrityLevelDef {
        name: "LOW",
        description: "Temporary data, cache, logs, user files",
        check_frequency: "On demand or every 1 second",
        priority: 4,
    },
];

const ROLE_TYPES: &[RoleTypeDef] = &[
    RoleTypeDef {
        name: "kernel",
        description: "Kernel process with full privileges",
        default_capabilities: &["READ", "WRITE", "EXECUTE", "DELETE", "ADMIN"],
        max_resources: 1000,
        privilege_level: 0,
    },
    RoleTypeDef {
        name: "driver",
        description: "Hardware driver with privileged access",
        default_capabilities: &["READ", "WRITE", "EXECUTE"],
        max_resources: 500,
        privilege_level: 1,
    },
    RoleTypeDef {
        name: "service",
        description: "System service (networking, storage, etc)",
        default_capabilities: &["READ", "WRITE", "EXECUTE"],
        max_resources: 300,
        privilege_level: 2,
    },
    RoleTypeDef {
        name: "task",
        description: "User-mode task/process with limited access",
        default_capabilities: &["READ"],
        max_resources: 100,
        privilege_level: 3,
    },
    RoleTypeDef {
        name: "user",
        description: "Regular user account with minimal privileges",
        default_capabilities: &["READ"],
        max_resources: 50,
        privilege_level: 4,
    },
];

struct KernelSimulator {
    capabilities: HashMap<usize, Capability>,
    capability_counter: usize,
    files: HashMap<String, File>,
    integrity_regions: Vec<IntegrityRegion>,
    is_running: bool,
}

struct Capability {
    id: usize,
    cap_type: String,
    owner: String,
}

struct File {
    name: String,
    max_size: usize,
    content: String,
}

struct IntegrityRegion {
    start: u64,
    end: u64,
    level: String,
}

impl KernelSimulator {
    fn new() -> Self {
        KernelSimulator {
            capabilities: HashMap::new(),
            capability_counter: 0,
            files: HashMap::new(),
            integrity_regions: Vec::new(),
            is_running: true,
        }
    }

    fn start(&self) {
        println!("\n{}{}{}\n", DUSTY_ROSE_LIGHT, TURTLE_LOGO, RESET);
        println!("{}========================================{}", DUSTY_ROSE_DARK, RESET);
        println!("{}SECURE ARM64 KERNEL SIMULATOR v1.0{}", DUSTY_ROSE_LIGHT, RESET);
        println!("{}========================================{}\n", DUSTY_ROSE_DARK, RESET);
        println!("System Configuration:");
        println!("  Capability Types: {}", CAPABILITY_TYPES.len());
        println!("  Integrity Levels: {}", INTEGRITY_LEVELS.len());
        println!("  Role Types: {}", ROLE_TYPES.len());
        println!();
        println!("Initializing kernel subsystems...");
        println!("  Capability system: {}OK{}", DUSTY_ROSE_LIGHT, RESET);
        println!("  Integrity monitor: {}OK{}", DUSTY_ROSE_LIGHT, RESET);
        println!("  Rootkit detector: {}OK{}\n", DUSTY_ROSE_LIGHT, RESET);
        println!("{}========================================{}", DUSTY_ROSE_DARK, RESET);
        println!("{}KERNEL READY - INTERACTIVE SHELL{}", DUSTY_ROSE_BOLD, RESET);
        println!("{}========================================{}\n", DUSTY_ROSE_DARK, RESET);
        println!("Type 'help' for available commands\n");
    }

    fn has_capability(&self, owner: &str, cap_type: &str) -> bool {
        self.capabilities.values().any(|c| {
            c.owner == owner && c.cap_type == cap_type
        })
    }

    fn process_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() { return; }

        match parts[0] {
            "help" => self.cmd_help(),
            "info" => self.cmd_info(),
            "cap-types" => self.cmd_cap_types(),
            "integrity-levels" => self.cmd_integrity_levels(),
            "roles" => self.cmd_roles(),
            "cap-grant" => self.cmd_cap_grant(&parts),
            "cap-list" => self.cmd_cap_list(),
            "cap-check" => self.cmd_cap_check(&parts),
            "cap-revoke" => self.cmd_cap_revoke(&parts),
            "file-create" => self.cmd_file_create(&parts),
            "file-list" => self.cmd_file_list(),
            "file-read-as" => self.cmd_file_read_as(&parts),
            "file-write-as" => self.cmd_file_write_as(&parts),
            "file-delete-as" => self.cmd_file_delete_as(&parts),
            "integrity-register" => self.cmd_integrity_register(&parts),
            "integrity-list" => self.cmd_integrity_list(),
            "status" => self.cmd_status(),
            "exit" | "quit" => self.cmd_exit(),
            "" => {},
            _ => println!("Unknown command: '{}'. Type 'help' for available commands.\n", parts[0]),
        }
    }

    fn cmd_help(&self) {
        println!("\n=== AVAILABLE COMMANDS ===\n");
        println!("DEFINITIONS:");
        println!("  cap-types                              - Show all capability types");
        println!("  integrity-levels                       - Show all integrity levels");
        println!("  roles                                  - Show all role types\n");
        println!("CAPABILITY MANAGEMENT:");
        println!("  cap-grant <owner> <type>               - Grant capability");
        println!("  cap-list                               - List all capabilities");
        println!("  cap-check <owner> <type>               - Check if owner has capability");
        println!("  cap-revoke <cap_id>                    - Revoke capability\n");
        println!("FILE OPERATIONS (WITH PERMISSION CHECK):");
        println!("  file-create <name> <size>              - Create file");
        println!("  file-list                              - List files");
        println!("  file-read-as <owner> <filename>        - Read file (checks READ cap)");
        println!("  file-write-as <owner> <filename> <msg> - Write file (checks WRITE cap)");
        println!("  file-delete-as <owner> <filename>      - Delete file (checks DELETE cap)\n");
        println!("INTEGRITY MONITORING:");
        println!("  integrity-register <s> <e> <l>         - Register region");
        println!("  integrity-list                         - List regions\n");
        println!("SYSTEM:");
        println!("  info                                   - Show system information");
        println!("  status                                 - Show system status");
        println!("  exit                                   - Exit simulator\n");
    }

    fn cmd_info(&self) {
        println!("\n========================================");
        println!("SYSTEM INFORMATION");
        println!("========================================\n");
        println!("Capability Types Defined: {}", CAPABILITY_TYPES.len());
        println!("Integrity Levels Defined: {}", INTEGRITY_LEVELS.len());
        println!("Role Types Defined: {}", ROLE_TYPES.len());
        println!("\nType specific commands to see details:");
        println!("  cap-types");
        println!("  integrity-levels");
        println!("  roles\n");
    }

    fn cmd_cap_types(&self) {
        println!("\n========================================");
        println!("CAPABILITY TYPES");
        println!("========================================\n");
        for cap in CAPABILITY_TYPES {
            println!("{}[{}]{}", DUSTY_ROSE_BOLD, cap.name, RESET);
            println!("  Description: {}", cap.description);
            println!("  Risk Level: {}", cap.risk_level.as_str());
            println!("  Can Delegate: {}", if cap.can_delegate { "Yes" } else { "No" });
            println!("  Requires Auth: {}", if cap.requires_auth { "Yes" } else { "No" });
            println!();
        }
    }

    fn cmd_integrity_levels(&self) {
        println!("\n========================================");
        println!("INTEGRITY LEVELS");
        println!("========================================\n");
        for level in INTEGRITY_LEVELS {
            println!("{}[{}]{} (Priority: {})", DUSTY_ROSE_BOLD, level.name, RESET, level.priority);
            println!("  Description: {}", level.description);
            println!("  Check Frequency: {}", level.check_frequency);
            println!();
        }
    }

    fn cmd_roles(&self) {
        println!("\n========================================");
        println!("ROLE TYPES");
        println!("========================================\n");
        for role in ROLE_TYPES {
            println!("{}[{}]{} (Privilege Level: {})", DUSTY_ROSE_BOLD, role.name, RESET, role.privilege_level);
            println!("  Description: {}", role.description);
            println!("  Default Capabilities: {}", role.default_capabilities.join(", "));
            println!("  Max Resources: {}", role.max_resources);
            println!();
        }
    }

    fn cmd_cap_grant(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            println!("Usage: cap-grant <owner> <type>\n");
            return;
        }
        let cap_type = parts[2].to_uppercase();
        if !CAPABILITY_TYPES.iter().any(|c| c.name == cap_type) {
            println!("Error: Invalid capability type '{}'\n", parts[2]);
            return;
        }
        let id = self.capability_counter;
        self.capability_counter += 1;
        self.capabilities.insert(id, Capability {
            id,
            cap_type: cap_type.clone(),
            owner: parts[1].to_string(),
        });
        println!("\n{}✓ Capability granted successfully{}", DUSTY_ROSE_LIGHT, RESET);
        println!("  ID: {}", id);
        println!("  Owner: {}", parts[1]);
        println!("  Type: {}\n", cap_type);
    }

    fn cmd_cap_list(&self) {
        println!("\n=== CAPABILITIES ({} total) ===\n", self.capabilities.len());
        if self.capabilities.is_empty() {
            println!("No capabilities in system.\n");
            return;
        }
        for (id, cap) in &self.capabilities {
            println!("  [{}] {} - Owner: {}", id, cap.cap_type, cap.owner);
        }
        println!();
    }

    fn cmd_cap_check(&self, parts: &[&str]) {
        if parts.len() < 3 {
            println!("Usage: cap-check <owner> <type>\n");
            return;
        }
        let owner = parts[1];
        let cap_type = parts[2].to_uppercase();
        if self.has_capability(owner, &cap_type) {
            println!("\n{}✓ {} has {} capability{}\n", DUSTY_ROSE_LIGHT, owner, cap_type, RESET);
        } else {
            println!("\n{}✗ {} does NOT have {} capability{}\n", DUSTY_ROSE_DARK, owner, cap_type, RESET);
        }
    }

    fn cmd_cap_revoke(&mut self, parts: &[&str]) {
        if parts.len() < 2 {
            println!("Usage: cap-revoke <cap_id>\n");
            return;
        }
        if let Ok(cap_id) = parts[1].parse::<usize>() {
            if self.capabilities.remove(&cap_id).is_some() {
                println!("\n{}✓ Capability {} revoked successfully{}\n", DUSTY_ROSE_LIGHT, cap_id, RESET);
            } else {
                println!("\nError: Capability {} not found\n", cap_id);
            }
        }
    }

    fn cmd_file_create(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            println!("Usage: file-create <filename> <max_size>\n");
            return;
        }
        let filename = parts[1].to_string();
        let max_size = parts[2].parse::<usize>().unwrap_or(0);
        if max_size == 0 {
            println!("Error: Invalid size\n");
            return;
        }
        self.files.insert(filename.clone(), File {
            name: filename.clone(),
            max_size,
            content: String::new(),
        });
        println!("\n{}✓ File created: {}{}\n", DUSTY_ROSE_LIGHT, filename, RESET);
    }

    fn cmd_file_list(&self) {
        println!("\n=== FILES ({} total) ===\n", self.files.len());
        if self.files.is_empty() {
            println!("No files in system.\n");
            return;
        }
        for (_, file) in &self.files {
            println!("  {} - {}/{} bytes", file.name, file.content.len(), file.max_size);
        }
        println!();
    }

    fn cmd_file_read_as(&self, parts: &[&str]) {
        if parts.len() < 3 {
            println!("Usage: file-read-as <owner> <filename>\n");
            return;
        }
        let owner = parts[1];
        let filename = parts[2];
        if !self.has_capability(owner, "READ") {
            println!("\n{}✗ PERMISSION DENIED{}", DUSTY_ROSE_DARK, RESET);
            println!("  {} lacks READ capability\n", owner);
            return;
        }
        if let Some(file) = self.files.get(filename) {
            println!("\n{}✓ READ ALLOWED{}", DUSTY_ROSE_LIGHT, RESET);
            println!("  Owner: {}", owner);
            println!("  File: {}", filename);
            println!("  Content: {}\n", if file.content.is_empty() { "[empty]" } else { &file.content });
        } else {
            println!("\n{}✗ File not found{}\n", DUSTY_ROSE_DARK, RESET);
        }
    }

    fn cmd_file_write_as(&mut self, parts: &[&str]) {
        if parts.len() < 4 {
            println!("Usage: file-write-as <owner> <filename> <content>\n");
            return;
        }
        let owner = parts[1];
        let filename = parts[2];
        let content = parts[3..].join(" ");
        if !self.has_capability(owner, "WRITE") {
            println!("\n{}✗ PERMISSION DENIED{}", DUSTY_ROSE_DARK, RESET);
            println!("  {} lacks WRITE capability\n", owner);
            return;
        }
        if let Some(file) = self.files.get_mut(filename) {
            if content.len() > file.max_size {
                println!("\n{}✗ WRITE FAILED{}", DUSTY_ROSE_DARK, RESET);
                println!("  Content exceeds max size\n");
                return;
            }
            file.content = content.clone();
            println!("\n{}✓ WRITE ALLOWED{}", DUSTY_ROSE_LIGHT, RESET);
            println!("  Owner: {}", owner);
            println!("  File: {}", filename);
            println!("  Bytes written: {}\n", content.len());
        } else {
            println!("\n{}✗ File not found{}\n", DUSTY_ROSE_DARK, RESET);
        }
    }

    fn cmd_file_delete_as(&mut self, parts: &[&str]) {
        if parts.len() < 3 {
            println!("Usage: file-delete-as <owner> <filename>\n");
            return;
        }
        let owner = parts[1];
        let filename = parts[2];
        if !self.has_capability(owner, "DELETE") {
            println!("\n{}✗ PERMISSION DENIED{}", DUSTY_ROSE_DARK, RESET);
            println!("  {} lacks DELETE capability\n", owner);
            return;
        }
        if self.files.remove(filename).is_some() {
            println!("\n{}✓ DELETE ALLOWED{}", DUSTY_ROSE_LIGHT, RESET);
            println!("  Owner: {}", owner);
            println!("  File deleted: {}\n", filename);
        } else {
            println!("\n{}✗ File not found{}\n", DUSTY_ROSE_DARK, RESET);
        }
    }

    fn cmd_integrity_register(&mut self, parts: &[&str]) {
        if parts.len() < 4 {
            println!("Usage: integrity-register <start> <end> <level>\n");
            return;
        }
        let level = parts[3].to_uppercase();
        if !INTEGRITY_LEVELS.iter().any(|l| l.name == level) {
            println!("Error: Invalid level\n");
            return;
        }
        if let (Ok(start), Ok(end)) = (u64::from_str_radix(parts[1], 16), u64::from_str_radix(parts[2], 16)) {
            self.integrity_regions.push(IntegrityRegion {
                start,
                end,
                level,
            });
            println!("\n{}✓ Region registered{}\n", DUSTY_ROSE_LIGHT, RESET);
        }
    }

    fn cmd_integrity_list(&self) {
        println!("\n=== INTEGRITY REGIONS ({} total) ===\n", self.integrity_regions.len());
        if self.integrity_regions.is_empty() {
            println!("No regions registered.\n");
            return;
        }
        for (i, region) in self.integrity_regions.iter().enumerate() {
            println!("  [{}] 0x{:x} - 0x{:x} ({})", i, region.start, region.end, region.level);
        }
        println!();
    }

    fn cmd_status(&self) {
        println!("\n=== SYSTEM STATUS ===\n");
        println!("Capabilities: {}", self.capabilities.len());
        println!("Files: {}", self.files.len());
        println!("Integrity regions: {}\n", self.integrity_regions.len());
    }

    fn cmd_exit(&mut self) {
        println!("\n{}Shutting down...{}\n", DUSTY_ROSE_DARK, RESET);
        self.is_running = false;
    }
}

fn main() {
    let mut kernel = KernelSimulator::new();
    kernel.start();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = BufReader::new(stdin.lock());
    for line in reader.lines() {
        if !kernel.is_running {
            break;
        }
        if let Ok(cmd) = line {
            kernel.process_command(&cmd);
        }
        print!("{}> {}", DUSTY_ROSE_LIGHT, RESET);
        stdout.flush().ok();
    }
}
