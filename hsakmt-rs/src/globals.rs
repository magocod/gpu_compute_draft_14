use std::sync::Mutex;

// HSAKMT global data

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct HsaKmtGlobal {
    pub hsakmt_kfd_fd: i32,
    pub hsakmt_kfd_open_count: u64,
    pub hsakmt_system_properties_count: u64,
    // hsakmt_mutex
    pub hsakmt_is_dgpu: bool,
    pub hsakmt_page_size: i32,
    pub hsakmt_page_shift: i32,
    /* whether to check all dGPUs in the topology support SVM API */
    pub hsakmt_is_svm_api_supported: bool,
    /* zfb is mainly used during emulation */
    pub hsakmt_zfb_support: i32,
}

static HSA_KMT_GLOBAL: Mutex<HsaKmtGlobal> = Mutex::new(HsaKmtGlobal {
    hsakmt_kfd_fd: -1,
    hsakmt_kfd_open_count: 0,
    hsakmt_system_properties_count: 0,
    hsakmt_is_dgpu: false,
    hsakmt_page_size: 0,
    hsakmt_page_shift: 0,
    hsakmt_is_svm_api_supported: false,
    hsakmt_zfb_support: 0,
});

pub fn hsakmt_global_print() {
    println!("{:#?}", HSA_KMT_GLOBAL.lock().unwrap());
}

pub fn hsakmt_global_get() -> HsaKmtGlobal {
    *HSA_KMT_GLOBAL.lock().unwrap()
}

pub fn hsakmt_kfd_open_count_increase() {
    HSA_KMT_GLOBAL.lock().unwrap().hsakmt_kfd_open_count += 1;
}

pub fn hsakmt_kfd_fd_set(hsakmt_kfd_fd: i32) {
    HSA_KMT_GLOBAL.lock().unwrap().hsakmt_kfd_fd = hsakmt_kfd_fd;
}

pub fn hsakmt_page_size_set(hsakmt_page_size: i32) {
    HSA_KMT_GLOBAL.lock().unwrap().hsakmt_page_size = hsakmt_page_size;
}

pub fn hsakmt_page_shift_set(hsakmt_page_shift: i32) {
    HSA_KMT_GLOBAL.lock().unwrap().hsakmt_page_shift = hsakmt_page_shift;
}

pub fn hsakmt_global_is_svm_api_supported_set(is_svm_api_supported: bool) {
    HSA_KMT_GLOBAL.lock().unwrap().hsakmt_is_svm_api_supported = is_svm_api_supported;
}

pub fn check_kfd_open_and_panic() {
    if HSA_KMT_GLOBAL.lock().unwrap().hsakmt_kfd_open_count == 0 {
        panic!("HSAKMT_STATUS_KERNEL_IO_CHANNEL_NOT_OPENED");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsakmt_open_kfd() {
        hsakmt_kfd_open_count_increase();
        hsakmt_global_print();
    }
}
