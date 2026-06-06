// src/cpu/riscv.rs
use crate::table::{CpuInformation, CpuArchitecture, CpuFeatures, CpuTopology, CpuEnvironment, VirtualAddress};
use core::arch::asm;

pub fn collect_riscv_info() -> CpuInformation {
    CpuInformation {
        architecture: CpuArchitecture::RiscV,
        vendor: *b"RISCV_CORE  ",
        features: CpuFeatures { sse: false, sse2: false, avx: false, avx2: false, aes: false, vmx: false, smep: false, smap: false },
        topology: CpuTopology { cores: 8, threads: 8 },
        environment: CpuEnvironment { virtualized: false },
    }
}

pub unsafe fn riscv_jump(entry: VirtualAddress, boot_info: VirtualAddress) -> ! {
    unsafe{
        asm!(
            "mv a0, {info}", // RISC-V padrão passa o primeiro argumento em A0
            "jr {target}",   // Jump Register
            info = in(reg) boot_info,
            target = in(reg) entry,
            options(noreturn)
        );
    } 
}