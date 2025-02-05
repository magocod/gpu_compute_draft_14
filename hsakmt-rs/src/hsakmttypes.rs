#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

//
// HSA STATUS codes returned by the KFD Interfaces
//

use std::fmt::{Debug, Formatter};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HsakmtStatus {
    HSAKMT_STATUS_SUCCESS = 0,         // Operation successful
    HSAKMT_STATUS_ERROR = 1,           // General error return if not otherwise specified
    HSAKMT_STATUS_DRIVER_MISMATCH = 2, // User mode component is not compatible with kernel HSA driver

    HSAKMT_STATUS_INVALID_PARAMETER = 3, // KFD identifies input parameters invalid
    HSAKMT_STATUS_INVALID_HANDLE = 4,    // KFD identifies handle parameter invalid
    HSAKMT_STATUS_INVALID_NODE_UNIT = 5, // KFD identifies node or unit parameter invalid

    HSAKMT_STATUS_NO_MEMORY = 6, // No memory available (when allocating queues or memory)
    HSAKMT_STATUS_BUFFER_TOO_SMALL = 7, // A buffer needed to handle a request is too small

    HSAKMT_STATUS_NOT_IMPLEMENTED = 10, // KFD function is not implemented for this set of paramters
    HSAKMT_STATUS_NOT_SUPPORTED = 11,   // KFD function is not supported on this node
    HSAKMT_STATUS_UNAVAILABLE = 12,     // KFD function is not available currently on this node (but
    // may be at a later time)
    HSAKMT_STATUS_OUT_OF_RESOURCES = 13, // KFD function request exceeds the resources currently available.

    HSAKMT_STATUS_KERNEL_IO_CHANNEL_NOT_OPENED = 20, // KFD driver path not opened
    HSAKMT_STATUS_KERNEL_COMMUNICATION_ERROR = 21,   // user-kernel mode communication failure
    HSAKMT_STATUS_KERNEL_ALREADY_OPENED = 22,        // KFD driver path already opened
    HSAKMT_STATUS_HSAMMU_UNAVAILABLE = 23, // ATS/PRI 1.1 (Address Translation Services) not available
    // (IOMMU driver not installed or not-available)
    HSAKMT_STATUS_WAIT_FAILURE = 30, // The wait operation failed
    HSAKMT_STATUS_WAIT_TIMEOUT = 31, // The wait operation timed out

    HSAKMT_STATUS_MEMORY_ALREADY_REGISTERED = 35, // Memory buffer already registered
    HSAKMT_STATUS_MEMORY_NOT_REGISTERED = 36,     // Memory buffer not registered
    HSAKMT_STATUS_MEMORY_ALIGNMENT = 37,          // Memory parameter not aligned
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaVersionInfo {
    pub KernelInterfaceMajorVersion: u32, // supported kernel interface major version
    pub KernelInterfaceMinorVersion: u32, // supported kernel interface minor version
}

#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub struct HsaSystemProperties {
    pub NumNodes: u32, // the number of "H-NUMA" memory nodes.
    // each node represents a discoverable node of the system
    // All other enumeration is done on a per-node basis
    pub PlatformOem: u32, // identifies HSA platform, reflects the OEMID in the CRAT
    pub PlatformId: u32,  // HSA platform ID, reflects OEM TableID in the CRAT
    pub PlatformRev: u32, // HSA platform revision, reflects Platform Table Revision ID
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaCapabilityStruct {
    HotPluggable: u32, // the node may be removed by some system action
    // (event will be sent)
    HSAMMUPresent: u32, // This node has an ATS/PRI 1.1 compatible
    // translation agent in the system (e.g. IOMMUv2)
    SharedWithGraphics: u32, // this HSA nodes' GPU function is also used for OS primary
    // graphics render (= UI)
    QueueSizePowerOfTwo: u32, // This node GPU requires the queue size to be a power of 2 value
    QueueSize32bit: u32,      // This node GPU requires the queue size to be less than 4GB
    QueueIdleEvent: u32,      // This node GPU supports notification on Queue Idle
    VALimit: u32,             // This node GPU has limited VA range for platform
    // (typical 40bit). Affects shared VM use for 64bit apps
    WatchPointsSupported: u32, // Indicates if Watchpoints are available on the node.
    WatchPointsTotalBits: u32, // Watchpoints available. To determine the number use 2^value

    DoorbellType: u32, // 0: This node has pre-1.0 doorbell characteristic
    // 1: This node has 1.0 doorbell characteristic
    // 2,3: reserved for future use
    AQLQueueDoubleMap: u32,                // The unit needs a VA “double map”
    DebugTrapSupported: u32,               // Indicates if Debug Trap is supported on the node.
    WaveLaunchTrapOverrideSupported: u32, // Indicates if Wave Launch Trap Override is supported on the node.
    WaveLaunchModeSupported: u32,         // Indicates if Wave Launch Mode is supported on the node.
    PreciseMemoryOperationsSupported: u32, // Indicates if Precise Memory Operations are supported on the node.
    DEPRECATED_SRAM_EDCSupport: u32,       // Old buggy user mode depends on this being 0
    Mem_EDCSupport: u32, // Indicates if GFX internal DRAM/HBM EDC/ECC functionality is active
    RASEventNotify: u32, // Indicates if GFX extended RASFeatures and RAS EventNotify status is available
    ASICRevision: u32,   // Indicates the ASIC revision of the chip on this node.
    SRAM_EDCSupport: u32, // Indicates if GFX internal SRAM EDC/ECC functionality is active
    pub(crate) SVMAPISupported: u32, // Whether or not the SVM API is supported
    CoherentHostAccess: u32, // Whether or not device memory can be coherently accessed by the host CPU
    DebugSupportedFirmware: u32, // Indicates if HWS firmware supports GPU debugging
    PreciseALUOperationsSupported: u32, //Indicates if precise ALU operations are supported for GPU debugging
    PerQueueResetSupported: u32,        // Indicates per-queue reset supported
}

impl Default for HsaCapabilityStruct {
    fn default() -> Self {
        Self {
            HotPluggable: 1, // the node may be removed by some system action
            // (event will be sent)
            HSAMMUPresent: 1, // This node has an ATS/PRI 1.1 compatible
            // translation agent in the system (e.g. IOMMUv2)
            SharedWithGraphics: 1, // this HSA nodes' GPU function is also used for OS primary
            // graphics render (= UI)
            QueueSizePowerOfTwo: 1, // This node GPU requires the queue size to be a power of 2 value
            QueueSize32bit: 1,      // This node GPU requires the queue size to be less than 4GB
            QueueIdleEvent: 1,      // This node GPU supports notification on Queue Idle
            VALimit: 1,             // This node GPU has limited VA range for platform
            // (typical 40bit). Affects shared VM use for 64bit apps
            WatchPointsSupported: 1, // Indicates if Watchpoints are available on the node.
            WatchPointsTotalBits: 4, // Watchpoints available. To determine the number use 2^value

            DoorbellType: 2, // 0: This node has pre-1.0 doorbell characteristic
            // 1: This node has 1.0 doorbell characteristic
            // 2,3: reserved for future use
            AQLQueueDoubleMap: 1,  // The unit needs a VA “double map”
            DebugTrapSupported: 1, // Indicates if Debug Trap is supported on the node.
            WaveLaunchTrapOverrideSupported: 1, // Indicates if Wave Launch Trap Override is supported on the node.
            WaveLaunchModeSupported: 1, // Indicates if Wave Launch Mode is supported on the node.
            PreciseMemoryOperationsSupported: 1, // Indicates if Precise Memory Operations are supported on the node.
            DEPRECATED_SRAM_EDCSupport: 1,       // Old buggy user mode depends on this being 0
            Mem_EDCSupport: 1, // Indicates if GFX internal DRAM/HBM EDC/ECC functionality is active
            RASEventNotify: 1, // Indicates if GFX extended RASFeatures and RAS EventNotify status is available
            ASICRevision: 4,   // Indicates the ASIC revision of the chip on this node.
            SRAM_EDCSupport: 1, // Indicates if GFX internal SRAM EDC/ECC functionality is active
            SVMAPISupported: 1, // Whether or not the SVM API is supported
            CoherentHostAccess: 1, // Whether or not device memory can be coherently accessed by the host CPU
            DebugSupportedFirmware: 1, // Indicates if HWS firmware supports GPU debugging
            PreciseALUOperationsSupported: 1, //Indicates if precise ALU operations are supported for GPU debugging
            PerQueueResetSupported: 1,        // Indicates per-queue reset supported
        }
    }
}

#[repr(C)]
pub union HSA_CAPABILITY {
    pub(crate) Value: u32,
    pub(crate) ui32: HsaCapabilityStruct,
}

impl Debug for HSA_CAPABILITY {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_CAPABILITY {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaEngineId {
    uCode: u32,               // ucode packet processor version
    pub(crate) Major: u32,    // GFXIP Major engine version
    pub(crate) Minor: u32,    // GFXIP Minor engine version
    pub(crate) Stepping: u32, // GFXIP Stepping info
}

impl Default for HsaEngineId {
    fn default() -> Self {
        Self {
            uCode: 10,   // ucode packet processor version
            Major: 6,    // GFXIP Major engine version
            Minor: 8,    // GFXIP Minor engine version
            Stepping: 8, // GFXIP Stepping info
        }
    }
}

#[repr(C)]
pub union HSA_ENGINE_ID {
    pub(crate) Value: u32,
    pub(crate) ui32: HsaEngineId,
}

impl Debug for HSA_ENGINE_ID {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_ENGINE_ID {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaEngineVersion {
    uCodeSDMA: u32, // ucode version SDMA engine
    uCodeRes: u32,  // ucode version (reserved)
    Reserved: u32,  // Reserved, must be 0
}

impl Default for HsaEngineVersion {
    fn default() -> Self {
        Self {
            uCodeSDMA: 10, // ucode version SDMA engine
            uCodeRes: 10,  // ucode version (reserved)
            Reserved: 12,  // Reserved, must be 0
        }
    }
}

#[repr(C)]
pub union HSA_ENGINE_VERSION {
    pub(crate) Value: u32,
    st: HsaEngineVersion,
}

impl Debug for HSA_ENGINE_VERSION {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_ENGINE_VERSION {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaDebugProperties {
    WatchAddrMaskLoBit: u64, // Only bits
    // WatchAddrMaskLoBit..WatchAddrMaskHiBit
    // of the
    WatchAddrMaskHiBit: u64, // watch address mask are used.
    // 0 is the least significant bit.
    DispatchInfoAlwaysValid: u64, // 0 if control of TTMP setup is
    // controlled on a per process
    // basis and is not always enabled
    // 1 if TTMP setup is always
    // enabled
    AddressWatchpointShareKind: u64, // whether the address watchpoint
    //     is per process or shared with
    //     all proccesses
    // 0 if shared or unsuppoted
    //    (unsupported indicated by
    //    address_watchpoint_count == 0)
    //    All current devices have shared watchpoints
    // 1 if unshared
    Reserved: u64, //
}

impl Default for HsaDebugProperties {
    fn default() -> Self {
        Self {
            WatchAddrMaskLoBit: 4, // Only bits
            // WatchAddrMaskLoBit..WatchAddrMaskHiBit
            // of the
            WatchAddrMaskHiBit: 6, // watch address mask are used.
            // 0 is the least significant bit.
            DispatchInfoAlwaysValid: 1, // 0 if control of TTMP setup is
            // controlled on a per process
            // basis and is not always enabled
            // 1 if TTMP setup is always
            // enabled
            AddressWatchpointShareKind: 1, // whether the address watchpoint
            //     is per process or shared with
            //     all proccesses
            // 0 if shared or unsuppoted
            //    (unsupported indicated by
            //    address_watchpoint_count == 0)
            //    All current devices have shared watchpoints
            // 1 if unshared
            Reserved: 52, //
        }
    }
}

// Debug Properties and values
// HSA runtime may expose a subset of the capabilities outlined to the application
#[repr(C)]
pub union HSA_DEBUG_PROPERTIES {
    pub(crate) Value: u64,
    st: HsaDebugProperties,
}

impl Debug for HSA_DEBUG_PROPERTIES {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_DEBUG_PROPERTIES {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

//
// HSA node properties. This structure is an output parameter of hsaKmtGetNodeProperties()
// The application or runtime can use the information herein to size the topology management structures
// Unless there is some very weird setup, there is at most one "GPU" device (with a certain number
// of throughput compute units (= SIMDs) associated with a H-NUMA node.
//

pub const HSA_PUBLIC_NAME_SIZE: usize = 64; // Marketing name string size

#[derive(Debug, PartialEq)]
pub struct HsaNodeProperties {
    pub(crate) NumCPUCores: u32, // # of latency (= CPU) cores present on this HSA node.
    // This value is 0 for a HSA node with no such cores,
    // e.g a "discrete HSA GPU"
    pub(crate) NumFComputeCores: u32, // # of HSA throughtput (= GPU) FCompute cores ("SIMD") present in a node.
    // This value is 0 if no FCompute cores are present (e.g. pure "CPU node").
    NumNeuralCores: u32, // # of HSA neural processing units (= AIE) present in a
    // node. This value is 0 if there are no NeuralCores.
    pub(crate) NumMemoryBanks: u32, // # of discoverable memory bank affinity properties on this "H-NUMA" node.
    pub(crate) NumCaches: u32, // # of discoverable cache affinity properties on this "H-NUMA"  node.

    pub(crate) NumIOLinks: u32, // # of discoverable IO link affinity properties of this node
    // connecting to other nodes.
    pub(crate) CComputeIdLo: u32, // low value of the logical processor ID of the latency (= CPU)
    // cores available on this node
    pub(crate) FComputeIdLo: u32, // low value of the logical processor ID of the throughput (= GPU)
    // units available on this node
    pub(crate) Capability: HSA_CAPABILITY, // see above

    pub(crate) MaxWavesPerSIMD: u32, // This identifies the max. number of launched waves per SIMD.
    // If NumFComputeCores is 0, this value is ignored.
    pub(crate) LDSSizeInKB: u32, // Size of Local Data Store in Kilobytes per SIMD Wavefront
    pub(crate) GDSSizeInKB: u32, // Size of Global Data Store in Kilobytes shared across SIMD Wavefronts

    pub(crate) WaveFrontSize: u32, // Number of SIMD cores per wavefront executed, typically 64,
    // may be 32 or a different value for some HSA based architectures
    pub(crate) NumShaderBanks: u32, // Number of Shader Banks or Shader Engines, typical values are 1 or 2

    pub(crate) NumArrays: u32,     // Number of SIMD arrays per engine
    pub(crate) NumCUPerArray: u32, // Number of Compute Units (CU) per SIMD array
    pub(crate) NumSIMDPerCU: u32,  // Number of SIMD representing a Compute Unit (CU)

    pub(crate) MaxSlotsScratchCU: u32, // Number of temp. memory ("scratch") wave slots available to access,
    // may be 0 if HW has no restrictions
    pub(crate) EngineId: HSA_ENGINE_ID, // Identifier (rev) of the GPU uEngine or Firmware, may be 0
    OverrideEngineId: HSA_ENGINE_ID, // Identifier (rev) of the Overrided GPU uEngine or Firmware, may be 0

    pub(crate) VendorId: u16, // GPU vendor id; 0 on latency (= CPU)-only nodes
    pub(crate) DeviceId: u16, // GPU device id; 0 on latency (= CPU)-only nodes

    pub(crate) LocationId: u32, // GPU BDF (Bus/Device/function number) - identifies the device
    // location in the overall system
    pub(crate) LocalMemSize: u64,              // Local memory size
    pub(crate) MaxEngineClockMhzFCompute: u32, // maximum engine clocks for CPU and
    pub(crate) MaxEngineClockMhzCCompute: u32, // GPU function, including any boost caopabilities,
    pub(crate) DrmRenderMinor: i32,            // DRM render device minor device number
    pub(crate) MarketingName: [u16; HSA_PUBLIC_NAME_SIZE], // Public name of the "device" on the node (board or APU name).
    // Unicode string
    AMDName: [u8; HSA_PUBLIC_NAME_SIZE], //CAL Name of the "device", ASCII
    pub(crate) uCodeEngineVersions: HSA_ENGINE_VERSION,
    pub(crate) DebugProperties: HSA_DEBUG_PROPERTIES, // Debug properties of this node.
    pub(crate) HiveID: u64, // XGMI Hive the GPU node belongs to in the system. It is an opaque and static
    // number hash created by the PSP
    pub(crate) NumSdmaEngines: u32, // number of PCIe optimized SDMA engines
    pub(crate) NumSdmaXgmiEngines: u32, // number of XGMI optimized SDMA engines

    pub(crate) NumSdmaQueuesPerEngine: u8, // number of SDMA queue per one engine
    pub(crate) NumCpQueues: u8,            // number of Compute queues
    pub(crate) NumGws: u8,                 // number of GWS barriers
    pub(crate) Integrated: u8, // 0 - discrete GPU, 1 - integrated GPU (including small APU and APP APU)

    pub(crate) Domain: u32,   // PCI domain of the GPU
    pub(crate) UniqueID: u64, // Globally unique immutable id

    pub(crate) VGPRSizePerCU: u32, // VGPR size in bytes per CU
    pub(crate) SGPRSizePerCU: u32, // SGPR size in bytes per CU

    pub(crate) NumXcc: u32,   // Number of XCC
    pub(crate) KFDGpuID: u32, // GPU Hash ID generated by KFD

    pub(crate) FamilyID: u32, // GPU family id
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaMemoryPropSt {
    SizeInBytesLow: u32, // physical memory size of the memory range in bytes (lower 32bit)
    SizeInBytesHigh: u32, // physical memory size of the memory range in bytes (higher 32bit)
}

#[repr(C)]
pub union HSA_MEMORY_PROP {
    pub SizeInBytes: u64, // physical memory size of the memory range in bytes
    pub ui32: HsaMemoryPropSt,
}

impl Debug for HSA_MEMORY_PROP {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_MEMORY_PROP {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaMemoryProperty {
    HotPluggable: u32, // the memory may be removed by some system action,
    // memory should be used for temporary data
    NonVolatile: u32, // memory content is preserved across a power-off cycle.
    Reserved: u32,
}

impl Default for HsaMemoryProperty {
    fn default() -> Self {
        Self {
            HotPluggable: 1, // the memory may be removed by some system action,
            // memory should be used for temporary data
            NonVolatile: 1, // memory content is preserved across a power-off cycle.
            Reserved: 30,
        }
    }
}

#[repr(C)]
pub union HSA_MEMORYPROPERTY {
    pub(crate) MemoryProperty: u32,
    ui32: HsaMemoryProperty,
}

impl Debug for HSA_MEMORYPROPERTY {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_MEMORYPROPERTY {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HSA_HEAPTYPE {
    HSA_HEAPTYPE_SYSTEM = 0,
    HSA_HEAPTYPE_FRAME_BUFFER_PUBLIC = 1, // CPU "visible" part of GPU device local memory (for discrete GPU)
    HSA_HEAPTYPE_FRAME_BUFFER_PRIVATE = 2, // CPU "invisible" part of GPU device local memory (for discrete GPU)
    // All HSA accessible memory is per definition "CPU visible"
    // "Private memory" is relevant for graphics interop only.
    HSA_HEAPTYPE_GPU_GDS = 3,     // GPU internal memory (GDS)
    HSA_HEAPTYPE_GPU_LDS = 4,     // GPU internal memory (LDS)
    HSA_HEAPTYPE_GPU_SCRATCH = 5, // GPU special memory (scratch)
    HSA_HEAPTYPE_DEVICE_SVM = 6,  // sys-memory mapped by device page tables
    HSA_HEAPTYPE_MMIO_REMAP = 7,  // remapped mmio, such as hdp flush registers

    HSA_HEAPTYPE_NUMHEAPTYPES,
    HSA_HEAPTYPE_SIZE = 0xFFFFFFFF,
}

impl TryFrom<usize> for HSA_HEAPTYPE {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_SYSTEM as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_SYSTEM)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_FRAME_BUFFER_PUBLIC as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_FRAME_BUFFER_PUBLIC)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_FRAME_BUFFER_PRIVATE as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_FRAME_BUFFER_PRIVATE)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_GDS as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_GDS)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_LDS as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_LDS)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_SCRATCH as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_GPU_SCRATCH)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_DEVICE_SVM as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_DEVICE_SVM)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_MMIO_REMAP as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_MMIO_REMAP)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_NUMHEAPTYPES as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_NUMHEAPTYPES)
            }
            x if x == HSA_HEAPTYPE::HSA_HEAPTYPE_SIZE as usize => {
                Ok(HSA_HEAPTYPE::HSA_HEAPTYPE_SIZE)
            }
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct HsaMemoryProperties {
    pub(crate) HeapType: HSA_HEAPTYPE, // system or frame buffer,
    pub prop: HSA_MEMORY_PROP,
    pub Flags: HSA_MEMORYPROPERTY, // See definitions above

    pub(crate) Width: u32, // memory width - the number of parallel bits of the memory interface
    pub(crate) MemoryClockMax: u32, // memory clock for the memory, this allows computing the available bandwidth
    // to the memory when needed
    VirtualBaseAddress: u64, // if set to value != 0, indicates the virtual base address of the memory // in process virtual space
}

impl Default for HsaMemoryProperties {
    fn default() -> Self {
        Self {
            HeapType: HSA_HEAPTYPE::HSA_HEAPTYPE_SYSTEM,
            prop: HSA_MEMORY_PROP { SizeInBytes: 0 },
            Flags: HSA_MEMORYPROPERTY { MemoryProperty: 0 },
            Width: 0,
            MemoryClockMax: 0,
            VirtualBaseAddress: 0,
        }
    }
}

//
// Discoverable Cache Properties. (optional).
// The structure is the output parameter of the hsaKmtGetNodeMemoryProperties() function
// Any of the parameters may be 0 (= not defined)
//

pub const HSA_CPU_SIBLINGS: usize = 256;
pub const HSA_PROCESSORID_ALL: usize = 0xFFFFFFFF;

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaCacheTypeSt {
    pub Data: u32,
    pub Instruction: u32,
    pub CPU: u32,
    pub HSACU: u32,
    pub Reserved: u32,
}

impl Default for HsaCacheTypeSt {
    fn default() -> Self {
        Self {
            Data: 1,
            Instruction: 1,
            CPU: 1,
            HSACU: 1,
            Reserved: 28,
        }
    }
}

#[repr(C)]
pub union HsaCacheType {
    pub Value: u32,
    pub ui32: HsaCacheTypeSt,
}

impl Debug for HsaCacheType {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HsaCacheType {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct HsaCacheProperties {
    pub(crate) ProcessorIdLow: u32, // Identifies the processor number

    pub(crate) CacheLevel: u32, // Integer representing level: 1, 2, 3, 4, etc
    pub(crate) CacheSize: u32,  // Size of the cache
    pub(crate) CacheLineSize: u32, // Cache line size in bytes
    pub(crate) CacheLinesPerTag: u32, // Cache lines per Cache Tag
    pub(crate) CacheAssociativity: u32, // Cache Associativity
    pub(crate) CacheLatency: u32, // Cache latency in ns
    pub(crate) CacheType: HsaCacheType,
    pub SiblingMap: [u32; HSA_CPU_SIBLINGS],
}

impl Default for HsaCacheProperties {
    fn default() -> Self {
        Self {
            ProcessorIdLow: 0,
            CacheLevel: 0,
            CacheSize: 0,
            CacheLineSize: 0,
            CacheLinesPerTag: 0,
            CacheAssociativity: 0,
            CacheLatency: 0,
            CacheType: HsaCacheType { Value: 0 },
            SiblingMap: [0; 256],
        }
    }
}

//
// Discoverable IoLink Properties (optional).
// The structure is the output parameter of the hsaKmtGetIoLinkProperties() function.
// Any of the parameters may be 0 (= not defined)
//

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum HSA_IOLINKTYPE {
    HSA_IOLINKTYPE_UNDEFINED = 0,
    HSA_IOLINKTYPE_HYPERTRANSPORT = 1,
    HSA_IOLINKTYPE_PCIEXPRESS = 2,
    HSA_IOLINKTYPE_AMBA = 3,
    HSA_IOLINKTYPE_MIPI = 4,
    HSA_IOLINK_TYPE_QPI_1_1 = 5,
    HSA_IOLINK_TYPE_RESERVED1 = 6,
    HSA_IOLINK_TYPE_RESERVED2 = 7,
    HSA_IOLINK_TYPE_RAPID_IO = 8,
    HSA_IOLINK_TYPE_INFINIBAND = 9,
    HSA_IOLINK_TYPE_RESERVED3 = 10,
    HSA_IOLINK_TYPE_XGMI = 11,
    HSA_IOLINK_TYPE_XGOP = 12,
    HSA_IOLINK_TYPE_GZ = 13,
    HSA_IOLINK_TYPE_ETHERNET_RDMA = 14,
    HSA_IOLINK_TYPE_RDMA_OTHER = 15,
    HSA_IOLINK_TYPE_OTHER = 16,
    HSA_IOLINKTYPE_NUMIOLINKTYPES,
    HSA_IOLINKTYPE_SIZE = 0xFFFFFFFF,
}

impl TryFrom<usize> for HSA_IOLINKTYPE {
    type Error = ();

    fn try_from(v: usize) -> Result<Self, Self::Error> {
        match v {
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_UNDEFINED as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_UNDEFINED)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_HYPERTRANSPORT as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_HYPERTRANSPORT)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_PCIEXPRESS as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_PCIEXPRESS)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_AMBA as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_AMBA)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_MIPI as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_MIPI)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_QPI_1_1 as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_QPI_1_1)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED1 as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED1)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED2 as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED2)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RAPID_IO as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RAPID_IO)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_INFINIBAND as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_INFINIBAND)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED3 as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RESERVED3)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_XGMI as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_XGMI)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_XGOP as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_XGOP)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_GZ as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_GZ)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_ETHERNET_RDMA as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_ETHERNET_RDMA)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RDMA_OTHER as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_RDMA_OTHER)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINK_TYPE_OTHER as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINK_TYPE_OTHER)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_NUMIOLINKTYPES as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_NUMIOLINKTYPES)
            }
            x if x == HSA_IOLINKTYPE::HSA_IOLINKTYPE_SIZE as usize => {
                Ok(HSA_IOLINKTYPE::HSA_IOLINKTYPE_SIZE)
            }
            _ => Err(()),
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaLinkPropertySt {
    Override: u32, // bus link properties are determined by this structure
    // not by the HSA_IOLINKTYPE. The other flags are valid
    // only if this bit is set to one
    NonCoherent: u32, // The link doesn't support coherent transactions
    // memory accesses across must not be set to "host cacheable"!
    NoAtomics32bit: u32, // The link doesn't support 32bit-wide atomic transactions
    NoAtomics64bit: u32, // The link doesn't support 64bit-wide atomic transactions
    NoPeerToPeerDMA: u32, // The link doesn't allow device P2P access
    Reserved: u32,
}

