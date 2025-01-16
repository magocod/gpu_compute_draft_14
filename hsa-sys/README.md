

https://github.com/ROCm/ROCR-Runtime/tree/amd-staging

https://rocm.docs.amd.com/projects/ROCR-Runtime/en/master/index.html


https://hsafoundation.com/standards/


```
bindgen ./wrapper.h -o src/bindings.rs
```


help

https://rocm.docs.amd.com/en/latest/conceptual/More-about-how-ROCm-uses-PCIe-Atomics.html

https://rocm.docs.amd.com/projects/HIP/en/develop/how-to/debugging.html

https://github.com/ROCm/ROCm/issues/2705
https://github.com/ROCm/HIP/issues/90

https://github.com/ROCm/llvm-project/tree/amd-staging/amd/hipcc

# Testing

test
```bash
cargo test -- --test-threads=1
```

test
```bash
cargo test
```
