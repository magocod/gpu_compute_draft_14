#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::useless_transmute)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::ptr_offset_with_cast)]

pub mod bindings;

// #[cfg(test)]
// mod tests {
//     use crate::bindings::{hsaKmtCloseKFD, hsaKmtOpenKFD, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS};
//
//     #[test]
//     fn test_example() {
//         unsafe {
//             let ret = hsaKmtOpenKFD();
//             assert_eq!(ret, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS);
//
//             println!("ret = {}", ret);
//
//             //  ..
//
//             let ret = hsaKmtCloseKFD();
//             assert_eq!(ret, _HSAKMT_STATUS_HSAKMT_STATUS_SUCCESS);
//         }
//     }
// }
