use crate::globals::hsakmt_global_get;
use crate::libhsakmt::hsakmt_ioctl;
use crate::types::HsakmtStatus::{
    HSAKMT_STATUS_DRIVER_MISMATCH, HSAKMT_STATUS_ERROR, HSAKMT_STATUS_SUCCESS,
};
use crate::types::{HsaVersionInfo, HsakmtStatus};
use std::sync::Mutex;

static VERSION_GLOBAL: Mutex<HsaVersionInfo> = Mutex::new(HsaVersionInfo {
    KernelInterfaceMajorVersion: 0,
    KernelInterfaceMinorVersion: 0,
});

pub fn hsa_kmt_get_version() -> HsaVersionInfo {
    // CHECK_KFD_OPEN();
    *VERSION_GLOBAL.lock().unwrap()
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct KfdIoctlGetVersionArgs {
    major_version: u32, /* from KFD */
    minor_version: u32, /* from KFD */
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn hsakmt_init_kfd_version() -> HsakmtStatus {
    let mut args = KfdIoctlGetVersionArgs {
        major_version: 0,
        minor_version: 0,
    };

    let global = hsakmt_global_get();

    // let ioc_get = (((2) << (((0+8)+8)+14)) | (((0x01)) << 0) | ((((std::mem::size_of::<KfdIoctlGetVersionArgs>()))) << ((0+8)+8)));
    let b = ('K' as i32) << 8;
    let b_2 = std::mem::size_of::<KfdIoctlGetVersionArgs>() << (8 + 8);
    let amdkfd_ioc_get_version = ((2) << ((8 + 8) + 14)) | b | ((0x01) << 0) | b_2 as i32;

    // macro AMDKFD_IOC_GET_VERSION ???
    // (((2U) << (((0+8)+8)+14)) | ((('K')) << (0+8)) | (((0x01)) << 0) | ((((sizeof(struct kfd_ioctl_get_version_args)))) << ((0+8)+8)))
    if hsakmt_ioctl(
        global.hsakmt_kfd_fd,
        amdkfd_ioc_get_version as u64,
        &mut args as *mut _ as *mut std::os::raw::c_void,
    ) == -1
    {
        return HSAKMT_STATUS_ERROR;
    }

    let mut hsakmt_kfd_version_info = VERSION_GLOBAL.lock().unwrap();

    hsakmt_kfd_version_info.KernelInterfaceMajorVersion = args.major_version;
    hsakmt_kfd_version_info.KernelInterfaceMinorVersion = args.minor_version;

    if args.major_version != 1 {
        return HSAKMT_STATUS_DRIVER_MISMATCH;
    }

    HSAKMT_STATUS_SUCCESS
}
