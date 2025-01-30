use hsakmt_rs::hsakmttypes::HsakmtStatus::HSAKMT_STATUS_SUCCESS;
use hsakmt_rs::open_close::{hsa_kmt_close_kfd, hsakmt_open_kfd};
use hsakmt_rs::version::hsa_kmt_get_version;

#[test]
fn test_example() {
    unsafe {
        let ret = hsakmt_open_kfd();
        assert_eq!(ret, HSAKMT_STATUS_SUCCESS);

        let version_info = hsa_kmt_get_version();
        println!("version_info: {:?}", version_info);

        let ret = hsa_kmt_close_kfd();
        assert_eq!(ret, HSAKMT_STATUS_SUCCESS);
    }
}
