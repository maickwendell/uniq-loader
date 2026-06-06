// src/memory.rs

use crate::table::{UniqXHardwareTable, MemoryRegion, ArchitecturePageModel, PhysicalAddress, BootInfo};
use core::sync::atomic::{AtomicU32, Ordering};

// =========================================================================
// 1. FAIR MULTICORE SYNCHRONIZATION (TICKET LOCK)
// =========================================================================

/// A starvation-free synchronization primitive.
/// Essential for High-Core-Count (HCC) and massive SMP systems (e.g., 256+ CPUs)
/// to ensure fair, FIFO access to memory management structures.
pub struct TicketLock {
    next_ticket: AtomicU32,
    current_ticket: AtomicU32,
}

impl TicketLock {
    /// Creates a new, unlocked instance of the TicketLock.
    pub const fn new() -> Self {
        Self {
            next_ticket: AtomicU32::new(0),
            current_ticket: AtomicU32::new(0),
        }
    }

    /// Acquires the lock. Spins safely using a hardware hint until the ticket number is called.
    pub fn lock(&self) {
        // Atomic fetch-and-add takes a ticket number, guaranteeing order
        let my_ticket = self.next_ticket.fetch_add(1, Ordering::Relaxed);
        
        // Spin until the current serving counter matches our personal ticket
        while self.current_ticket.load(Ordering::Acquire) != my_ticket {
            core::hint::spin_loop(); // Injects PAUSE instruction to optimize CPU pipeline consumption
        }
    }

    /// Releases the lock, incrementing the counter to serve the next processor in line.
    pub fn unlock(&self) {
        self.current_ticket.fetch_add(1, Ordering::Release);
    }
}

/// Global memory synchronization subsystem instance
static GLOBAL_MEMORY_LOCK: TicketLock = TicketLock::new();

// =========================================================================
// 2. DYNAMIC REGION-BASED BOOT MEMORY ALLOCATOR
// =========================================================================

// Global tracking trackers for the Bootstrap Memory Region Allocator.
// Replaces hardcoded small buffers with actual firmware-provided RAM regions.
static mut ALLOCATOR_BASE_ADDRESS: PhysicalAddress = 0;
static mut ALLOCATOR_CURRENT_OFFSET: u64 = 0;
static mut ALLOCATOR_MAX_CAPACITY: u64 = 0;

/// Initializes the localized boot allocator context within a verified physical memory region.
/// 
/// # Safety
/// This function is unsafe because it directly configures raw hardware memory pointers.
pub unsafe fn initialize_boot_allocator(base: PhysicalAddress, size: u64) {
    GLOBAL_MEMORY_LOCK.lock();
    unsafe{
        ALLOCATOR_BASE_ADDRESS = base;
        ALLOCATOR_MAX_CAPACITY = size;
        ALLOCATOR_CURRENT_OFFSET = 0;
    }
    GLOBAL_MEMORY_LOCK.unlock();
}

/// Allocates a continuous block of physical memory with alignment guarantees.
/// Prevents global memory pool exhaustion and mitigates alignment exceptions.
pub fn allocate_boot_memory(size: u64, alignment: u64) -> Option<PhysicalAddress> {
    // Basic verification to ensure alignment constraints are powers of two
    if alignment == 0 || (alignment & (alignment - 1)) != 0 {
        return None;
    }

    GLOBAL_MEMORY_LOCK.lock();
    unsafe {
        let current_address = ALLOCATOR_BASE_ADDRESS + ALLOCATOR_CURRENT_OFFSET;
        
        // Mathematical alignment adjustment upwards
        let aligned_address = (current_address + alignment - 1) & !(alignment - 1);
        
        // Calculate the relative updated offset relative to the base address
        let usage_offset = aligned_address - ALLOCATOR_BASE_ADDRESS;
        
        // Enforce strict integer overflow prevention check
        let total_needed = match usage_offset.checked_add(size) {
            Some(val) => val,
            None => {
                GLOBAL_MEMORY_LOCK.unlock();
                return None; // Integer overflow detected, rejecting allocation malicious block
            }
        };

        // Out-Of-Memory check against current region thresholds
        if total_needed > ALLOCATOR_MAX_CAPACITY {
            GLOBAL_MEMORY_LOCK.unlock();
            return None;
        }

        // Apply state updates to the allocation pointer trackers
        ALLOCATOR_CURRENT_OFFSET = total_needed;
        GLOBAL_MEMORY_LOCK.unlock();
        
        Some(aligned_address)
    }
}

