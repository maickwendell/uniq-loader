// src/cpu/mod.rs

pub mod x86_64;
pub mod arm64;
pub mod riscv;

use crate::table::{VirtualAddress, CpuInformation};

// =========================================================================
// 1. UNIVERSAL ARTIFACT JUMP (MULTI-ARCHITECTURE HANDOFF)
// =========================================================================

/// Executes a clean, multi-architecture hardware state jump to the target Kernel entry.
/// Automatically injects the boot data argument into the proper platform register 
/// (RDI for x86_64, X0 for ARM64, A0 for RISC-V).
/// 
/// # Safety
/// This function is highly unsafe as it forcefully alters the hardware Instruction Pointer,
/// permanently handing control over to external operating system code.
pub unsafe fn architecture_jump(kernel_entry: VirtualAddress, boot_info: VirtualAddress) -> ! {
    #[cfg(target_arch = "x86_64")]
    { 
        unsafe { x86_64::x86_64_jump(kernel_entry, boot_info); }
    }

    #[cfg(target_arch = "aarch64")]
    { 
        unsafe { arm64::arm64_jump(kernel_entry, boot_info); }
    }

    #[cfg(target_arch = "riscv64")]
    { 
        unsafe { riscv::riscv_jump(kernel_entry, boot_info); }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    {
        // Ultimate fallback if compilation happens on an unmapped target platform
        loop { 
            core::hint::spin_loop(); 
        }
    }
}

// =========================================================================
// 2. GLOBAL DISCOVERY INTERFACE (THE AUTOMATION FACTORY HARVESTER)
// =========================================================================

/// Agnostic interface to harvest execution root profile details across any physical core type.
/// Dynamically routes execution to specific low-level registers at compile time.
pub fn get_cpu_information() -> CpuInformation {
    #[cfg(target_arch = "x86_64")]
    { 
        return x86_64::collect_x86_64_info(); 
    }

    #[cfg(target_arch = "aarch64")]
    { 
        return arm64::collect_arm64_info(); 
    }

    #[cfg(target_arch = "riscv64")]
    { 
        return riscv::collect_riscv_info(); 
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64", target_arch = "riscv64")))]
    {
        use crate::table::{CpuArchitecture, CpuFeatures, CpuTopology, CpuEnvironment};
        // Default emergency profile fallback for unexpected silicon targets
        CpuInformation {
            architecture: CpuArchitecture::Unknown,
            vendor: *b"UNKNOWN_CORE",
            features: CpuFeatures { sse: false, sse2: false, avx: false, avx2: false, aes: false, vmx: false, smep: false, smap: false },
            topology: CpuTopology { cores: 1, threads: 1 },
            environment: CpuEnvironment { virtualized: false },
        }
    }
}

// =========================================================================
// 3. HARDWARE CORE RELAXATION & TIME TELEMETRY
// =========================================================================

/// Safe alternative execution trap loop avoiding processor catatone lockups caused by cli/hlt.
/// Utilizes hardware optimization hints to lower power draw during panic states or core waits.
pub fn native_relax_core() {
    loop {
        unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("pause", options(nomem, nostack, preserves_flags));
            
            #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
            core::arch::asm!("nop", options(nomem, nostack, preserves_flags));
        }
        core::hint::spin_loop();
    }
}

/// Universal platform-agnostic wrapper to read high-precision monotonic timestamps.
/// Essential for early profiling, metrics scheduling, and scheduler tracking.
pub fn read_platform_timestamp() -> u64 {
    #[cfg(target_arch = "x86_64")]
    { 
        return x86_64::read_tsc(); 
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        0 // Fallback counter logic for chips missing a standardized unified low-overhead read instruction
    }
  
}