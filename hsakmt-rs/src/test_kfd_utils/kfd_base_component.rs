#![allow(non_snake_case, dead_code)]

use crate::open_close::hsakmt_open_kfd;
use crate::topology::hsaKmtAcquireSystemProperties;
use crate::types::HsaSystemProperties;
use crate::types::HsakmtStatus::HSAKMT_STATUS_SUCCESS;

#[derive(Debug)]
pub struct KFDBaseComponentTest {
    m_SystemProperties: HsaSystemProperties,
}

impl KFDBaseComponentTest {
    pub fn new() -> Self {
        Self {
            m_SystemProperties: Default::default(),
        }
    }

    pub unsafe fn set_up(&mut self) {
        let ret = hsakmt_open_kfd();
        assert_eq!(ret, HSAKMT_STATUS_SUCCESS);

        // In order to be correctly testing the KFD interfaces and ensure
        // that the KFD acknowledges relevant node parameters
        // for the rest of the tests and used for more specific topology tests,
        // call to GetSystemProperties for a system snapshot of the topology here
        let ret = hsaKmtAcquireSystemProperties(&mut self.m_SystemProperties);
        assert_eq!(ret, HSAKMT_STATUS_SUCCESS);
    }
}

impl Default for KFDBaseComponentTest {
    fn default() -> Self {
        Self::new()
    }
}
