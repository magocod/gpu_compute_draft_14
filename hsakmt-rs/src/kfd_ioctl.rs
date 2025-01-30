#![allow(
    non_camel_case_types,
    non_snake_case,
    dead_code,
    non_upper_case_globals,
    clippy::enum_clike_unportable_variant,
    clippy::mixed_case_hex_literals
)]

pub type __u64 = ::std::os::raw::c_ulonglong;

pub struct kfd_process_device_apertures {
    lds_base: __u64,      /* from KFD */
    lds_limit: __u64,     /* from KFD */
    scratch_base: __u64,  /* from KFD */
    scratch_limit: __u64, /* from KFD */
    gpuvm_base: __u64,    /* from KFD */
    gpuvm_limit: __u64,   /* from KFD */
    gpu_id: __u64,        /* from KFD */
    pad: __u64,
}
