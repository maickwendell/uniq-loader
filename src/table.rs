// src/table.rs

// =========================================================================
// 1. TYPE ALIASES & GLOBAL ENUMS
// =========================================================================

/// Universal and flexible type alias for physical memory addressing (up to 64-bit native).
pub type PhysicalAddress = u64;

/// Universal type alias for virtual memory representation based on target architecture width.
pub type VirtualAddress = usize;

/// Standard architectural families supported by the UniqX specification.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CpuArchitecture {
    Unknown = 0,
    X86_64 = 1,
    Arm64 = 2,
    RiscV = 3,
}

/// Dynamic multi-architecture page layout configurations (Agnostic Paging).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum ArchitecturePageModel {
    Standard4K,
    Medium16K,
    Large64K,
    Huge2M,
    Massive1G,
}

/// Comprehensive typing system for memory mapping and memory hot-plug readiness.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MemoryType {
    Usable = 0,
    Reserved = 1,
    Acpi = 2,
    Mmio = 3,
    Framebuffer = 4,
    Kernel = 5,
    Bootloader = 6,
    Guard = 7,
}

// =========================================================================
// 2. CPU METADATA STRUCTURES
// =========================================================================

/// Hardware feature flags gathered directly from specialized registers (e.g., CPUID).
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct CpuFeatures {
    pub sse: bool,
    pub sse2: bool,
    pub avx: bool,
    pub avx2: bool,
    pub aes: bool,
    pub vmx: bool,
    pub smep: bool,
    pub smap: bool,
}

/// Symmetrical Multiprocessing (SMP) core layout mapping.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct CpuTopology {
    pub cores: u32,
    pub threads: u32,
}

/// Virtualization environment discovery parameters.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct CpuEnvironment {
    pub virtualized: bool, // Set to true if running under KVM, QEMU, VMware, Hyper-V, etc.
}

/// Root hardware profile containing full execution details.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct CpuInformation {
    pub architecture: CpuArchitecture,
    pub vendor: [u8; 12],
    pub features: CpuFeatures,
    pub topology: CpuTopology,
    pub environment: CpuEnvironment,
}

// =========================================================================
// 3. MEMORY MANAGEMENT & SECURITY ATTRIBUTES
// =========================================================================

/// Hardware page permissions mapped straight to page tables (e.g., NX bit enforcement).
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct MemoryPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

/// NUMA and Bank-aware Memory Region Descriptor.
#[derive(Copy, Clone, Debug)]
#[repr(C)] // Uses natural alignment for clean indexing inside continuous arrays
pub struct MemoryRegion {
    pub base: PhysicalAddress,
    pub size: u64,
    pub region_type: MemoryType,
    pub permissions: MemoryPermissions,
    pub numa_node_id: u32, // Maps physical CPU socket or memory bank ownership
}

/// Elastic address wrapper engineered to scale past 64-bit registers in the future.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct UniversalAddress {
    pub raw_bytes: [u8; 32], // 256-bit ready address space container
}

// =========================================================================
// 4. THE UNIVERSALADDRESS OPERATION ENGINE
// =========================================================================

impl UniversalAddress {
    /// Constructs a UniversalAddress from a standard native 64-bit pointer.
    pub fn from_u64(address: u64) -> Self {
        let mut raw = [0u8; 32];
        let bytes = address.to_le_bytes();
        raw[0..8].copy_from_slice(&bytes);
        Self { raw_bytes: raw }
    }

    /// Safely computes address mathematical additions while protecting against Integer Overflows.
    pub fn add(&self, offset: u64) -> Option<Self> {
        let mut current = [0u8; 8];
        current.copy_from_slice(&self.raw_bytes[0..8]);
        let current_u64 = u64::from_le_bytes(current);
        
        let new_u64 = current_u64.checked_add(offset)?;
        Some(Self::from_u64(new_u64))
    }

    /// Safely computes address mathematical subtractions protecting underflow boundaries.
    pub fn subtract(&self, offset: u64) -> Option<Self> {
        let mut current = [0u8; 8];
        current.copy_from_slice(&self.raw_bytes[0..8]);
        let current_u64 = u64::from_le_bytes(current);
        
        let new_u64 = current_u64.checked_sub(offset)?;
        Some(Self::from_u64(new_u64))
    }

    /// Performs exact linear lexicographical comparison between two universal memory keys.
    pub fn compare(&self, other: &Self) -> core::cmp::Ordering {
        self.raw_bytes.cmp(&other.raw_bytes)
    }
}

// =========================================================================
// 5. MASTER BOOT HEADERS (HANDOFF CONTRACT)
// =========================================================================

/// Integrity and Criptographic Trust Validation layer.
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct TrustHeader {
    pub signature: [u8; 8],         // Layout magic signature string (e.g., b"UNIQXHW\0")
    pub version: u32,              // Layout/Protocol standard format iteration version
    pub table_checksum: u32,       // IEEE 802.3 CRC32 verification hash field
}

/// The absolute master structure containing complete environmental telemetry for the Kernel.
#[repr(C, packed)]
pub struct UniqXHardwareTable {
    pub trust_header: TrustHeader,
    pub page_model: ArchitecturePageModel,
    pub cpu_metadata: CpuInformation,
    pub total_memory_bytes: UniversalAddress,
    pub lock_status: u8,            // 0xAA = Open/Staging, 0x55 = Rigid Root of Trust Locked
}

/// Dynamic, metadata envelope container passed into the execution registers during handoff.
#[repr(C, packed)]
pub struct BootInfo {
    pub hardware_table_address: PhysicalAddress, // Points to the dynamic allocated UniqXHardwareTable
    pub boot_info_version: u32,
    pub total_numa_nodes: u32,
}