impl Default for HsaLinkPropertySt {
    fn default() -> Self {
        Self {
            Override: 1, // bus link properties are determined by this structure
            // not by the HSA_IOLINKTYPE. The other flags are valid
            // only if this bit is set to one
            NonCoherent: 1, // The link doesn't support coherent transactions
            // memory accesses across must not be set to "host cacheable"!
            NoAtomics32bit: 1, // The link doesn't support 32bit-wide atomic transactions
            NoAtomics64bit: 1, // The link doesn't support 64bit-wide atomic transactions
            NoPeerToPeerDMA: 1, // The link doesn't allow device P2P access
            Reserved: 27,
        }
    }
}

#[repr(C)]
pub union HSA_LINKPROPERTY {
    pub LinkProperty: u32,
    ui32: HsaLinkPropertySt,
}

impl Debug for HSA_LINKPROPERTY {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl PartialEq for HSA_LINKPROPERTY {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub struct HsaIoLinkProperties {
    pub(crate) IoLinkType: HSA_IOLINKTYPE, // see above
    pub(crate) VersionMajor: u32,          // Bus interface version (optional)
    pub(crate) VersionMinor: u32,          // Bus interface version (optional)

    pub(crate) NodeFrom: u32, //
    pub(crate) NodeTo: u32,   //

