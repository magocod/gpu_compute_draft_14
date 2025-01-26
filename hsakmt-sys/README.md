
https://github.com/ROCm/ROCR-Runtime/tree/amd-staging

https://rocm.docs.amd.com/projects/ROCR-Runtime/en/master/index.html

bindgen /opt/rocm/include/hsakmt/hsakmt.h -o ./tmp/bindings.rs


```
bindgen ./wrapper.h -o ./tmp/bindings.rs
```