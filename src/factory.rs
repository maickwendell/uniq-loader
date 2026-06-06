// src/factory.rs

use crate::table::{
    UniqXHardwareTable, TrustHeader, ArchitecturePageModel, 
    BootInfo, UniversalAddress
};
use crate::cpu::get_cpu_information;
use crate::memory::{calculate_table_crc32, allocate_boot_memory};

const CURRENT_BOOT_INFO_VERSION: u32 = 1;
const CURRENT_TRUST_LAYOUT_VERSION: u32 = 1;

// =========================================================================
// UNIVERSAL AUTOMATION FACTORY (CORRECTED)
// =========================================================================

/// Automatically constructs, populates, and seals the hardware table dynamically.
pub fn generate_universal_boot_contract(detected_numa_nodes: u32) -> Option<(BootInfo, UniqXHardwareTable)> {
    // 1. CORREÇÃO: Coleta as informações ricas da CPU sem descartar
    let gathered_cpu_info = get_cpu_information();

    // 2. Mapeamento abstrato do tamanho da memória
    let selected_page_model = ArchitecturePageModel::Standard4K;
    let total_bytes_discovered: u64 = 16 * 1024 * 1024 * 1024; // 16 GB Simulated
    let universal_memory_capacity = UniversalAddress::from_u64(total_bytes_discovered);

    // 3. Montagem inicial da tabela com o campo do Checksum RIGOROSAMENTE EM ZERO
    let mut hardware_table = UniqXHardwareTable {
        trust_header: TrustHeader {
            signature: *b"UNIQXHW\0",
            version: CURRENT_TRUST_LAYOUT_VERSION,
            table_checksum: 0, // CORREÇÃO: Garantido em zero para o cálculo do CRC
        },
        page_model: selected_page_model,
        cpu_metadata: gathered_cpu_info, // CORREÇÃO: Injetado o metadado coletado da CPU
        total_memory_bytes: universal_memory_capacity,
        lock_status: 0xAA, 
    };

    // 4. CORREÇÃO: ALGORITMO DO CRC32 CORRETO (O ovo ou a galinha resolvido)
    unsafe {
        let table_byte_ptr = &hardware_table as *const UniqXHardwareTable as *const u8;
        let table_byte_size = core::mem::size_of::<UniqXHardwareTable>();
        let hardware_table_slice = core::slice::from_raw_parts(table_byte_ptr, table_byte_size);
        
        // Calcula o CRC com o campo zerado e só depois injeta o resultado final
        let computed_crc = calculate_table_crc32(hardware_table_slice);
        hardware_table.trust_header.table_checksum = computed_crc;
    }

    // 5. CORREÇÃO: Alocação Dinâmica da Tabela (Fim do endereço mágico fixo)
    // Aloca espaço para a tabela dinamicamente na RAM usando o nosso gerenciador de boot
    let table_size = core::mem::size_of::<UniqXHardwareTable>() as u64;
    let allocated_table_address = allocate_boot_memory(table_size, 16)?; // Alinhamento de 16 bytes

    // Estrutura o envelope BootInfo apontando para o endereço dinâmico alocado
    let final_boot_info = BootInfo {
        hardware_table_address: allocated_table_address,
        boot_info_version: CURRENT_BOOT_INFO_VERSION,
        total_numa_nodes: detected_numa_nodes, // CORREÇÃO: Deixa de ser estático
    };

    Some((final_boot_info, hardware_table))
}

// =========================================================================
// CORREÇÃO: FUNÇÃO DE VALIDAÇÃO DO RECEPTOR (KERNEL)
// =========================================================================

/// Verifies if a received hardware table hasn't been corrupted, using your exact logic
pub unsafe fn verify_received_table_integrity(table_ptr: *mut UniqXHardwareTable) -> bool {
    if table_ptr.is_null() { return false; }
    unsafe{
        let mut table = core::ptr::read_volatile(table_ptr);
    
        // PASSO A: Salva o CRC original que veio gravado
        let original_checksum = table.trust_header.table_checksum;
        
        // PASSO B: Zera o campo para poder fazer o recálculo sobre a mesma base
        table.trust_header.table_checksum = 0;
        
        // PASSO C: Recalcula o CRC32
        let table_byte_ptr = &table as *const UniqXHardwareTable as *const u8;
        let table_byte_size = core::mem::size_of::<UniqXHardwareTable>();
        let hardware_table_slice = core::slice::from_raw_parts(table_byte_ptr, table_byte_size);
        let recalculated_checksum = calculate_table_crc32(hardware_table_slice);
        
        // PASSO D: Compara os dois resultados
        original_checksum == recalculated_checksum
    }
    
}