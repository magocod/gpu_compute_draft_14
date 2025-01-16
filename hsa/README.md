
test_global_array

```bash
/opt/rocm/bin/hipcc global_array.cpp --cuda-device-only -c -emit-llvm --offload-arch=gfx1032
```

```bash
/opt/rocm/llvm/bin/llvm-dis global_array-hip-amdgcn-amd-amdhsa-gfx1032.bc -o global_array-hip-amdgcn-amd-amdhsa_gfx1032.ll
```

```bash
/opt/rocm/llvm/bin/clang -target amdgcn-amd-amdhsa -mcpu=gfx1032 global_array-hip-amdgcn-amd-amdhsa_gfx1032.ll -o global_array-hip-amdgcn-amd-amdhsa_gfx1032.o
```