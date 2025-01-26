//
// HSA STATUS codes returned by the KFD Interfaces
//

#[allow(non_camel_case_types)]
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

#[allow(non_snake_case, dead_code)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaVersionInfo {
    pub KernelInterfaceMajorVersion: u32, // supported kernel interface major version
    pub KernelInterfaceMinorVersion: u32, // supported kernel interface minor version
}

#[allow(non_snake_case, dead_code)]
#[derive(Debug, PartialEq)]
pub struct HsaSystemProperties {
    pub NumNodes: u32, // the number of "H-NUMA" memory nodes.
    // each node represents a discoverable node of the system
    // All other enumeration is done on a per-node basis
    pub PlatformOem: u32, // identifies HSA platform, reflects the OEMID in the CRAT
    pub PlatformId: u32,  // HSA platform ID, reflects OEM TableID in the CRAT
    pub PlatformRev: u32, // HSA platform revision, reflects Platform Table Revision ID
}
