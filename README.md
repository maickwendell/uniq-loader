# UniqX Bootloader (`uniq-loader`) 🛡️

The Universal Modular Bootloader and Architecture Abstraction Layer for the **UniqX Ecosystem**.

`uniq-loader` is a bare-metal bootloader written in pure Rust (`no_std`), designed to bridge the gap between modern hardware architectures and the UniqX microkernel. It standardizes hardware discovery, creates an immutable Root of Trust (RoT), and builds a unified interface for the system execution environment.

## 🚀 Features

- **Zero-Dependency Core:** Built using `#![no_std]` and `#![no_main]` to run directly on bare metal without an underlying operating system.
- **Hardware Abstraction Table:** Standardizes CPU topology, memory mappings, and capacities into a unified `UniqXHardwareTable` before kernel initialization.
- **Root of Trust (RoT):** Implements post-write integrity verification and memory locking simulation (`lock_status = 0xAA`) to prevent runtime tampering with hardware specifications.
- **Direct Assembly Interfacing:** Uses optimized inline assembly (`asm!`) to safely probe the physical silicon, implementing strict register preservation protocols to bypass LLVM constraints (such as `rbx` isolation during `cpuid` execution).
- **Multi-Architecture Ready:** Modular abstractions fully prepared and structured for `x86_64`, `ARM64` (AArch64), and `RISC-V` (RV64).
- **Optimized for Embedded Silicon:** Compiled with tight constraints, making it ideal for size-optimized deployments (`opt-level = "z"` / LTO).

## 📁 Project Structure

```text
src/
├── main.rs           # Bootloader entry point (_start), panic handling, and handoff
├── table.rs          # Definitions for contracts, memory permissions, and addresses
├── memory.rs         # Primitive boot allocation, CRC32 calculation, and overlap checks
├── factory.rs        # Generation and validation of the universal boot contract
└── cpu/              # CPU architecture subsystem
    ├── mod.rs        # Architecture-agnostic interface and core relaxation routines
    ├── x86_64.rs     # Safe CPUID discovery, TSC reading, and x86_64 context switching
    ├── arm64.rs      # Native ARM64 (AArch64) metadata and absolute jumps
    └── riscv.rs      # Native RISC-V (RV64) metadata and absolute jumps


## 🏗️ Architecture Flow

Initialization (_start): The physical processor wakes up, initializes a primitive boot allocator, and sets up early staging boundaries.

Discovery & Creation: Measures hardware specs, extracts vendor strings, and builds the contract through generate_universal_boot_contract.

Integrity Validation: Calculates and validates the structure's layout against a 32-bit CRC checksum, checking the magic signature (UNIQXHW\0).

Hardware Lock: Simulates MMU protection by sealing the state to a locked status, preventing post-write runtime overrides.

Kernel Handoff: Executes an absolute architecture-specific jump to the kernel entry point, passing the verified environment contract. In case of failure, it safely diverges into an optimized low-power core relaxation loop (pause / nop).

## 🛠️ Building from Source

Ensure you have the appropriate bare-metal target installed for your toolchain (e.g., x86_64-unknown-none):

# Add the target if needed
rustup target add x86_64-unknown-none

# Clean artifacts and compile a fresh optimized release binary
cargo clean && cargo build --release

Developed as the foundational bedrock for the UniqX Operating System ecosystem.
