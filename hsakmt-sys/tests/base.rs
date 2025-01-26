use hsakmt_sys::bindings::{
    hsaKmtCloseKFD, hsaKmtGetVersion, hsaKmtOpenKFD, HsaVersionInfo,
    _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS,
};

#[test]
fn test_example() {
    unsafe {
        let ret = hsaKmtOpenKFD();
        assert_eq!(ret, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS);

        let mut version_info = std::mem::MaybeUninit::<HsaVersionInfo>::uninit();

        let ret = hsaKmtGetVersion(version_info.as_mut_ptr());
        assert_eq!(ret, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS);

        let version_info = version_info.assume_init();
        println!("version_info: {:?}", version_info);

        let ret = hsaKmtCloseKFD();
        assert_eq!(ret, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS);
    }
}
