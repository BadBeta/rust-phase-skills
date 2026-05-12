# Rust WASM Core Subskill

> Quick reference for Rust WebAssembly toolchain and project setup.

## When to Activate

Activate when user asks about:
- Setting up a new Rust WASM project
- Cargo.toml configuration for WASM
- wasm-bindgen CLI usage
- Trunk build tool
- Binary size optimization
- wasm-opt configuration
- WASI or Component Model

## Full Reference

See `rust_wasm_core.md` for complete documentation.

## Essential Commands

```bash
# Project setup
cargo new --lib my_wasm_app
rustup target add wasm32-unknown-unknown

# Build with wasm-bindgen
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/app.wasm \
    --out-dir pkg --target web

# Build with Trunk
trunk serve          # Development
trunk build --release  # Production

# Optimize
wasm-opt -Oz input.wasm -o output.wasm
```

## Minimal Cargo.toml

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
```

## Key Patterns

1. **Always use `--release`** for production
2. **Enable LTO and single codegen unit** for smaller binaries
3. **Use `panic = "abort"`** to remove unwinding code
4. **Apply wasm-opt** for final optimization

## Critical Note

**wasm-pack is archived as of January 2025.** Use `wasm-bindgen` CLI directly or Trunk.
