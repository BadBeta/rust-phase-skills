# Rust WebAssembly Performance & Optimization

> Comprehensive guide to optimizing Rust WebAssembly applications for maximum performance.

## Table of Contents

1. [Performance Fundamentals](#performance-fundamentals)
2. [Binary Size Optimization](#binary-size-optimization)
3. [Runtime Performance](#runtime-performance)
4. [SIMD Optimization](#simd-optimization)
5. [Memory Optimization](#memory-optimization)
6. [Parallelism with Web Workers](#parallelism-with-web-workers)
7. [Profiling & Benchmarking](#profiling--benchmarking)
8. [wasm-opt Configuration](#wasm-opt-configuration)
9. [Framework-Specific Optimization](#framework-specific-optimization)
10. [Patterns & Anti-Patterns](#patterns--anti-patterns)
11. [Common Failures](#common-failures)
12. [Quick Reference](#quick-reference)

---

## Performance Fundamentals

### Understanding WASM Performance Characteristics

WebAssembly provides near-native performance but has unique characteristics:

```
┌─────────────────────────────────────────────────────────────┐
│                    Performance Factors                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Binary    │    │   Runtime   │    │  JS Interop │     │
│  │    Size     │    │  Execution  │    │   Overhead  │     │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘     │
│         │                  │                  │             │
│         ▼                  ▼                  ▼             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Download   │    │   CPU-bound │    │  Boundary   │     │
│  │    Time     │    │    Tasks    │    │  Crossings  │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### When WASM Outperforms JavaScript

| Use Case | WASM Advantage | Typical Speedup |
|----------|----------------|-----------------|
| Image processing | Predictable memory layout | 2-10x |
| Cryptography | No JIT warmup, SIMD | 3-20x |
| Physics simulation | Tight loops, no GC | 2-5x |
| Audio processing | Low-latency, deterministic | 2-8x |
| Parsing/compilation | Complex algorithms | 3-15x |
| Compression | Bitwise operations | 5-20x |

### The Performance Tradeoff Triangle

```
              Binary Size
                  ▲
                 /│\
                / │ \
               /  │  \
              /   │   \
             /    │    \
            /─────┼─────\
           /      │      \
          /       │       \
         ▼────────┴────────▼
    Runtime Speed      JS Interop Cost
```

---

## Binary Size Optimization

### Cargo.toml Release Profile

```toml
[profile.release]
# Optimization level
opt-level = "z"          # Optimize for size ("s" = optimize for speed)
lto = true               # Link-time optimization (fat or thin)
codegen-units = 1        # Single codegen unit for better optimization
panic = "abort"          # No unwinding code
strip = true             # Strip symbols (Rust 1.59+)

[profile.release.package."*"]
opt-level = "z"          # Also optimize dependencies

# Alternative: speed-focused profile
[profile.release-fast]
inherits = "release"
opt-level = 3
lto = "thin"             # Faster builds, slightly larger output
```

### Size Optimization Comparison

```rust
// Size impact of different features (approximate)

// std::fmt - adds ~20-50KB
println!("Debug: {}", value);  // AVOID in release

// Use this instead for production
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => { web_sys::console::log_1(&format!($($arg)*).into()) }
}
#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {}
}

// Panics with formatting - adds ~10-30KB each
panic!("Error: {} at {}", msg, location);  // Avoid

// Use simple panics or expect
panic!("critical error");
result.expect("operation failed");
```

### Dependency Auditing with cargo-bloat

```bash
# Install bloat analyzer
cargo install cargo-bloat

# Analyze what's taking space
cargo bloat --release --target wasm32-unknown-unknown -n 20

# Sample output analysis
# File  .text    Size  Crate Name
#  8.2%  15.7% 23.4KiB std   core::fmt::write
#  5.1%   9.8% 14.6KiB std   core::fmt::Formatter::pad
#  3.2%   6.1%  9.1KiB regex regex::exec::ProgramCache::new
```

### Feature Flag Optimization

```toml
# Cargo.toml - Disable default features aggressively
[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }

# Use wee_alloc for smaller allocator (saves ~10KB)
wee_alloc = { version = "0.4", optional = true }

[features]
default = ["wee_alloc"]
```

```rust
// lib.rs - Use wee_alloc
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
```

### Binary Analysis with twiggy

```bash
# Install twiggy
cargo install twiggy

# Analyze binary composition
twiggy top -n 20 target/wasm32-unknown-unknown/release/app.wasm

# Find what's keeping code alive
twiggy dominators target/wasm32-unknown-unknown/release/app.wasm

# Show garbage (unreachable code)
twiggy garbage target/wasm32-unknown-unknown/release/app.wasm

# Diff between two builds
twiggy diff old.wasm new.wasm
```

---

## Runtime Performance

### Hot Path Optimization

```rust
use wasm_bindgen::prelude::*;

// SLOW: Repeated allocations and JS calls
#[wasm_bindgen]
pub fn process_items_slow(items: &JsValue) -> Result<JsValue, JsValue> {
    let array: js_sys::Array = items.clone().into();
    let mut results = Vec::new();

    for i in 0..array.length() {
        let item = array.get(i);
        let processed = expensive_operation(&item)?;
        results.push(processed);
    }

    Ok(results.into_iter().collect::<js_sys::Array>().into())
}

// FAST: Batch processing with pre-allocated buffer
#[wasm_bindgen]
pub fn process_items_fast(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut result = Vec::with_capacity(len);

    // Process in cache-friendly chunks
    for chunk in data.chunks(64) {
        result.extend(process_chunk(chunk));
    }

    result
}

#[inline(always)]
fn process_chunk(chunk: &[u8]) -> impl Iterator<Item = u8> + '_ {
    chunk.iter().map(|&b| b.wrapping_mul(2))
}
```

### Avoiding Allocation in Hot Paths

```rust
use std::cell::RefCell;

// Thread-local buffer for reuse
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(4096));
}

#[wasm_bindgen]
pub fn transform_data(input: &[u8]) -> Vec<u8> {
    BUFFER.with(|buffer| {
        let mut buf = buffer.borrow_mut();
        buf.clear();

        // Reuse existing capacity
        buf.extend(input.iter().map(|&b| b ^ 0xFF));

        buf.clone()
    })
}

// For truly zero-allocation, use static buffers
static mut STATIC_BUFFER: [u8; 4096] = [0u8; 4096];

#[wasm_bindgen]
pub unsafe fn transform_in_place(len: usize) -> *const u8 {
    for i in 0..len.min(4096) {
        STATIC_BUFFER[i] ^= 0xFF;
    }
    STATIC_BUFFER.as_ptr()
}
```

### Loop Optimization

```rust
// SLOW: Bounds checking on every access
fn sum_slow(data: &[i32]) -> i64 {
    let mut sum: i64 = 0;
    for i in 0..data.len() {
        sum += data[i] as i64;  // Bounds check each time
    }
    sum
}

// FAST: Iterator eliminates bounds checks
fn sum_fast(data: &[i32]) -> i64 {
    data.iter().map(|&x| x as i64).sum()
}

// FASTER: Manual unrolling for predictable data
fn sum_unrolled(data: &[i32]) -> i64 {
    let mut sum: i64 = 0;
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    for chunk in chunks {
        sum += chunk[0] as i64;
        sum += chunk[1] as i64;
        sum += chunk[2] as i64;
        sum += chunk[3] as i64;
    }

    for &x in remainder {
        sum += x as i64;
    }

    sum
}
```

### Inlining Strategy

```rust
// Force inline for small, frequently called functions
#[inline(always)]
fn fast_abs(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

// Prevent inlining for large functions (reduces binary size)
#[inline(never)]
fn complex_initialization() -> Config {
    // Large initialization code
    Config::default()
}

// Let compiler decide (default)
#[inline]
fn medium_function(x: i32) -> i32 {
    x * x + 2 * x + 1
}
```

---

## SIMD Optimization

### Enabling SIMD (128-bit operations)

```toml
# .cargo/config.toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+simd128"]
```

### Using portable_simd (nightly)

```rust
#![feature(portable_simd)]
use std::simd::*;

#[wasm_bindgen]
pub fn sum_simd(data: &[f32]) -> f32 {
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();

    let mut sum = f32x4::splat(0.0);

    for chunk in chunks {
        let v = f32x4::from_slice(chunk);
        sum += v;
    }

    // Horizontal sum
    let mut result = sum.reduce_sum();

    for &x in remainder {
        result += x;
    }

    result
}

// Image processing with SIMD
#[wasm_bindgen]
pub fn brighten_image(pixels: &mut [u8], amount: u8) {
    let add = u8x16::splat(amount);

    for chunk in pixels.chunks_exact_mut(16) {
        let v = u8x16::from_slice(chunk);
        let brightened = v.saturating_add(add);
        chunk.copy_from_slice(&brightened.to_array());
    }
}
```

### SIMD with wide crate (stable)

```toml
[dependencies]
wide = "0.7"
```

```rust
use wide::*;

pub fn dot_product_simd(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());

    let mut sum = f32x4::ZERO;
    let chunks_a = a.chunks_exact(4);
    let chunks_b = b.chunks_exact(4);
    let remainder_a = chunks_a.remainder();
    let remainder_b = chunks_b.remainder();

    for (ca, cb) in chunks_a.zip(chunks_b) {
        let va = f32x4::from(ca);
        let vb = f32x4::from(cb);
        sum = va.mul_add(vb, sum);  // Fused multiply-add
    }

    let mut result: f32 = sum.reduce_add();

    for (&a, &b) in remainder_a.iter().zip(remainder_b) {
        result += a * b;
    }

    result
}
```

### SIMD Feature Detection

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = WebAssembly)]
    fn validate(bytes: &[u8]) -> bool;
}

// Runtime SIMD detection
pub fn has_simd_support() -> bool {
    // SIMD detection bytes (simplified)
    const SIMD_TEST: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d,  // WASM magic
        0x01, 0x00, 0x00, 0x00,  // Version
        // ... SIMD instruction bytes
    ];

    validate(SIMD_TEST)
}

// Conditional SIMD usage
#[wasm_bindgen]
pub fn process_optimized(data: &[f32]) -> f32 {
    if has_simd_support() {
        process_simd(data)
    } else {
        process_scalar(data)
    }
}
```

---

## Memory Optimization

### Memory Layout for Cache Efficiency

```rust
// BAD: Array of Structs (AoS) - poor cache locality
struct ParticleBad {
    position: [f32; 3],
    velocity: [f32; 3],
    color: [u8; 4],
    age: f32,
    // Padding issues, scattered access
}

// GOOD: Struct of Arrays (SoA) - better cache locality
struct ParticleSystem {
    positions_x: Vec<f32>,
    positions_y: Vec<f32>,
    positions_z: Vec<f32>,
    velocities_x: Vec<f32>,
    velocities_y: Vec<f32>,
    velocities_z: Vec<f32>,
    ages: Vec<f32>,
}

impl ParticleSystem {
    // Update positions - sequential memory access
    fn update_positions(&mut self, dt: f32) {
        for i in 0..self.positions_x.len() {
            self.positions_x[i] += self.velocities_x[i] * dt;
            self.positions_y[i] += self.velocities_y[i] * dt;
            self.positions_z[i] += self.velocities_z[i] * dt;
        }
    }
}
```

### Shared Memory with JavaScript

```rust
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

// Expose WASM memory for zero-copy access
#[wasm_bindgen]
pub fn get_memory() -> JsValue {
    wasm_bindgen::memory()
}

// Get pointer and length for direct JS access
#[wasm_bindgen]
pub struct ImageBuffer {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl ImageBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![0u8; width * height * 4],
        }
    }

    pub fn ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    // Process in WASM, read from JS
    pub fn invert(&mut self) {
        for pixel in self.data.chunks_exact_mut(4) {
            pixel[0] = 255 - pixel[0];
            pixel[1] = 255 - pixel[1];
            pixel[2] = 255 - pixel[2];
            // Alpha unchanged
        }
    }
}
```

```javascript
// JavaScript side - zero-copy access
const buffer = new ImageBuffer(1920, 1080);
const wasmMemory = get_memory();

// Create view into WASM memory
const view = new Uint8Array(
    wasmMemory.buffer,
    buffer.ptr(),
    buffer.len()
);

// Write directly to WASM memory
const imageData = ctx.getImageData(0, 0, 1920, 1080);
view.set(imageData.data);

// Process in WASM
buffer.invert();

// Read back (same view, memory already updated)
imageData.data.set(view);
ctx.putImageData(imageData, 0, 0);
```

### Memory Growth Strategy

```rust
// Pre-allocate to avoid growth during critical paths
#[wasm_bindgen(start)]
pub fn init() {
    // Pre-allocate 16MB
    let pages_needed = (16 * 1024 * 1024) / 65536;  // 64KB pages

    // Access memory to trigger growth
    let _preallocate: Vec<u8> = vec![0u8; pages_needed * 65536];

    // Memory stays allocated even after vec drops
}

// Monitor memory usage
#[wasm_bindgen]
pub fn memory_usage() -> usize {
    let memory = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap();

    let buffer = memory.buffer();
    buffer.byte_length() as usize
}
```

---

## Parallelism with Web Workers

### Web Worker Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Main Thread                             │
│  ┌─────────────┐                                            │
│  │   UI/DOM    │                                            │
│  │  Rendering  │                                            │
│  └──────┬──────┘                                            │
│         │ postMessage                                        │
├─────────┼───────────────────────────────────────────────────┤
│         ▼                                                    │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │  Worker 1   │    │  Worker 2   │    │  Worker N   │     │
│  │  (WASM)     │    │  (WASM)     │    │  (WASM)     │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                              │
│         SharedArrayBuffer (if available)                     │
└─────────────────────────────────────────────────────────────┘
```

### Worker-Ready WASM Module

```rust
// lib.rs - Must be thread-safe for workers
use wasm_bindgen::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

// Static counter shared across worker instances
static PROCESSED: AtomicUsize = AtomicUsize::new(0);

#[wasm_bindgen]
pub fn process_chunk(data: &[u8], chunk_id: usize) -> Vec<u8> {
    let result: Vec<u8> = data.iter()
        .map(|&b| complex_transform(b))
        .collect();

    PROCESSED.fetch_add(data.len(), Ordering::Relaxed);

    result
}

#[wasm_bindgen]
pub fn get_processed_count() -> usize {
    PROCESSED.load(Ordering::Relaxed)
}

fn complex_transform(b: u8) -> u8 {
    // CPU-intensive operation
    (0..100).fold(b, |acc, _| acc.wrapping_mul(7).wrapping_add(13))
}
```

### Worker Implementation (JavaScript)

```javascript
// worker.js
import init, { process_chunk } from './pkg/app.js';

let initialized = false;

self.onmessage = async (e) => {
    if (!initialized) {
        await init();
        initialized = true;
    }

    const { data, chunkId } = e.data;
    const result = process_chunk(new Uint8Array(data), chunkId);

    // Transfer ownership back (zero-copy)
    self.postMessage(
        { chunkId, result: result.buffer },
        [result.buffer]  // Transferable
    );
};

// main.js
class WorkerPool {
    constructor(size = navigator.hardwareConcurrency || 4) {
        this.workers = [];
        this.queue = [];
        this.results = new Map();

        for (let i = 0; i < size; i++) {
            const worker = new Worker('./worker.js', { type: 'module' });
            worker.onmessage = (e) => this.handleResult(e, i);
            this.workers.push({ worker, busy: false });
        }
    }

    async processInParallel(data, chunkSize = 1024 * 1024) {
        const chunks = [];
        for (let i = 0; i < data.length; i += chunkSize) {
            chunks.push(data.slice(i, i + chunkSize));
        }

        const promises = chunks.map((chunk, i) =>
            this.dispatch({ data: chunk.buffer, chunkId: i })
        );

        const results = await Promise.all(promises);
        return this.mergeResults(results);
    }

    dispatch(task) {
        return new Promise((resolve) => {
            const available = this.workers.find(w => !w.busy);
            if (available) {
                available.busy = true;
                available.worker.postMessage(task, [task.data]);
                this.results.set(task.chunkId, resolve);
            } else {
                this.queue.push({ task, resolve });
            }
        });
    }

    handleResult(e, workerIdx) {
        const { chunkId, result } = e.data;
        this.results.get(chunkId)(new Uint8Array(result));
        this.results.delete(chunkId);

        this.workers[workerIdx].busy = false;

        if (this.queue.length > 0) {
            const { task, resolve } = this.queue.shift();
            this.dispatch(task).then(resolve);
        }
    }

    mergeResults(results) {
        const totalLength = results.reduce((acc, r) => acc + r.length, 0);
        const merged = new Uint8Array(totalLength);
        let offset = 0;
        for (const result of results) {
            merged.set(result, offset);
            offset += result.length;
        }
        return merged;
    }
}

// Usage
const pool = new WorkerPool();
const largeData = new Uint8Array(100 * 1024 * 1024);  // 100MB
const result = await pool.processInParallel(largeData);
```

### SharedArrayBuffer for True Shared Memory

```javascript
// Requires COOP/COEP headers:
// Cross-Origin-Opener-Policy: same-origin
// Cross-Origin-Embedder-Policy: require-corp

// main.js
const sharedBuffer = new SharedArrayBuffer(1024 * 1024);
const sharedArray = new Int32Array(sharedBuffer);

// Share with workers
workers.forEach(worker => {
    worker.postMessage({ buffer: sharedBuffer });
});

// Atomic operations for synchronization
Atomics.store(sharedArray, 0, 1);  // Signal start
Atomics.notify(sharedArray, 0);    // Wake workers

// Wait for completion
Atomics.wait(sharedArray, 1, 0);   // Block until done
```

---

## Profiling & Benchmarking

### Browser DevTools Profiling

```rust
use web_sys::console;

// Manual timing
#[wasm_bindgen]
pub fn timed_operation(data: &[u8]) -> Vec<u8> {
    console::time_with_label("wasm_operation");

    let result = expensive_computation(data);

    console::time_end_with_label("wasm_operation");

    result
}

// Performance marks for timeline
#[wasm_bindgen]
pub fn profiled_operation(data: &[u8]) -> Vec<u8> {
    let window = web_sys::window().unwrap();
    let performance = window.performance().unwrap();

    performance.mark("wasm-start").unwrap();

    let result = expensive_computation(data);

    performance.mark("wasm-end").unwrap();
    performance.measure_with_start_mark_and_end_mark(
        "wasm-duration",
        "wasm-start",
        "wasm-end"
    ).unwrap();

    result
}
```

### Criterion.rs Benchmarking

```toml
[dev-dependencies]
criterion = { version = "0.5", default-features = false }

[[bench]]
name = "wasm_benchmarks"
harness = false
```

```rust
// benches/wasm_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_processing(c: &mut Criterion) {
    let data: Vec<u8> = (0..10000).map(|i| i as u8).collect();

    c.bench_function("scalar processing", |b| {
        b.iter(|| process_scalar(black_box(&data)))
    });

    c.bench_function("simd processing", |b| {
        b.iter(|| process_simd(black_box(&data)))
    });

    c.bench_function("parallel processing", |b| {
        b.iter(|| process_parallel(black_box(&data)))
    });
}

fn benchmark_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");

    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            format!("vec_alloc_{}", size),
            size,
            |b, &size| b.iter(|| Vec::<u8>::with_capacity(size))
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_processing, benchmark_allocations);
criterion_main!(benches);
```

### wasm-snip for Dead Code Removal

```bash
# Install wasm-snip
cargo install wasm-snip

# Remove specific functions
wasm-snip target/wasm32-unknown-unknown/release/app.wasm \
    -o optimized.wasm \
    --snip-rust-fmt-code \
    --snip-rust-panicking-code

# Before/after comparison
ls -la *.wasm
```

---

## wasm-opt Configuration

### Optimization Levels

```bash
# Install binaryen
# Arch: pacman -S binaryen
# macOS: brew install binaryen

# Optimization levels
wasm-opt -O1 input.wasm -o output.wasm   # Basic optimizations
wasm-opt -O2 input.wasm -o output.wasm   # More aggressive
wasm-opt -O3 input.wasm -o output.wasm   # Maximum speed
wasm-opt -Os input.wasm -o output.wasm   # Optimize for size
wasm-opt -Oz input.wasm -o output.wasm   # Aggressive size reduction
```

### Advanced wasm-opt Options

```bash
# Full optimization pipeline for production
wasm-opt -Oz \
    --enable-simd \
    --enable-bulk-memory \
    --enable-sign-ext \
    --enable-mutable-globals \
    --strip-debug \
    --strip-producers \
    --vacuum \
    --coalesce-locals \
    --reorder-functions \
    --duplicate-function-elimination \
    --inlining-optimizing \
    input.wasm -o optimized.wasm

# Size-focused preset
wasm-opt -Oz \
    --strip-debug \
    --strip-dwarf \
    --strip-producers \
    --vacuum \
    --remove-unused-brs \
    --remove-unused-names \
    --merge-blocks \
    input.wasm -o small.wasm
```

### Trunk Integration

```toml
# Trunk.toml
[build]
target = "index.html"

[[hooks]]
stage = "post_build"
command = "sh"
command_arguments = [
    "-c",
    "wasm-opt -Oz --enable-simd dist/*.wasm -o dist/optimized.wasm && mv dist/optimized.wasm dist/*.wasm"
]
```

---

## Framework-Specific Optimization

### Leptos Optimization

```rust
use leptos::*;

// SLOW: Recomputes on every signal change
fn slow_component() -> impl IntoView {
    let count = create_signal(0);
    let items = create_signal(vec![1, 2, 3, 4, 5]);

    view! {
        // This closure runs on every count change
        <ul>
            {move || items.get().iter().map(|i| {
                view! { <li>{i * count.get()}</li> }  // Expensive
            }).collect::<Vec<_>>()}
        </ul>
    }
}

// FAST: Memoized computation
fn fast_component() -> impl IntoView {
    let count = create_signal(0);
    let items = create_signal(vec![1, 2, 3, 4, 5]);

    // Memo only recalculates when items change
    let processed = create_memo(move |_| {
        items.get().iter().map(|i| i * 2).collect::<Vec<_>>()
    });

    view! {
        <For
            each=move || processed.get().into_iter().enumerate()
            key=|(i, _)| *i
            children=|(_, item)| view! { <li>{item}</li> }
        />
    }
}

// Keyed iteration for minimal DOM updates
fn keyed_list(items: ReadSignal<Vec<Item>>) -> impl IntoView {
    view! {
        <For
            each=move || items.get()
            key=|item| item.id  // Stable key
            children=|item| view! { <ItemView item=item/> }
        />
    }
}
```

### Yew Optimization

```rust
use yew::prelude::*;

// SLOW: Re-renders entire list
#[function_component]
fn SlowList(props: &ListProps) -> Html {
    html! {
        <ul>
            { for props.items.iter().map(|item| {
                html! { <li key={item.id}>{ &item.name }</li> }
            })}
        </ul>
    }
}

// FAST: Memoized child components
#[derive(Properties, PartialEq)]
struct ItemProps {
    item: Item,
}

#[function_component]
fn MemoizedItem(props: &ItemProps) -> Html {
    html! { <li>{ &props.item.name }</li> }
}

#[function_component]
fn FastList(props: &ListProps) -> Html {
    html! {
        <ul>
            { for props.items.iter().map(|item| {
                html! { <MemoizedItem key={item.id} item={item.clone()} /> }
            })}
        </ul>
    }
}

// Use callbacks efficiently
#[function_component]
fn OptimizedCallbacks() -> Html {
    let count = use_state(|| 0);

    // BAD: Creates new closure every render
    // let onclick = |_| count.set(*count + 1);

    // GOOD: Memoized callback
    let onclick = {
        let count = count.clone();
        Callback::from(move |_| count.set(*count + 1))
    };

    html! {
        <button {onclick}>{ *count }</button>
    }
}
```

---

## Patterns & Anti-Patterns

### Pattern 1: Batch JS Interop Calls

```rust
// PATTERN: Minimize boundary crossings
#[wasm_bindgen]
pub fn batch_update(updates: &[f32], count: usize) -> Vec<f32> {
    // Do all processing in WASM
    let mut results = Vec::with_capacity(count);
    for chunk in updates.chunks(3) {
        results.push(transform(chunk[0], chunk[1], chunk[2]));
    }
    // Single return to JS
    results
}
```

### Pattern 2: Pre-allocate Buffers

```rust
// PATTERN: Reuse memory allocations
struct Renderer {
    vertex_buffer: Vec<f32>,
    index_buffer: Vec<u32>,
}

impl Renderer {
    fn with_capacity(vertices: usize, indices: usize) -> Self {
        Self {
            vertex_buffer: Vec::with_capacity(vertices),
            index_buffer: Vec::with_capacity(indices),
        }
    }

    fn render(&mut self, scene: &Scene) {
        self.vertex_buffer.clear();  // Reuse capacity
        self.index_buffer.clear();
        // Fill buffers...
    }
}
```

### Pattern 3: SIMD with Scalar Fallback

```rust
// PATTERN: Feature detection with fallback
pub fn process_data(data: &mut [f32]) {
    #[cfg(target_feature = "simd128")]
    {
        process_simd(data);
    }

    #[cfg(not(target_feature = "simd128"))]
    {
        process_scalar(data);
    }
}
```

### Pattern 4: Chunked Processing for Large Data

```rust
// PATTERN: Process in manageable chunks
#[wasm_bindgen]
pub fn process_large_dataset(data: &[u8]) -> Vec<u8> {
    const CHUNK_SIZE: usize = 64 * 1024;  // 64KB chunks

    data.chunks(CHUNK_SIZE)
        .flat_map(|chunk| process_chunk(chunk))
        .collect()
}
```

### Pattern 5: Lazy Initialization

```rust
use once_cell::sync::Lazy;

// PATTERN: Defer expensive setup
static LOOKUP_TABLE: Lazy<[u8; 256]> = Lazy::new(|| {
    let mut table = [0u8; 256];
    for i in 0..256 {
        table[i] = compute_lookup(i as u8);
    }
    table
});

fn fast_lookup(input: u8) -> u8 {
    LOOKUP_TABLE[input as usize]
}
```

### Anti-Pattern 1: Excessive Console Logging

```rust
// ANTI-PATTERN: Logging in hot paths
pub fn process_pixels(pixels: &mut [u8]) {
    for (i, pixel) in pixels.iter_mut().enumerate() {
        web_sys::console::log_1(&format!("Processing pixel {}", i).into());  // TERRIBLE
        *pixel = transform(*pixel);
    }
}
```

### Anti-Pattern 2: String Concatenation in Loops

```rust
// ANTI-PATTERN: Repeated allocations
pub fn build_output(items: &[Item]) -> String {
    let mut result = String::new();
    for item in items {
        result = result + &item.to_string() + "\n";  // Allocates each iteration
    }
    result
}

// CORRECT
pub fn build_output_fast(items: &[Item]) -> String {
    let mut result = String::with_capacity(items.len() * 50);
    for item in items {
        result.push_str(&item.to_string());
        result.push('\n');
    }
    result
}
```

### Anti-Pattern 3: Unbounded Memory Growth

```rust
// ANTI-PATTERN: No capacity limits
static mut CACHE: Vec<Data> = Vec::new();

pub fn cache_result(data: Data) {
    unsafe {
        CACHE.push(data);  // Grows forever
    }
}
```

### Anti-Pattern 4: Blocking on Main Thread

```rust
// ANTI-PATTERN: Synchronous heavy computation
#[wasm_bindgen]
pub fn heavy_computation(data: &[u8]) -> Vec<u8> {
    // 5 seconds of CPU-bound work blocks UI
    data.iter().map(|&b| expensive_transform(b)).collect()
}

// CORRECT: Use Web Worker or chunked async
```

### Anti-Pattern 5: Unnecessary Cloning

```rust
// ANTI-PATTERN: Clone when not needed
pub fn process(data: Vec<u8>) -> Vec<u8> {
    let copy = data.clone();  // Unnecessary
    transform(copy)
}

// CORRECT: Take ownership or borrow
pub fn process_fast(data: Vec<u8>) -> Vec<u8> {
    transform(data)
}

pub fn process_borrowed(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| transform_byte(b)).collect()
}
```

---

## Common Failures

### Failure 1: JIT Deoptimization from Type Instability

```javascript
// FAILURE: WASM returns different types
const result = wasmModule.compute(input);
// Sometimes number, sometimes BigInt - causes JS deopt

// FIX: Consistent return types
#[wasm_bindgen]
pub fn compute(input: u32) -> u32 {  // Always u32
    // Never return i64/BigInt unexpectedly
    input.saturating_mul(2)
}
```

### Failure 2: Memory Pressure from Forgotten Cleanup

```rust
// FAILURE: Resources not freed
thread_local! {
    static HANDLERS: RefCell<Vec<Closure<dyn Fn()>>> = RefCell::new(Vec::new());
}

pub fn add_handler(f: impl Fn() + 'static) {
    HANDLERS.with(|h| {
        h.borrow_mut().push(Closure::new(f));  // Never cleared
    });
}

// FIX: Explicit cleanup
pub fn clear_handlers() {
    HANDLERS.with(|h| h.borrow_mut().clear());
}
```

### Failure 3: Slow Startup from Large Initialization

```rust
// FAILURE: Expensive init blocks page load
#[wasm_bindgen(start)]
pub fn main() {
    // 500ms of initialization work
    initialize_large_lookup_tables();
    precompute_all_values();
}

// FIX: Lazy initialization
static DATA: Lazy<LargeData> = Lazy::new(LargeData::compute);

pub fn get_data() -> &'static LargeData {
    &*DATA  // Only computed on first access
}
```

### Failure 4: Performance Cliff from Memory Growth

```rust
// FAILURE: Allocation causes memory growth during animation
pub fn animate_frame(canvas: &[u8]) -> Vec<u8> {
    vec![0u8; canvas.len()]  // May trigger growth
}

// FIX: Pre-allocate before critical path
thread_local! {
    static FRAME_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(4 * 1920 * 1080));
}
```

### Failure 5: Lost Performance from Debug Builds

```bash
# FAILURE: Testing performance with debug build
cargo build --target wasm32-unknown-unknown
# 10x slower than release, misleading benchmarks

# FIX: Always benchmark release
cargo build --release --target wasm32-unknown-unknown
wasm-opt -O3 target/wasm32-unknown-unknown/release/app.wasm -o app.wasm
```

---

## Quick Reference

### Binary Size Checklist

```
□ opt-level = "z" or "s" in Cargo.toml
□ lto = true
□ codegen-units = 1
□ panic = "abort"
□ strip = true
□ wee_alloc as global allocator
□ No std::fmt in release
□ Minimal dependencies
□ wasm-opt -Oz applied
□ twiggy analysis completed
```

### Performance Optimization Checklist

```
□ Batch JS interop calls
□ Pre-allocate buffers
□ Use iterators over index loops
□ Cache-friendly data layouts (SoA vs AoS)
□ SIMD where applicable
□ Web Workers for parallel processing
□ Lazy initialization for heavy setup
□ Profile with browser DevTools
□ Benchmark with criterion
□ Test with production build (--release)
```

### Profiling Commands

```bash
# Analyze binary
twiggy top app.wasm
cargo bloat --release --target wasm32-unknown-unknown

# Benchmark
cargo bench --target wasm32-unknown-unknown

# Optimize
wasm-opt -O3 --enable-simd app.wasm -o optimized.wasm
```

### Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Initial load | < 100ms | Performance.measure() |
| Frame budget | < 16ms | requestAnimationFrame |
| Binary size | < 500KB | gzip compressed |
| Memory growth | Predictable | Memory API |
| JS interop | < 1% of frame | DevTools profiler |

---

## Sources

- [WebAssembly Performance Best Practices](https://webassembly.org)
- [Rust WASM Book - Optimization](https://rustwasm.github.io/docs/book/reference/code-size.html)
- [wasm-bindgen Performance](https://rustwasm.github.io/wasm-bindgen/reference/types.html)
- [Binaryen wasm-opt](https://github.com/WebAssembly/binaryen)
- [twiggy Code Size Profiler](https://github.com/nickel-lang/nickel)
- [Chrome DevTools WASM Profiling](https://developer.chrome.com/docs/devtools/)
