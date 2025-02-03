# HSAKMT RUST

ROCt Thunk Library (`libhsakmt`) rewrite from C to Rust



questions
https://users.rust-lang.org/t/why-rust-has-no-tilde-operator-as-in-c-language/101882

https://community.amd.com/t5/general-discussions/amd-kfd-api-documentation-where-to-find-it/td-p/564393


other

bindgen ./include/linux/kfd_ioctl.h -o ./tmp/kfd_ioctl_bindings.rs

bindgen ./include/hsakmttypes.h -o ./tmp/hsakmttypes_bindings.rs


warning

https://users.rust-lang.org/t/solved-how-to-move-non-send-between-threads-or-an-alternative/19928/4