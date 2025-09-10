# GXT Extism Plugin

**ATTENTION**: The gxt library needs a source of randomness to work and therefor has an indirect dependency on `getrandom`.
For this to work properly, the plugin needs to be compiled for wasi:

```bash
# add wasm32-wasip1 target
rustup target add wasm32-wasip1

# build for wasm32-wasip1
cargo build -p gxt-api-extism --target wasm32-wasip1
```
