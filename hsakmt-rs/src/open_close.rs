use crate::globals::{
    hsakmt_global_get, hsakmt_kfd_fd_set, hsakmt_kfd_open_count_increase, hsakmt_page_shift_set,
    hsakmt_page_size_set,
};
use crate::topology::hsakmt_topology_sysfs_get_system_props;
use crate::types::HsakmtStatus::{
    HSAKMT_STATUS_KERNEL_ALREADY_OPENED, HSAKMT_STATUS_KERNEL_IO_CHANNEL_NOT_OPENED,
    HSAKMT_STATUS_SUCCESS,
};
use crate::types::{HsaSystemProperties, HsakmtStatus};
use crate::version::hsakmt_init_kfd_version;
use libc::{
    close, dlerror, dlsym, getenv, open, strcmp, sysconf, O_CLOEXEC, O_RDWR, RTLD_DEFAULT,
    _SC_PAGESIZE,
};
use std::ffi::CString;

pub const KFD_DEVICE_NAME: &str = "/dev/kfd";

pub fn ffs(n: i32) -> u32 {
    (n & -n).ilog2() + 1
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn init_page_size() {
    let hsakmt_page_size = sysconf(_SC_PAGESIZE) as i32;
    hsakmt_page_size_set(hsakmt_page_size);
    hsakmt_page_shift_set((ffs(hsakmt_page_size) - 1) as i32);
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn hsakmt_open_kfd() -> HsakmtStatus {
    let mut fd = -1;
    let mut sys_props = HsaSystemProperties {
        NumNodes: 0,
        PlatformOem: 0,
        PlatformId: 0,
        PlatformRev: 0,
    };

    let global = hsakmt_global_get();

    if global.hsakmt_kfd_open_count == 0 {
        let symbol_name = CString::new("amdgpu_device_get_fd").unwrap();

        let hsakmt_fn_amdgpu_device_get_fd = dlsym(RTLD_DEFAULT, symbol_name.as_ptr());
        let error = dlerror();

        if !error.is_null() {
            println!("amdgpu_device_get_fd is not available: {:?}", error);
        } else {
            println!(
                "amdgpu_device_get_fd is available: {:?}",
                hsakmt_fn_amdgpu_device_get_fd
            );
        }

        // result = init_vars_from_env();

        // if (result != HSAKMT_STATUS_SUCCESS)
        // goto open_failed;

        if global.hsakmt_kfd_fd < 0 {
            let kfd_device_name = CString::new(KFD_DEVICE_NAME).unwrap();
            fd = open(kfd_device_name.as_ptr(), O_RDWR | O_CLOEXEC);

            if fd == -1 {
                close(fd);
                return HSAKMT_STATUS_KERNEL_IO_CHANNEL_NOT_OPENED;
            }

            hsakmt_kfd_fd_set(fd);
        }

        init_page_size();

        let ret = hsakmt_init_kfd_version();
        if ret != HSAKMT_STATUS_SUCCESS {
            close(fd);
        }

        let ev = CString::new("HSA_USE_SVM").unwrap();

        let use_svm_str = getenv(ev.as_ptr());

        let ct = CString::new("0").unwrap();
        #[allow(clippy::nonminimal_bool)]
        let _hsakmt_is_svm_api_supported =
            !(!use_svm_str.is_null() && strcmp(use_svm_str, ct.as_ptr()) == 0);

        let ret = hsakmt_topology_sysfs_get_system_props(&mut sys_props);
        if ret != HSAKMT_STATUS_SUCCESS {
            close(fd);
        }

        hsakmt_kfd_open_count_increase();

        // println!("{:#?}", sys_props)

        // hsakmt_init_device_debugging_memory

        // hsakmt_init_counter_props
    } else {
        hsakmt_kfd_open_count_increase();
        return HSAKMT_STATUS_KERNEL_ALREADY_OPENED;
    }

    HSAKMT_STATUS_SUCCESS
}

/// ...
///
///
/// # Safety
///
/// TODO safety function explain
pub unsafe fn hsa_kmt_close_kfd() -> HsakmtStatus {
    HSAKMT_STATUS_SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fmm::hsakmt_fmm_global_print;
    use crate::globals::hsakmt_global_print;
    use crate::version::hsa_kmt_get_version;

    #[test]
    fn test_ffs() {
        let n = 18;
        let r_1 = ffs(n);

        let n = 19;
        let r_2 = ffs(n);

        assert_eq!(r_1, 2);
        assert_eq!(r_2, 1);
    }

    #[test]
    fn test_hsakmt_open_kfd() {
        unsafe {
            let ret = hsakmt_open_kfd();
            assert_eq!(ret, HSAKMT_STATUS_SUCCESS);

            println!("{:#?}", hsa_kmt_get_version())
        }

        hsakmt_global_print();
        hsakmt_fmm_global_print();
    }

    #[test]
    fn test_hsakmt_open_kfd_already_opened() {
        unsafe {
            let ret = hsakmt_open_kfd();
            assert_eq!(ret, HSAKMT_STATUS_SUCCESS);

            let ret = hsakmt_open_kfd();
            assert_eq!(ret, HSAKMT_STATUS_KERNEL_ALREADY_OPENED);
        }

        hsakmt_global_print();
        hsakmt_fmm_global_print();
    }
}
