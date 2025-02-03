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
