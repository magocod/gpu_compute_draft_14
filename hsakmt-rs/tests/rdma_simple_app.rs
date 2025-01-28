use hsakmt_rs::open_close::hsakmt_open_kfd;
use hsakmt_rs::types::HsakmtStatus::HSAKMT_STATUS_SUCCESS;
use hsakmt_rs::version::hsa_kmt_get_version;
use libc::{open, strerror, O_RDWR};
use std::ffi::CString;

const AMDP2PTEST_DEVICE_PATH: &str = "/dev/amdp2ptest";

static mut RDMA_FD: i32 = -1;

unsafe fn rdma_open() {
    let amd_p2ptest_device_path = CString::new(AMDP2PTEST_DEVICE_PATH).unwrap();

    RDMA_FD = open(amd_p2ptest_device_path.as_ptr(), O_RDWR);

    if RDMA_FD == -1 {
        let ret = std::io::Error::last_os_error().raw_os_error().unwrap();
        panic!(
            "error opening driver (errno= {} / {:?})",
            ret,
            strerror(ret)
        );
    }
}

#[test]
fn rdma_simple_app() {
    unsafe {
        let ret = hsakmt_open_kfd();
        assert_eq!(ret, HSAKMT_STATUS_SUCCESS);

        let version_info = hsa_kmt_get_version();
        println!("{:?}", version_info);

        rdma_open();
    }
}