    pub(crate) Weight: u32, // weight factor (derived from CDIT)

    pub(crate) MinimumLatency: u32, // minimum cost of time to transfer (rounded to ns)
    pub(crate) MaximumLatency: u32, // maximum cost of time to transfer (rounded to ns)
    pub(crate) MinimumBandwidth: u32, // minimum interface Bandwidth in MB/s
    pub(crate) MaximumBandwidth: u32, // maximum interface Bandwidth in MB/s
    pub(crate) RecTransferSize: u32, // recommended transfer size to reach maximum bandwidth in Bytes
    pub(crate) RecSdmaEngIdMask: u32, // recommended sdma engine IDs to reach maximum bandwidth
    pub Flags: HSA_LINKPROPERTY,     // override flags (may be active for specific platforms)
}

impl Default for HsaIoLinkProperties {
    fn default() -> Self {
        Self {
            IoLinkType: HSA_IOLINKTYPE::HSA_IOLINKTYPE_UNDEFINED,
            VersionMajor: 0,
            VersionMinor: 0,
            NodeFrom: 0,
            NodeTo: 0,
            Weight: 0,
            MinimumLatency: 0,
            MaximumLatency: 0,
            MinimumBandwidth: 0,
            MaximumBandwidth: 0,
            RecTransferSize: 0,
            RecSdmaEngIdMask: 0,
            Flags: HSA_LINKPROPERTY { LinkProperty: 0 },
        }
    }
}

// FROM TOPOLOGY

// TODO impl Clone
#[derive(Debug, PartialEq)]
pub struct node_props_t {
    pub(crate) node: HsaNodeProperties,
    pub(crate) mem: Vec<HsaMemoryProperties>, /* node->NumBanks elements */
    pub cache: Vec<HsaCacheProperties>,
    pub link: Vec<HsaIoLinkProperties>,
}

impl node_props_t {
    pub fn new() -> Self {
        Self {
            node: HsaNodeProperties {
                NumCPUCores: 0,
                NumFComputeCores: 0,
                NumNeuralCores: 0,
                NumMemoryBanks: 0,
                NumCaches: 0,
                NumIOLinks: 0,
                CComputeIdLo: 0,
                FComputeIdLo: 0,
                Capability: HSA_CAPABILITY { Value: 0 },
                MaxWavesPerSIMD: 0,
                LDSSizeInKB: 0,
                GDSSizeInKB: 0,
                WaveFrontSize: 0,
                NumShaderBanks: 0,
                NumArrays: 0,
                NumCUPerArray: 0,
                NumSIMDPerCU: 0,
                MaxSlotsScratchCU: 0,
                EngineId: HSA_ENGINE_ID { Value: 0 },
                OverrideEngineId: HSA_ENGINE_ID { Value: 0 },
                VendorId: 0,
                DeviceId: 0,
                LocationId: 0,
                LocalMemSize: 0,
                MaxEngineClockMhzFCompute: 0,
                MaxEngineClockMhzCCompute: 0,
                DrmRenderMinor: 0,
                MarketingName: [0; 64],
                AMDName: [0; 64],
                uCodeEngineVersions: HSA_ENGINE_VERSION { Value: 0 },
                DebugProperties: HSA_DEBUG_PROPERTIES { Value: 0 },
                HiveID: 0,
                NumSdmaEngines: 0,
                NumSdmaXgmiEngines: 0,
                NumSdmaQueuesPerEngine: 0,
                NumCpQueues: 0,
                NumGws: 0,
                Integrated: 0,
                Domain: 0,
                UniqueID: 0,
                VGPRSizePerCU: 0,
                SGPRSizePerCU: 0,
                NumXcc: 0,
                KFDGpuID: 0,
                FamilyID: 0,
            },
            mem: vec![],
            cache: vec![],
            link: vec![],
        }
    }
}

impl Default for node_props_t {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq)]
pub struct hsa_gfxip_table {
    pub(crate) device_id: u16, // Device ID
    pub(crate) major: u8,      // GFXIP Major engine version
    pub(crate) minor: u8,      // GFXIP Minor engine version
    pub(crate) stepping: u8,   // GFXIP Stepping info
    amd_name: &'static str,    // CALName of the device
}

impl hsa_gfxip_table {
    pub fn new(
        device_id: u16,         // Device ID
        major: u8,              // GFXIP Major engine version
        minor: u8,              // GFXIP Minor engine version
        stepping: u8,           // GFXIP Stepping info
        amd_name: &'static str, // CALName of the device
    ) -> Self {
        Self {
            device_id,
            major,
            minor,
            stepping,
            amd_name,
        }
    }
}

pub fn get_hsa_gfxip_table() -> [hsa_gfxip_table; 180] {
    [
        /* Kaveri Family */
        hsa_gfxip_table::new(0x1304, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1305, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1306, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1307, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1309, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130A, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130B, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130C, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130D, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130E, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x130F, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1310, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1311, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1312, 7, 0, 0, "Spooky"),
        hsa_gfxip_table::new(0x1313, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1315, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x1316, 7, 0, 0, "Spooky"),
        hsa_gfxip_table::new(0x1317, 7, 0, 0, "Spooky"),
        hsa_gfxip_table::new(0x1318, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x131B, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x131C, 7, 0, 0, "Spectre"),
        hsa_gfxip_table::new(0x131D, 7, 0, 0, "Spectre"),
        /* Hawaii Family */
        hsa_gfxip_table::new(0x67A0, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67A1, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67A2, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67A8, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67A9, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67AA, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67B0, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67B1, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67B8, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67B9, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67BA, 7, 0, 1, "Hawaii"),
        hsa_gfxip_table::new(0x67BE, 7, 0, 1, "Hawaii"),
        /* Carrizo Family */
        hsa_gfxip_table::new(0x9870, 8, 0, 1, "Carrizo"),
        hsa_gfxip_table::new(0x9874, 8, 0, 1, "Carrizo"),
        hsa_gfxip_table::new(0x9875, 8, 0, 1, "Carrizo"),
        hsa_gfxip_table::new(0x9876, 8, 0, 1, "Carrizo"),
        hsa_gfxip_table::new(0x9877, 8, 0, 1, "Carrizo"),
        /* Tonga Family */
        hsa_gfxip_table::new(0x6920, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6921, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6928, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6929, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x692B, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x692F, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6930, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6938, 8, 0, 2, "Tonga"),
        hsa_gfxip_table::new(0x6939, 8, 0, 2, "Tonga"),
        /* Fiji */
        hsa_gfxip_table::new(0x7300, 8, 0, 3, "Fiji"),
        hsa_gfxip_table::new(0x730F, 8, 0, 3, "Fiji"),
        /* Polaris10 */
        hsa_gfxip_table::new(0x67C0, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C1, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C2, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C4, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C7, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C8, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67C9, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67CA, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67CC, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67CF, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67D0, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x67DF, 8, 0, 3, "Polaris10"),
        hsa_gfxip_table::new(0x6FDF, 8, 0, 3, "Polaris10"),
        /* Polaris11 */
        hsa_gfxip_table::new(0x67E0, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67E1, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67E3, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67E7, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67E8, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67E9, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67EB, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67EF, 8, 0, 3, "Polaris11"),
        hsa_gfxip_table::new(0x67FF, 8, 0, 3, "Polaris11"),
        /* Polaris12 */
        hsa_gfxip_table::new(0x6980, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6981, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6985, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6986, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6987, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6995, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x6997, 8, 0, 3, "Polaris12"),
        hsa_gfxip_table::new(0x699F, 8, 0, 3, "Polaris12"),
        /* VegaM */
        hsa_gfxip_table::new(0x694C, 8, 0, 3, "VegaM"),
        hsa_gfxip_table::new(0x694E, 8, 0, 3, "VegaM"),
        hsa_gfxip_table::new(0x694F, 8, 0, 3, "VegaM"),
        /* Vega10 */
        hsa_gfxip_table::new(0x6860, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6861, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6862, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6863, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6864, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6867, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6868, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x6869, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x686A, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x686B, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x686C, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x686D, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x686E, 9, 0, 0, "Vega10"),
        hsa_gfxip_table::new(0x687F, 9, 0, 0, "Vega10"),
        /* Vega12 */
        hsa_gfxip_table::new(0x69A0, 9, 0, 4, "Vega12"),
        hsa_gfxip_table::new(0x69A1, 9, 0, 4, "Vega12"),
        hsa_gfxip_table::new(0x69A2, 9, 0, 4, "Vega12"),
        hsa_gfxip_table::new(0x69A3, 9, 0, 4, "Vega12"),
        hsa_gfxip_table::new(0x69Af, 9, 0, 4, "Vega12"),
        /* Raven */
        hsa_gfxip_table::new(0x15DD, 9, 0, 2, "Raven"),
        hsa_gfxip_table::new(0x15D8, 9, 0, 2, "Raven"),
        /* Vega20 */
        hsa_gfxip_table::new(0x66A0, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66A1, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66A2, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66A3, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66A4, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66A7, 9, 0, 6, "Vega20"),
        hsa_gfxip_table::new(0x66AF, 9, 0, 6, "Vega20"),
        /* Arcturus */
        hsa_gfxip_table::new(0x7388, 9, 0, 8, "Arcturus"),
        hsa_gfxip_table::new(0x738C, 9, 0, 8, "Arcturus"),
        hsa_gfxip_table::new(0x738E, 9, 0, 8, "Arcturus"),
        hsa_gfxip_table::new(0x7390, 9, 0, 8, "Arcturus"),
        /* Aldebaran */
        hsa_gfxip_table::new(0x7408, 9, 0, 10, "Aldebaran"),
        hsa_gfxip_table::new(0x740C, 9, 0, 10, "Aldebaran"),
        hsa_gfxip_table::new(0x740F, 9, 0, 10, "Aldebaran"),
        hsa_gfxip_table::new(0x7410, 9, 0, 10, "Aldebaran"),
        /* Renoir */
        hsa_gfxip_table::new(0x15E7, 9, 0, 12, "Renoir"),
        hsa_gfxip_table::new(0x1636, 9, 0, 12, "Renoir"),
        hsa_gfxip_table::new(0x1638, 9, 0, 12, "Renoir"),
        hsa_gfxip_table::new(0x164C, 9, 0, 12, "Renoir"),
        /* Navi10 */
        hsa_gfxip_table::new(0x7310, 10, 1, 0, "Navi10"),
        hsa_gfxip_table::new(0x7312, 10, 1, 0, "Navi10"),
        hsa_gfxip_table::new(0x7318, 10, 1, 0, "Navi10"),
        hsa_gfxip_table::new(0x731A, 10, 1, 0, "Navi10"),
        hsa_gfxip_table::new(0x731E, 10, 1, 0, "Navi10"),
        hsa_gfxip_table::new(0x731F, 10, 1, 0, "Navi10"),
        /* cyan_skillfish */
        hsa_gfxip_table::new(0x13F9, 10, 1, 3, "cyan_skillfish"),
        hsa_gfxip_table::new(0x13FA, 10, 1, 3, "cyan_skillfish"),
        hsa_gfxip_table::new(0x13FB, 10, 1, 3, "cyan_skillfish"),
        hsa_gfxip_table::new(0x13FC, 10, 1, 3, "cyan_skillfish"),
        hsa_gfxip_table::new(0x13FE, 10, 1, 3, "cyan_skillfish"),
        hsa_gfxip_table::new(0x143F, 10, 1, 3, "cyan_skillfish"),
        /* Navi14 */
        hsa_gfxip_table::new(0x7340, 10, 1, 2, "Navi14"),
        hsa_gfxip_table::new(0x7341, 10, 1, 2, "Navi14"),
        hsa_gfxip_table::new(0x7347, 10, 1, 2, "Navi14"),
        /* Navi12 */
        hsa_gfxip_table::new(0x7360, 10, 1, 1, "Navi12"),
        hsa_gfxip_table::new(0x7362, 10, 1, 1, "Navi12"),
        /* SIENNA_CICHLID */
        hsa_gfxip_table::new(0x73A0, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A1, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A2, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A3, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A5, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A8, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73A9, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73AC, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73AD, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73AB, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73AE, 10, 3, 0, "SIENNA_CICHLID"),
        hsa_gfxip_table::new(0x73BF, 10, 3, 0, "SIENNA_CICHLID"),
        /* NAVY_FLOUNDER */
        hsa_gfxip_table::new(0x73C0, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73C1, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73C3, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DA, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DB, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DC, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DD, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DE, 10, 3, 1, "NAVY_FLOUNDER"),
        hsa_gfxip_table::new(0x73DF, 10, 3, 1, "NAVY_FLOUNDER"),
        /* DIMGREY_CAVEFISH */
        hsa_gfxip_table::new(0x73E0, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73E1, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73E2, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73E8, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73E9, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73EA, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73EB, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73EC, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73ED, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73EF, 10, 3, 2, "DIMGREY_CAVEFISH"),
        hsa_gfxip_table::new(0x73FF, 10, 3, 2, "DIMGREY_CAVEFISH"),
        /* VanGogh */
        hsa_gfxip_table::new(0x163F, 10, 3, 3, "VanGogh"),
        /* BEIGE_GOBY */
        hsa_gfxip_table::new(0x7420, 10, 3, 4, "BEIGE_GOBY"),
        hsa_gfxip_table::new(0x7421, 10, 3, 4, "BEIGE_GOBY"),
        hsa_gfxip_table::new(0x7422, 10, 3, 4, "BEIGE_GOBY"),
        hsa_gfxip_table::new(0x7423, 10, 3, 4, "BEIGE_GOBY"),
        hsa_gfxip_table::new(0x743F, 10, 3, 4, "BEIGE_GOBY"),
        /* Yellow_Carp */
        hsa_gfxip_table::new(0x164D, 10, 3, 5, "YELLOW_CARP"),
        hsa_gfxip_table::new(0x1681, 10, 3, 5, "YELLOW_CARP"),
    ]
}

/* Calculate VGPR and SGPR register file size per CU */
pub const SGPR_SIZE_PER_CU: usize = 0x4000;

pub const GFX_VERSION_KAVERI: usize = 0x070000;
pub const GFX_VERSION_HAWAII: usize = 0x070001;
pub const GFX_VERSION_CARRIZO: usize = 0x080001;
pub const GFX_VERSION_TONGA: usize = 0x080002;
pub const GFX_VERSION_FIJI: usize = 0x080003;
pub const GFX_VERSION_POLARIS10: usize = 0x080003;
pub const GFX_VERSION_POLARIS11: usize = 0x080003;
pub const GFX_VERSION_POLARIS12: usize = 0x080003;
pub const GFX_VERSION_VEGAM: usize = 0x080003;
pub const GFX_VERSION_VEGA10: usize = 0x090000;
pub const GFX_VERSION_RAVEN: usize = 0x090002;
pub const GFX_VERSION_VEGA12: usize = 0x090004;
pub const GFX_VERSION_VEGA20: usize = 0x090006;
pub const GFX_VERSION_ARCTURUS: usize = 0x090008;
pub const GFX_VERSION_ALDEBARAN: usize = 0x09000A;
pub const GFX_VERSION_AQUA_VANJARAM: usize = 0x090400;
pub const GFX_VERSION_RENOIR: usize = 0x09000C;
pub const GFX_VERSION_NAVI10: usize = 0x0A0100;
pub const GFX_VERSION_NAVI12: usize = 0x0A0101;
pub const GFX_VERSION_NAVI14: usize = 0x0A0102;
pub const GFX_VERSION_CYAN_SKILLFISH: usize = 0x0A0103;
pub const GFX_VERSION_SIENNA_CICHLID: usize = 0x0A0300;
pub const GFX_VERSION_NAVY_FLOUNDER: usize = 0x0A0301;
pub const GFX_VERSION_DIMGREY_CAVEFISH: usize = 0x0A0302;
pub const GFX_VERSION_VANGOGH: usize = 0x0A0303;
pub const GFX_VERSION_BEIGE_GOBY: usize = 0x0A0304;
pub const GFX_VERSION_YELLOW_CARP: usize = 0x0A0305;
pub const GFX_VERSION_PLUM_BONITO: usize = 0x0B0000;
pub const GFX_VERSION_WHEAT_NAS: usize = 0x0B0001;
pub const GFX_VERSION_GFX1200: usize = 0x0C0000;
pub const GFX_VERSION_GFX1201: usize = 0x0C0001;

/* Expects HSA_ENGINE_ID.ui32, returns gfxv (full) in hex */
pub fn HSA_GET_GFX_VERSION_FULL(ui32: &HsaEngineId) -> u32 {
    ((ui32.Major) << 16) | ((ui32.Minor) << 8) | (ui32.Stepping)
}

pub const GPU_HUGE_PAGE_SIZE: usize = 2 << 20;

// #define ALIGN_UP(x,align) (((uint64_t)(x) + (align) - 1) & ~(uint64_t)((align)-1))
pub fn ALIGN_UP(x: u64, align: u64) -> u64 {
    ((x) + (align) - 1) & !((align) - 1)
}

//
// Memory allocation definitions for the KFD HSA interface
//

#[derive(Debug, Copy, Clone)]
pub struct HsaMemFlagSt {
    pub NonPaged: u32,     // default = 0: pageable memory
    pub CachePolicy: u32,  // see HSA_CACHING_TYPE
    pub ReadOnly: u32,     // default = 0: Read/Write memory
    pub PageSize: u32,     // see HSA_PAGE_SIZE
    pub HostAccess: u32,   // default = 0: GPU access only
    pub NoSubstitute: u32, // default = 0: if specific memory is not available on node (e.g. on
    // discrete GPU local), allocation may fall back to system memory node 0
    // memory (= always available). Otherwise no allocation is possible.
    pub GDSMemory: u32, // default = 0: If set, the allocation will occur in GDS heap.
    // HostAccess must be 0, all other flags (except NoSubstitute) should
    // be 0 when setting this entry to 1. GDS allocation may fail due to
    // limited resources. Application code is required to work without
    // any allocated GDS memory using regular memory.
    // Allocation fails on any node without GPU function.
    pub Scratch: u32, // default = 0: If set, the allocation will occur in GPU "scratch area".
    // HostAccess must be 0, all other flags (except NoSubstitute) should be 0
    // when setting this entry to 1. Scratch allocation may fail due to limited
    // resources. Application code is required to work without any allocation.
    // Allocation fails on any node without GPU function.
    pub AtomicAccessFull: u32, // default = 0: If set, the memory will be allocated and mapped to allow
    // atomic ops processing. On AMD APU, this will use the ATC path on system
    // memory, irrespective of the NonPaged flag setting (= if NonPaged is set,
    // the memory is pagelocked but mapped through IOMMUv2 instead of GPUVM).
    // All atomic ops must be supported on this memory.
    pub AtomicAccessPartial: u32, // default = 0: See above for AtomicAccessFull description, however
    // focused on AMD discrete GPU that support PCIe atomics; the memory
    // allocation is mapped to allow for PCIe atomics to operate on system
    // memory, irrespective of NonPaged set or the presence of an ATC path
    // in the system. The atomic operations supported are limited to SWAP,
    // CompareAndSwap (CAS) and FetchAdd (this PCIe op allows both atomic
    // increment and decrement via 2-complement arithmetic), which are the
    // only atomic ops directly supported in PCI Express.
    // On AMD APU, setting this flag will allocate the same type of memory
    // as AtomicAccessFull, but it will be considered compatible with
    // discrete GPU atomic operations access.
    pub ExecuteAccess: u32, // default = 0: Identifies if memory is primarily used for data or accessed
    // for executable code (e.g. queue memory) by the host CPU or the device.
    // Influences the page attribute setting within the allocation
    pub CoarseGrain: u32, // default = 0: The memory can be accessed assuming cache
    // coherency maintained by link infrastructure and HSA agents.
    // 1: memory consistency needs to be enforced at
    // synchronization points at dispatch or other software
    // enforced synchronization boundaries.
    pub AQLQueueMemory: u32, // default = 0; If 1: The caller indicates that the memory will be used as AQL queue memory.
    // The KFD will ensure that the memory returned is allocated in the optimal memory location
    // and optimal alignment requirements
    pub FixedAddress: u32, // Allocate memory at specified virtual address. Fail if address is not free.
    pub NoNUMABind: u32,   // Don't bind system memory to a specific NUMA node
    pub Uncached: u32,     // Caching flag for fine-grained memory on A+A HW platform
    pub NoAddress: u32, // only do vram allocation, return a handle, not allocate virtual address.
    pub OnlyAddress: u32, // only do virtal address allocation without vram allocation.
    pub ExtendedCoherent: u32, // system-scope coherence on atomic instructions
    pub GTTAccess: u32, // default = 0; If 1: The caller indicates this memory will be mapped to GART for MES
    // KFD will allocate GTT memory with the Preferred_node set as gpu_id for GART mapping
    pub Contiguous: u32, // Allocate contiguous VRAM
    pub Reserved: u32,
}

impl Default for HsaMemFlagSt {
    fn default() -> Self {
        Self {
            NonPaged: 1,     // default = 0: pageable memory
            CachePolicy: 2,  // see HSA_CACHING_TYPE
            ReadOnly: 1,     // default = 0: Read/Write memory
            PageSize: 2,     // see HSA_PAGE_SIZE
            HostAccess: 1,   // default = 0: GPU access only
            NoSubstitute: 1, // default = 0: if specific memory is not available on node (e.g. on
            // discrete GPU local), allocation may fall back to system memory node 0
            // memory (= always available). Otherwise no allocation is possible.
            GDSMemory: 1, // default = 0: If set, the allocation will occur in GDS heap.
            // HostAccess must be 0, all other flags (except NoSubstitute) should
            // be 0 when setting this entry to 1. GDS allocation may fail due to
            // limited resources. Application code is required to work without
            // any allocated GDS memory using regular memory.
            // Allocation fails on any node without GPU function.
            Scratch: 1, // default = 0: If set, the allocation will occur in GPU "scratch area".
            // HostAccess must be 0, all other flags (except NoSubstitute) should be 0
            // when setting this entry to 1. Scratch allocation may fail due to limited
            // resources. Application code is required to work without any allocation.
            // Allocation fails on any node without GPU function.
            AtomicAccessFull: 1, // default = 0: If set, the memory will be allocated and mapped to allow
            // atomic ops processing. On AMD APU, this will use the ATC path on system
            // memory, irrespective of the NonPaged flag setting (= if NonPaged is set,
            // the memory is pagelocked but mapped through IOMMUv2 instead of GPUVM).
            // All atomic ops must be supported on this memory.
            AtomicAccessPartial: 1, // default = 0: See above for AtomicAccessFull description, however
            // focused on AMD discrete GPU that support PCIe atomics, the memory
            // allocation is mapped to allow for PCIe atomics to operate on system
            // memory, irrespective of NonPaged set or the presence of an ATC path
            // in the system. The atomic operations supported are limited to SWAP,
            // CompareAndSwap (CAS) and FetchAdd (this PCIe op allows both atomic
            // increment and decrement via 2-complement arithmetic), which are the
            // only atomic ops directly supported in PCI Express.
            // On AMD APU, setting this flag will allocate the same type of memory
            // as AtomicAccessFull, but it will be considered compatible with
            // discrete GPU atomic operations access.
            ExecuteAccess: 1, // default = 0: Identifies if memory is primarily used for data or accessed
            // for executable code (e.g. queue memory) by the host CPU or the device.
            // Influences the page attribute setting within the allocation
            CoarseGrain: 1, // default = 0: The memory can be accessed assuming cache
            // coherency maintained by link infrastructure and HSA agents.
            // 1: memory consistency needs to be enforced at
            // synchronization points at dispatch or other software
            // enforced synchronization boundaries.
            AQLQueueMemory: 1, // default = 0, If 1: The caller indicates that the memory will be used as AQL queue memory.
            // The KFD will ensure that the memory returned is allocated in the optimal memory location
            // and optimal alignment requirements
            FixedAddress: 1, // Allocate memory at specified virtual address. Fail if address is not free.
            NoNUMABind: 1,   // Don't bind system memory to a specific NUMA node
            Uncached: 1,     // Caching flag for fine-grained memory on A+A HW platform
            NoAddress: 1, // only do vram allocation, return a handle, not allocate virtual address.
            OnlyAddress: 1, // only do virtal address allocation without vram allocation.
            ExtendedCoherent: 1, // system-scope coherence on atomic instructions
            GTTAccess: 1, // default = 0; If 1: The caller indicates this memory will be mapped to GART for MES
            // KFD will allocate GTT memory with the Preferred_node set as gpu_id for GART mapping
            Contiguous: 1, // Allocate contiguous VRAM
            Reserved: 9,
        }
    }
}

pub union HsaMemFlagUnion {
    pub ui32: HsaMemFlagSt,
    pub Value: u32,
}

pub struct HsaMemFlags {
    pub st: HsaMemFlagUnion,
}
