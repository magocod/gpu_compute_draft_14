# numa_sys

Bindings generated with: https://github.com/rust-lang/rust-bindgen
```
bindgen /usr/include/numaif.h -o ./tmp/numa_if_bindings.rs
```

# Testing

test
```bash
cargo test -- --test-threads=1
```

Error if tests are run in parallel
```bash
cargo test
```
