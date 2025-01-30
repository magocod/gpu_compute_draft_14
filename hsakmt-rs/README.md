# HSAKMT RUST

ROCt Thunk Library (`libhsakmt`) rewrite from C to Rust



questions
https://users.rust-lang.org/t/why-rust-has-no-tilde-operator-as-in-c-language/101882


other

bindgen ./include/linux/kfd_ioctl.h -o ./tmp/kfd_ioctl_bindings.rs

bindgen ./include/hsakmttypes.h -o ./tmp/hsakmttypes_bindings.rs