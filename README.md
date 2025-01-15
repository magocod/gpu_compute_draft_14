# GPU COMPUTE DRAFT

(Important) All these examples of programs with GPU have only been tested in:

OS
* Ubuntu Ubuntu 22.04.5 LTS 64-bit

GPU
* Radeon rx 6500xt
* Radeon rx 6600

CPU
* ryzen 5700G

ROCM
* 6.2.2.60202-116~22.04


The main goal of this project is to create a draft of what it would be like to have data structures controlled and managed 100% on the GPU.

Inspired by the following libraries:
* https://github.com/stotko/stdgpu
* https://github.com/NVIDIA/thrust
* https://github.com/nvidia/cccl


thanks.
* https://github.com/Umio-Yasuno/amdgpu_top
* https://github.com/kenba/opencl3

---

# Vision

* https://github.com/crossbeam-rs/crossbeam
* https://docs.rust-embedded.org/discovery/microbit/index.html
* https://github.com/prsyahmi/GpuRamDrive.git

---

# Opencl crates

---

## opencl-sys

Rust bindings - OpenCL C 2.0 (ROCM CL headers)

## opencl

TODO explain

## opencl-examples

TODO explain

## opencl-collections

Implementations that seek to provide containers the same (similar) to those provided by rust std and c++
* https://doc.rust-lang.org/std/collections/
* https://cplusplus.com/reference/stl/

* https://github.com/stotko/stdgpu
* https://github.com/NVIDIA/cuCollections

## opencl-collections-examples

Examples of use of collections (opencl-collections) and comparisons with existing libraries in rust

## opencl-node

TODO explain

* https://github.com/napi-rs/napi-rs
* https://github.com/neon-bindings/neon

---

# ROCM crates

---

## hip-sys

TODO explain

## hip

TODO explain

## hsa-sys

TODO explain

## hsa

TODO explain

## amdsmi-sys

Rust bindings of AMD System Management Interface (AMD SMI) Library (https://github.com/ROCm/amdsmi)

## amdsmi

Rust wrapper of AMD System Management Interface (AMD SMI) Library (https://github.com/ROCm/amdsmi)

---

# Other crates

---

## gpu-fs

TODO explain

## gpu-ipc

TODO explain

* https://dbus.freedesktop.org/doc/dbus-specification.html
* https://github.com/dbus2/zbus
* https://github.com/diwic/dbus-rs

---

# Docs

```bash
mdbook serve gpu-compute-book
```

---

# Testing

test
```bash
cargo test -- --test-threads=1
```

// texts

All the execution control of the device has to be done at host side. The device has to care only about completing the queued jobs, as fast as possible.
That's all. If you intent to slow down the execution then do it at host side.


https://github.com/tracel-ai/cubecl-hip

https://github.com/tracel-ai/cubecl

https://github.com/ROCm/ROCm/issues/419
apt show rocm-libs -a

