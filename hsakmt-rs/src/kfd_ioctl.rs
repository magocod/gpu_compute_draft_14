#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

pub type __s8 = ::std::os::raw::c_schar;
pub type __u8 = ::std::os::raw::c_uchar;
pub type __s16 = ::std::os::raw::c_short;
pub type __u16 = ::std::os::raw::c_ushort;
pub type __s32 = ::std::os::raw::c_int;
pub type __u32 = ::std::os::raw::c_uint;
pub type __s64 = ::std::os::raw::c_longlong;
pub type __u64 = ::std::os::raw::c_ulonglong;

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct kfd_process_device_apertures {
    pub lds_base: __u64,      /* from KFD */
    pub lds_limit: __u64,     /* from KFD */
    pub scratch_base: __u64,  /* from KFD */
    pub scratch_limit: __u64, /* from KFD */
    pub gpuvm_base: __u64,    /* from KFD */
    pub gpuvm_limit: __u64,   /* from KFD */
    pub gpu_id: __u32,        /* from KFD */
    pub pad: __u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct kfd_ioctl_get_process_apertures_new_args {
    /* User allocated. Pointer to struct kfd_process_device_apertures
     * filled in by Kernel
     */
    pub kfd_process_device_apertures_ptr: *mut __u64,
    /* to KFD - indicates amount of memory present in
     *  kfd_process_device_apertures_ptr
     * from KFD - Number of entries filled by KFD.
     */
    pub num_of_nodes: __u32,
    pub pad: __u32,
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct kfd_ioctl_acquire_vm_args {
    pub drm_fd: __u32, /* to KFD */
    pub gpu_id: __u32, /* to KFD */
}

/* For kfd_ioctl_set_memory_policy_args.default_policy and alternate_policy */
// #define KFD_IOC_CACHE_POLICY_COHERENT 0
// #define KFD_IOC_CACHE_POLICY_NONCOHERENT 1

/* For kfd_ioctl_set_memory_policy_args.default_policy and alternate_policy */
pub const KFD_IOC_CACHE_POLICY_COHERENT: usize = 0;
pub const KFD_IOC_CACHE_POLICY_NONCOHERENT: usize = 1;

pub struct kfd_ioctl_set_memory_policy_args {
    pub alternate_aperture_base: *mut __u64, /* to KFD */
    pub alternate_aperture_size: __u64,      /* to KFD */

    pub gpu_id: __u32,           /* to KFD */
    pub default_policy: __u32,   /* to KFD */
    pub alternate_policy: __u32, /* to KFD */
    pub pad: __u32,
}

/* Allocation flags: memory types */
pub const KFD_IOC_ALLOC_MEM_FLAGS_VRAM: usize = (1 << 0);
pub const KFD_IOC_ALLOC_MEM_FLAGS_GTT: usize = (1 << 1);
pub const KFD_IOC_ALLOC_MEM_FLAGS_USERPTR: usize = (1 << 2);
pub const KFD_IOC_ALLOC_MEM_FLAGS_DOORBELL: usize = (1 << 3);
pub const KFD_IOC_ALLOC_MEM_FLAGS_MMIO_REMAP: usize = (1 << 4);
/* Allocation flags: attributes/access options */
pub const KFD_IOC_ALLOC_MEM_FLAGS_WRITABLE: usize = (1 << 31);
pub const KFD_IOC_ALLOC_MEM_FLAGS_EXECUTABLE: usize = (1 << 30);
pub const KFD_IOC_ALLOC_MEM_FLAGS_PUBLIC: usize = (1 << 29);
pub const KFD_IOC_ALLOC_MEM_FLAGS_NO_SUBSTITUTE: usize = (1 << 28);
pub const KFD_IOC_ALLOC_MEM_FLAGS_AQL_QUEUE_MEM: usize = (1 << 27);
pub const KFD_IOC_ALLOC_MEM_FLAGS_COHERENT: usize = (1 << 26);
pub const KFD_IOC_ALLOC_MEM_FLAGS_UNCACHED: usize = (1 << 25);
pub const KFD_IOC_ALLOC_MEM_FLAGS_EXT_COHERENT: usize = (1 << 24);
pub const KFD_IOC_ALLOC_MEM_FLAGS_CONTIGUOUS_BEST_EFFORT: usize = (1 << 23);

/* Allocate memory for later SVM (shared virtual memory) mapping.
 *
 * @va_addr:     virtual address of the memory to be allocated
 *               all later mappings on all GPUs will use this address
 * @size:        size in bytes
 * @handle:      buffer handle returned to user mode, used to refer to
 *               this allocation for mapping, unmapping and freeing
 * @mmap_offset: for CPU-mapping the allocation by mmapping a render node
 *               for userptrs this is overloaded to specify the CPU address
 * @gpu_id:      device identifier
 * @flags:       memory type and attributes. See KFD_IOC_ALLOC_MEM_FLAGS above
 */
pub struct kfd_ioctl_alloc_memory_of_gpu_args {
    pub va_addr: *mut __u64, /* to KFD */
    pub size: __u64,         /* to KFD */
    pub handle: __u64,       /* from KFD */
    pub mmap_offset: __u64,  /* to KFD (userptr), from KFD (mmap offset) */
    pub gpu_id: __u32,       /* to KFD */
    pub flags: __u32,
}

/* Free memory allocated with kfd_ioctl_alloc_memory_of_gpu
 *
 * @handle: memory handle returned by alloc
 */
pub struct kfd_ioctl_free_memory_of_gpu_args {
    pub handle: __u64, /* to KFD */
}
