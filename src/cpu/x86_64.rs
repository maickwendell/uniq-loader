// src/cpu/x86_64.rs

use crate::table::{CpuInformation, CpuArchitecture, CpuFeatures, CpuTopology, CpuEnvironment, VirtualAddress};
use core::arch::asm;

pub fn collect_x86_64_info() -> CpuInformation {
    let mut vendor_buffer = [b' '; 12];
    let mut features = CpuFeatures { sse: false, sse2: false, avx: false, avx2: false, aes: false, vmx: false, smep: false, smap: false };
    let mut topology = CpuTopology { cores: 1, threads: 1 };
    let mut environment = CpuEnvironment { virtualized: false };

    unsafe {
        let ebx_out: u32;
        let ecx_out: u32;
        let edx_out: u32;

        // 1. Vendor Extraction using explicit architectural register swapping safe for LLVM
        asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_reg:e}, ebx",
            "pop rbx",
            inout("eax") 0 => _,
            out("ecx") ecx_out,
            out("edx") edx_out,
            ebx_reg = out(reg) ebx_out,
            options(nomem, preserves_flags)
        );
        
        let ebx_b = ebx_out.to_le_bytes();
        let edx_b = edx_out.to_le_bytes();
        let ecx_b = ecx_out.to_le_bytes();
        for i in 0..4 {
            vendor_buffer[i] = ebx_b[i];
            vendor_buffer[i + 4] = edx_b[i];
            vendor_buffer[i + 8] = ecx_b[i];
        }

        // 2. Standard Features (EAX=1)
        let ebx_1: u32;
        let ecx_1: u32;
        let edx_1: u32;
        asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_reg:e}, ebx",
            "pop rbx",
            inout("eax") 1 => _,
            out("ecx") ecx_1,
            out("edx") edx_1,
            ebx_reg = out(reg) ebx_1,
            options(nomem, preserves_flags)
        );
        features.sse = (edx_1 & (1 << 25)) != 0;
        features.sse2 = (edx_1 & (1 << 26)) != 0;
        features.aes = (ecx_1 & (1 << 25)) != 0;
        features.avx = (ecx_1 & (1 << 28)) != 0;
        features.vmx = (ecx_1 & (1 << 5)) != 0;
        environment.virtualized = (ecx_1 & (1 << 31)) != 0;

        topology.threads = (ebx_1 >> 16) & 0xFF;

        // 3. Extended Features (EAX=7, ECX=0)
        let ebx_7: u32;
        asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_reg:e}, ebx",
            "pop rbx",
            inout("eax") 7 => _,
            inout("ecx") 0 => _,
            out("edx") _,
            ebx_reg = out(reg) ebx_7,
            options(nomem, preserves_flags)
        );
        features.avx2 = (ebx_7 & (1 << 5)) != 0;
        features.smep = (ebx_7 & (1 << 7)) != 0;
        features.smap = (ebx_7 & (1 << 20)) != 0;
    }

    CpuInformation {
        architecture: CpuArchitecture::X86_64,
        vendor: vendor_buffer,
        features,
        topology,
        environment,
    }
}

pub unsafe fn x86_64_jump(entry: VirtualAddress, boot_info: VirtualAddress) -> ! {
    unsafe {
        asm!(
            "cli",                  
            "mov rdi, {info}",      
            "jmp {target}",
            info = in(reg) boot_info,
            target = in(reg) entry,
            options(noreturn)
        );
    }
}

pub fn read_tsc() -> u64 {
    let mut eax: u32;
    let mut edx: u32;
    unsafe {
        asm!("rdtsc", lateout("eax") eax, lateout("edx") edx, options(nomem, nostack, preserves_flags));
    }
    ((edx as u64) << 32) | (eax as u64)
}