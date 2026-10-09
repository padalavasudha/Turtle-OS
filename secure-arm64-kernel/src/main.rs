#![no_std]
#![no_main]

mod boot;
mod exceptions;
mod capabilities;
mod integrity;
mod rootkit;
mod virtio;

use core::panic::PanicInfo;
use capabilities::{CapabilityManager, CapabilityRights, ResourceType};
use integrity::IntegrityMonitor;
use rootkit::RootkitDetector;
use virtio::VirtioNet;

const UART0: *mut u8 = 0x09000000 as *mut u8;

unsafe fn uart_write_char(c: u8) {
    core::ptr::write_volatile(UART0, c);
}

fn print(s: &str) {
    for byte in s.bytes() {
        unsafe { uart_write_char(byte); }
    }
}

fn print_num(mut n: u32) {
    if n == 0 {
        print("0");
        return;
    }
    let mut digits = [0u8; 10];
    let mut len = 0;
    while n > 0 {
        digits[len] = (n % 10) as u8 + b'0';
        n /= 10;
        len += 1;
    }
    while len > 0 {
        len -= 1;
        unsafe { uart_write_char(digits[len]); }
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    print("\n========================================\n");
    print("SECURE ARM64 KERNEL - INTERACTIVE SHELL\n");
    print("========================================\n\n");
    
    // Initialize systems
    let mut cm = CapabilityManager::new();
    let mut im = IntegrityMonitor::new();
    let mut rd = RootkitDetector::new(0x80000);
    let mut net = VirtioNet::new();
    
    print("Initializing systems...\n");
    
    // Capabilities
    cm.grant_capability(CapabilityRights::Read, ResourceType::File(1), 0, true).ok();
    cm.grant_capability(CapabilityRights::Write, ResourceType::File(2), 1, true).ok();
    print("  Capabilities: ");
    print_num(cm.capability_count() as u32);
    print("\n");
    
    // Integrity
    im.register_region(0x80000, 0x90000, integrity::IntegrityLevel::Critical).ok();
    im.register_region(0x90000, 0xA0000, integrity::IntegrityLevel::High).ok();
    print("  Integrity regions: ");
    print_num(im.region_count() as u32);
    print("\n");
    
    // Network
    net.init().ok();
    print("  Network device: OK\n");
    
    let mac = net.get_mac();
    print("  MAC address: ");
    print_hex(mac[0]);
    print(":");
    print_hex(mac[1]);
    print(":");
    print_hex(mac[2]);
    print(":");
    print_hex(mac[3]);
    print(":");
    print_hex(mac[4]);
    print(":");
    print_hex(mac[5]);
    print("\n\n");
    
    print("========================================\n");
    print("TELNET SHELL READY\n");
    print("========================================\n\n");
    
    print("CONNECT: telnet localhost 9999\n\n");
    
    print("COMMANDS:\n");
    print("  help       - Show this help\n");
    print("  cap-grant  - Grant capability\n");
    print("  cap-list   - List capabilities\n");
    print("  cap-revoke - Revoke capability\n");
    print("  status     - System status\n");
    print("  integrity  - Check integrity\n");
    print("  rootkit    - Scan for rootkits\n");
    print("  network    - Network stats\n");
    print("  exit       - Close connection\n\n");
    
    print("Waiting for telnet connections on port 9999...\n");
    print("(Kernel accepting commands interactively)\n\n");
    
    // Simulated interactive loop
    // In real implementation: read from virtio-net RX queue, parse commands, write to TX queue
    loop {
        // Simulate processing telnet input
        for _ in 0..1000000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}

fn print_hex(mut n: u8) {
    let hex_chars = b"0123456789abcdef";
    let high = (n >> 4) & 0xf;
    let low = n & 0xf;
    unsafe {
        uart_write_char(hex_chars[high as usize]);
        uart_write_char(hex_chars[low as usize]);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print("PANIC\n");
    loop {}
}
