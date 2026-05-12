# Performance Subskill

> Quick reference for WASM optimization and profiling.

## When to Activate

Activate when user asks about:
- WASM binary size reduction
- Runtime performance optimization
- SIMD in WebAssembly
- Memory optimization
- Web Workers with WASM
- Profiling WASM applications
- wasm-opt configuration

## Full Reference

See `rust_wasm_performance.md` for complete documentation.

## Binary Size Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = "z"      # Size optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Remove unwinding
strip = true         # Strip symbols
```

```bash
# Post-build optimization
wasm-opt -Oz --enable-simd input.wasm -o output.wasm
```

## Runtime Performance

```rust
// Batch operations to minimize JS boundary crossings
#[wasm_bindgen]
pub fn process_batch(data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b.wrapping_mul(2)).collect()
}

// Pre-allocate buffers
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(4096));
}
```

## SIMD (128-bit)

```toml
# .cargo/config.toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+simd128"]
```

```rust
#![feature(portable_simd)]
use std::simd::*;

fn sum_simd(data: &[f32]) -> f32 {
    data.chunks_exact(4)
        .map(|c| f32x4::from_slice(c))
        .fold(f32x4::splat(0.0), |a, b| a + b)
        .reduce_sum()
}
```

## Profiling Commands

```bash
# Binary analysis
twiggy top app.wasm
cargo bloat --release --target wasm32-unknown-unknown

# Size breakdown
twiggy dominators app.wasm
```

## Key Patterns

1. **Batch JS interop** - One call with array > many calls
2. **Pre-allocate buffers** - Avoid allocation in hot paths
3. **Use iterators** - Eliminates bounds checks
4. **SIMD for parallel data** - 4x speedup for compatible operations
5. **Web Workers** - Offload heavy computation from main thread

## Performance Targets

| Metric | Target |
|--------|--------|
| Binary size | < 500KB gzipped |
| Initial load | < 100ms |
| Frame budget | < 16ms |
| JS interop | < 1% of frame |
