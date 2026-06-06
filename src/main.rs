// src/main.rs
#![no_std]
#![no_main]
#![allow(dead_code)] // Silenciar avisos de codigos não utilizados

// Declare the cleaned, highly-optimized modular sub-systems of UniqLoader
mod table;
mod memory;
mod cpu; // Automatically resolves to the src/cpu/mod.rs file structure
mod factory; // Explicitly bring the automated factory module into scope

use core::panic::PanicInfo;
use crate::memory::{write_hardware_table, lock_hardware_table, initialize_boot_allocator};
use crate::cpu::{architecture_jump, native_relax_core};
use crate::factory::generate_universal_boot_contract;
use crate::table::VirtualAddress;


/// Absolute execution entry point for the bare-metal UniqLoader (Stage 3).
/// Configured under no_mangle to prevent the compiler from altering the symbol link name.
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // 1. EARLY ALLOCATION SYSTEM BOOTSTRAPPING
    // Before creating the table dynamically, we must establish a baseline memory boundary.
    // For this bootstrap phase, we assume a continuous safe firmware-given area (e.g., 0x00100000).
    unsafe {
        let early_allocator_base = 0x00100000;
        let early_allocator_size = 4 * 1024 * 1024; // 4 Megabytes of operational room
        initialize_boot_allocator(early_allocator_base, early_allocator_size);
    }

    // 2. AUTOMATION: Harvest telemetry and generate the hardware table contract
    // We pass '1' as the discovered NUMA node ID count fallback for standard initialization.
    let (boot_info, hardware_table) = match generate_universal_boot_contract(1) {
        Some(contract) => contract,
        None => {
            // If allocation or overflow protections trigger, gracefully loop core down
            native_relax_core();
            // Bloco terminal inline garante o tipo '!' para o braço None
            loop{}
        }, 
    }; 

    // 3. ROOT OF TRUST STAGING & RECORDING
    // Write the compiled hardware matrix straight into its dynamic allocated physical position
    write_hardware_table(&boot_info, hardware_table);
    
    // Seal the parameters inside RAM, locking internal status values to 0x55 (Locked)
    lock_hardware_table(&boot_info);

    // 4. ARCHITECTURAL HANDOFF JUMP (THE KERNEL GATEWAY)
    // Map the expected destination execution pointer of the incoming operating system kernel.
    let kernel_entry_point: VirtualAddress = 0x00200000;
    
    // Convert the local BootInfo structure reference into a clean agnostic VirtualAddress pointer
    let boot_info_address: VirtualAddress = &boot_info as *const _ as usize;

    unsafe {
        // Execute the final multi-architecture register handoff leap into the kernel
        architecture_jump(kernel_entry_point, boot_info_address);
    }
}

/// Emergency fallback routine invoked whenever a critical bare-metal fault triggers.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Gracefully relax the current physical core, saving power and avoiding pipeline exceptions
    native_relax_core();
    loop{}
}