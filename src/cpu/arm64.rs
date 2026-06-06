// src/cpu/arm64.rs
use crate::table::{CpuInformation, CpuArchitecture, CpuFeatures, CpuTopology, CpuEnvironment, VirtualAddress};
use core::arch::asm;

pub fn collect_arm64_info() -> CpuInformation {
    // ARM64 lê registradores de sistema (MIDR_EL1) via firmware em implementações reais
    CpuInformation {
        architecture: CpuArchitecture::Arm64,
        vendor: *b"ARM_CORTEX  ",
        features: CpuFeatures { sse: false, sse2: false, avx: false, avx2: false, aes: true, vmx: false, smep: false, smap: false },
        topology: CpuTopology { cores: 4, threads: 4 },
        environment: CpuEnvironment { virtualized: false },
    }
}

pub unsafe fn arm64_jump(entry: VirtualAddress, boot_info: VirtualAddress) -> ! {
    unsafe{
        asm!(
            "mov x0, {info}", // ARM64 padrão passa o primeiro argumento em X0
            "br {target}",    // Branch Register salto absoluto
            info = in(reg) boot_info,
            target = in(reg) entry,
            options(noreturn)
        );
    } 
}