// =========================================================================
// 3. BOUNDARY SANITIZATION & CRITICAL OVERLAP PROTECTION
// =========================================================================

/// Verifies memory boundaries to ensure no two mapped layouts cross into each other's spaces.
/// Features explicit protection against deliberate or accidental Integer Overflow attacks.
pub fn detects_memory_overlap(region_a: &MemoryRegion, region_b: &MemoryRegion) -> bool {
    // Safely parse ends via checked additions. If an overflow occurs, flag the layout as corrupt/overlapping.
    let a_end = match region_a.base.checked_add(region_a.size) {
        Some(val) => val,
        None => return true, 
    };

    let b_end = match region_b.base.checked_add(region_b.size) {
        Some(val) => val,
        None => return true,
    };

    // Standard spatial collision intersection matrix rule
    region_a.base < b_end && region_b.base < a_end
}

/// Safely zeroes out memory blocks to wipe sensitive leftovers or clear uninitialized segments (.bss)
pub fn secure_zero_memory(destination: PhysicalAddress, byte_count: u64) {
    let mut pointer = destination as *mut u8;
    for _ in 0..byte_count {
        unsafe {
            // Write volatile prevents compiler optimizations from removing wiping code execution loops
            core::ptr::write_volatile(pointer, 0x00);
            pointer = pointer.add(1);
        }
    }
}

// =========================================================================
// 4. AGNOSTIC PAGE LAYOUT ANALYSIS
// =========================================================================

/// Translates multi-architecture page layout configurations into real hardware byte increments.
/// Fully compatible with x86_64, ARM64, RISC-V, and PowerPC paging granularities.
pub fn get_model_page_size_bytes(model: ArchitecturePageModel) -> u64 {
    match model {
        ArchitecturePageModel::Standard4K => 4 * 1024,
        ArchitecturePageModel::Medium16K  => 16 * 1024,
        ArchitecturePageModel::Large64K   => 64 * 1024,
        ArchitecturePageModel::Huge2M     => 2 * 1024 * 1024,
        ArchitecturePageModel::Massive1G  => 1 * 1024 * 1024 * 1024,
    }
}

// =========================================================================
// 5. BOOTINFO METADATA INTERFACING (HANDOFF IMPLEMENTATION)
// =========================================================================

/// Writes the UniqX Hardware Table straight into its finalized physical position passed inside BootInfo.
pub fn write_hardware_table(boot_info: &BootInfo, table: UniqXHardwareTable) {
    let target_pointer = boot_info.hardware_table_address as *mut UniqXHardwareTable;
    unsafe {
        core::ptr::write_volatile(target_pointer, table);
    }
}

/// Reads the live UniqX Hardware Table directly using the pointer located inside the BootInfo wrapper.
pub fn read_hardware_table(boot_info: &BootInfo) -> UniqXHardwareTable {
    let target_pointer = boot_info.hardware_table_address as *const UniqXHardwareTable;
    unsafe {
        core::ptr::read_volatile(target_pointer)
    }
}

/// Locks the table inside memory by flagging its activation byte status to 0x55.
pub fn lock_hardware_table(boot_info: &BootInfo) {
    let target_pointer = boot_info.hardware_table_address as *mut UniqXHardwareTable;
    unsafe {
        let mut table = core::ptr::read_volatile(target_pointer);
        table.lock_status = 0x55; // Core Root of Trust locked signature
        core::ptr::write_volatile(target_pointer, table);
    }
}

/// Computes table verification checks via hardware-friendly bitwise CRC32 IEEE 802.3
pub fn calculate_table_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}