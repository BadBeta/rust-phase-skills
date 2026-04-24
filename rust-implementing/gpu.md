# GPU Programming in Rust (wgpu)

Graphics and compute on the GPU from Rust — buffers, textures, bind groups, pipelines, command submission, shader authoring (WGSL primary), error handling, testing, and the cross-platform story (native + WebGPU/wasm).

The principal Rust stack for GPU work is **wgpu** — the WebGPU-spec implementation used by Firefox, Deno, Bevy, and (via backends) most modern Rust graphics/compute code. This file focuses on wgpu patterns. For WASM-specific delivery, cross-reference [gui-wasm.md](gui-wasm.md).

## When to use the GPU

| Use case | GPU warranted? |
|---|---|
| Real-time rendering (games, visualizations, UI compositor) | Yes — that's what it's for |
| Large-batch numeric compute (linear algebra, convolutions, hashing N-million-entry datasets) | Yes, when parallelism is > ~10× the CPU's |
| Per-pixel image processing (filters, feature extraction, encoding assists) | Yes |
| ML inference at the edge (small models, custom ops) | Yes — wgpu is a viable backend if you don't want CUDA or Metal lock-in |
| A loop over a 10k-element array that runs a few times | **No** — GPU upload/download cost dwarfs compute savings |
| Scalar state machines, networking, business logic | **No** — wrong primitive |
| Single-shot operations where CPU SIMD would suffice | **No** — rayon + SIMD intrinsics or `std::simd` first |

**Rule:** the dispatch cost of a wgpu compute pass (upload buffer, submit, poll, download) is typically hundreds of microseconds to milliseconds. GPU work only wins when the compute per dispatch exceeds that fixed cost.

## The wgpu crate stack

wgpu is intentionally layered:

| Layer | Purpose | Unsafe? |
|---|---|---|
| `wgpu` | User-facing safe API | `#![warn(unsafe_op_in_unsafe_fn)]` — minimal, documented |
| `wgpu-core` | Resource management, validation, command submission orchestration | Small amount, all at well-known boundaries |
| `wgpu-hal` | Hardware abstraction — the actual FFI to Vulkan / Metal / DX12 / GL / WebGPU | All the unsafe lives here |
| `wgpu-types` | Shared types used by all layers | Safe |
| `naga` | Shader translator (WGSL ↔ SPIR-V ↔ GLSL ↔ MSL ↔ HLSL) | Safe (parser/codegen, no FFI) |

**Architectural lesson:** this is a canonical "concentrate unsafe in one layer" pattern (see [rust-planning/unsafe-strategy.md](../rust-planning/unsafe-strategy.md)). The user writes against `wgpu`, which delegates to `wgpu-core` (safe with known-small unsafe), which delegates to `wgpu-hal` (unsafe FFI). The top of the user stack stays safe.

## Backend selection via feature flags

wgpu supports many backends; you select via Cargo features on the `wgpu` crate:

```toml
[dependencies]
wgpu = { version = "22", default-features = false, features = [
    "vulkan",  # Linux, Windows, Android
    "metal",   # macOS, iOS
    "dx12",    # Windows
    "gles",    # Linux/Android fallback
    # "webgl", # Browser via WebGL2 (compile target wasm32)
    # "angle", # ANGLE GL-to-Vulkan/Metal/DX
    "wgsl",   # WGSL shader support
    # "noop",  # Headless stub backend for testing (see §Testing)
]}
```

For cross-platform binaries you typically enable `vulkan`, `metal`, `dx12`, and `gles` — wgpu picks one at runtime based on `Backends` and the available adapter. For wasm, enable `webgl` for the fallback path (WebGPU is otherwise picked up automatically).

## The minimum viable compute pipeline

Complete compute example: multiply a buffer of f32s by 2.0 on the GPU and read back. Uses `pollster::block_on` for sync-blocking.

```rust
use pollster::FutureExt as _;
use std::borrow::Cow;
use wgpu::util::DeviceExt;

fn main() -> anyhow::Result<()> {
    // 1. Instance — enumerate adapters
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    // 2. Adapter — a physical GPU
    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default())
        .block_on()
        .ok_or_else(|| anyhow::anyhow!("no adapter"))?;

    // 3. Device + Queue — logical GPU + command submission channel
    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
        .block_on()?;

    // 4. Input data on the GPU
    let input: Vec<f32> = (0..1024).map(|i| i as f32).collect();
    let storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("storage"),
        contents: bytemuck::cast_slice(&input),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });

    // 5. Staging buffer for readback (CPU can map MAP_READ, not STORAGE)
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: (input.len() * std::mem::size_of::<f32>()) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // 6. Shader (WGSL inline — real projects use include_str!("double.wgsl"))
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("double"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(r#"
            @group(0) @binding(0) var<storage, read_write> data: array<f32>;
            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) id: vec3<u32>) {
                if id.x < arrayLength(&data) {
                    data[id.x] = data[id.x] * 2.0;
                }
            }
        "#)),
    });

    // 7. Pipeline
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("double-pipeline"),
        layout: None,           // "auto" layout — wgpu derives from shader
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    // 8. Bind group — the data the pipeline operates on
    let bind_group_layout = pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("double-bg"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    // 9. Encode commands
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("encoder"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("double-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        let workgroups = (input.len() as u32).div_ceil(64);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0,
        (input.len() * std::mem::size_of::<f32>()) as u64);

    // 10. Submit to the GPU — returns immediately; GPU works in background
    queue.submit([encoder.finish()]);

    // 11. Read back — map_async + poll to drive it to completion
    let slice = staging_buffer.slice(..);
    let (tx, rx) = flume::bounded(1);
    slice.map_async(wgpu::MapMode::Read, move |r| { tx.send(r).unwrap(); });
    device.poll(wgpu::PollType::Wait)?;   // Drives the async op; blocks until GPU done
    rx.recv()??;
    let data = slice.get_mapped_range();
    let output: Vec<f32> = bytemuck::cast_slice(&data).to_vec();
    drop(data);
    staging_buffer.unmap();

    assert_eq!(output[10], 20.0);
    Ok(())
}
```

**Three things this shows that trip up newcomers:**

1. **Two buffers for readback.** `STORAGE` buffers cannot be `MAP_READ` (hardware limitation). You copy to a separate `MAP_READ | COPY_DST` staging buffer.
2. **`map_async` requires polling.** On native, calling `map_async` alone does nothing — you must call `device.poll(PollType::Wait)` to drive the GPU-side async op to completion. Forgetting this means the callback never fires.
3. **`queue.submit` returns immediately.** The GPU runs in parallel with your CPU code. To observe results, you must synchronize (via map_async + poll, or explicit fences / surface present).

## Async model

wgpu operations split cleanly into:

| Kind | Example | Blocking? |
|---|---|---|
| Record-time | `create_buffer`, `create_pipeline`, `encoder.begin_compute_pass`, `pass.dispatch_workgroups` | Instant, local CPU work |
| Submit | `queue.submit([encoder.finish()])` | Returns immediately — GPU starts async |
| Readback | `buffer.slice(..).map_async(MapMode::Read, callback)` + `device.poll(PollType::Wait)` | Callback fires when GPU signals done |
| Adapter/Device creation | `request_adapter`, `request_device` | Async futures; use `pollster::block_on` or your runtime |

**Runtime-agnostic.** wgpu does not bind you to tokio. Common choices:

- **`pollster::block_on`** — tiny (~200 lines) sync-blocking adapter. Standard choice for wgpu-only programs.
- **`tokio`** — when your app is otherwise async and you want `device.poll_async()` (or its equivalent in your wgpu version) integrated with your runtime.
- **Custom executor** — GUI apps driving their own event loop (Bevy, egui) poll wgpu synchronously from their frame loop.

For a render loop on a surface, the pattern is:
```rust
loop {
    window.request_redraw();
    // platform event loop → your render callback → encode → submit → surface.present()
    // No explicit poll needed; surface.present() implicitly synchronizes
}
```

For compute without presentation, you're responsible for calling `poll` or awaiting `poll_async`.

## Shader authoring

wgpu's native shader language is **WGSL** (WebGPU Shading Language) — a Rust-friendly, safety-oriented language that naga translates to SPIR-V / MSL / HLSL / GLSL per backend.

```wgsl
// double.wgsl
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x < arrayLength(&data) {
        data[id.x] = data[id.x] * 2.0;
    }
}
```

Alternatives:
- **SPIR-V** via `wgpu::ShaderSource::SpirV(...)` (requires `spirv` feature on wgpu-core) — useful if you already have a compile pipeline producing SPIR-V
- **GLSL** via `wgpu::ShaderSource::Glsl { shader, stage, defines }` (requires `glsl` feature) — useful for porting existing GLSL codebases
- **naga-ir** for programmatically-generated shaders
- **Third-party**: `rust-gpu` (Rust → SPIR-V compiler) for writing shaders in Rust itself

**Rule:** default to WGSL for new code. It's the spec-defined, best-tested path, and gives you consistent cross-platform behavior. Reach for SPIR-V/GLSL only when migrating existing shader code or producing shaders from an external toolchain.

## Error handling

wgpu uses a **validation scope** model distinct from idiomatic Rust `Result`:

```rust
device.push_error_scope(wgpu::ErrorFilter::Validation);

// ... encode commands that might fail validation ...

if let Some(err) = device.pop_error_scope().await {
    eprintln!("validation error: {err}");
}
```

The scope stack captures errors that occur while the scope is active. `ErrorFilter` variants: `OutOfMemory`, `Validation`, `Internal`.

Errors that happen outside any active scope fire the callback registered with `Device::on_uncaptured_error()` — typically defaulted to a panic or log in examples, but you set it to your own handler in production.

**Why this model (not `Result`):** WebGPU spec requires errors to be reportable even when operations complete asynchronously on the GPU driver's timeline. Synchronous `Result` returns can't represent all failure modes (e.g., GPU hang, driver-detected later). The scope model accumulates errors and lets you inspect at convenient points.

**Synchronous errors still exist** for creation-time failures:
- `RequestDeviceError` when `adapter.request_device` fails
- Panics from pipeline creation when bind group layouts are incompatible (in debug builds)

Pattern: wrap the wgpu entrypoints in your crate's typed error:
```rust
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum GpuInitError {
    #[error("no suitable adapter")]
    NoAdapter,
    #[error("device request failed: {0}")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("validation: {0}")]
    Validation(String),
}
```

## Unsafe discipline

In **your** crate that uses wgpu: almost always `#![forbid(unsafe_code)]` at the crate root. The wgpu API is safe — the unsafe is in `wgpu-hal` where it belongs.

You'd reach for unsafe in wgpu code only to:
- Create a Surface from a raw window handle on an exotic platform — `Instance::create_surface_unsafe` exists for this
- Interop with an externally-created Vulkan/Metal/DX12 device — rare; see `wgpu_hal::api::*::Device::from_raw` patterns
- `bytemuck::cast_slice` large buffers (safe if types are `Pod`/`Zeroable` — see [rust-planning/unsafe-strategy.md](../rust-planning/unsafe-strategy.md))

Every reach for unsafe at the wgpu layer warrants a `// SAFETY:` comment and justification.

## Testing

GPU tests are infamously painful — CI runners don't have GPUs, different drivers behave differently, validation layers must match. wgpu addresses this with multiple test strategies:

### `noop` backend (for resource-management logic)

```toml
[dependencies]
wgpu = { version = "22", features = ["noop"] }
```

```rust
let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::NOOP,
    noop: wgpu::NoopBackendOptions { enable: true },
    ..Default::default()
});
// Enumerates exactly one noop adapter; devices, buffers, bind groups, pipelines
// can all be created, BUT: compute passes and render passes execute no shaders.
```

**What the noop backend is good for:**
- Validating that your resource creation code is correct (layouts, usages, sizes)
- Testing lifetime/ownership logic around wgpu handles
- Smoke-testing build on CI without a GPU
- Testing your API surface without end-to-end execution

**What it isn't good for:**
- Verifying shader output (no execution)
- Performance regression
- Backend-specific bug reproduction

### Real-backend integration tests

For end-to-end tests, use wgpu's own test infrastructure pattern:
- `cargo-nextest` for parallel test execution (wgpu uses this)
- Feature-gated test runners per backend (`#[cfg(feature = "vulkan")]`)
- Self-hosted CI with GPU runners, or GitHub Actions with `lavapipe` (software Vulkan)
- `lavapipe` / `SwiftShader` as software renderers for running Vulkan-path tests without GPU

### Snapshot tests with `insta`

For rendering output that's stable across runs on a given backend:

```rust
#[test]
fn renders_triangle() {
    let bytes = render_to_buffer();
    insta::assert_binary_snapshot!("triangle.png", bytes);
}
```

Best paired with `lavapipe` for deterministic output on CI.

## Performance patterns

| Pattern | Why |
|---|---|
| Reuse `CommandEncoder` per frame if possible | Avoids per-frame allocation |
| Pre-create bind group layouts, reuse | Creating them at runtime is expensive |
| Batch similar draws (fewer state changes) | Pipeline switches + bind group swaps are driver-expensive |
| Use indirect draws / indirect dispatch | Lets GPU drive counts without CPU round-trip |
| Use `Device::create_pipeline_cache` (native-only) | Skips shader recompilation on startup |
| Upload with `queue.write_buffer` for small updates; `copy_buffer_to_buffer` for large | Smaller updates coalesce; large copies bypass staging |
| Aligned texture uploads (row pitch multiple of 256 bytes) | Hardware requirement on most backends |
| `MAP_READ` read-back lazily — only when needed, not per frame | Forces CPU-GPU sync |

## Debugging

| Tool | Platform | What it shows |
|---|---|---|
| **RenderDoc** | Linux/Windows (Vulkan/GL/DX12) | Frame capture, resource inspection, shader debugging, draw-call timeline |
| **Xcode Metal frame debugger** | macOS | Metal-equivalent of RenderDoc |
| **PIX** | Windows | Microsoft-first-party DX12 debugger |
| `wgpu` validation (default on debug) | All | Catches most API misuse early; `RUST_LOG=wgpu_core::device::resource=info` for more signal |
| Vulkan validation layers (`VK_LAYER_KHRONOS_validation`) | Vulkan backend | Driver-level validation beyond wgpu's own |
| `wgpu-profiler` | All | Per-pass GPU timing without leaving wgpu |
| Chrome DevTools WebGPU inspector | wasm build | Developer tools when shipping to browsers |

**Common failure modes to recognize:**

- "buffer usage mismatch" — forgot a `BufferUsages` flag (e.g., `COPY_SRC` on a buffer you later copy from)
- "bind group layout incompatible" — your bind group's layout doesn't match what the pipeline expects
- "binding X not set" — pipeline expected something at binding X but your bind group didn't provide it
- "surface configured with wrong format" — Surface format doesn't match what the render pipeline declares
- Silent no-op on native — forgot to `poll` after `map_async`, or forgot to `submit` the encoder
- Works on one backend, breaks on another — often WGSL → MSL/HLSL/SPIR-V translation edge cases (file a bug; naga ships fixes)

## Cross-platform: native vs WebGPU

wgpu runs natively (Linux/Windows/macOS/Android/iOS) AND in browsers (via WebGPU, with WebGL2 fallback).

| Concern | Native | Browser |
|---|---|---|
| Runtime | Your choice (pollster, tokio, Bevy's) | Browser event loop — use `wasm-bindgen-futures::spawn_local` |
| `device.poll()` | You call it | Browser drives it; usually not needed |
| Surface creation | From `winit`/`SDL`/native window handle | From `<canvas>` element |
| Shader languages | WGSL, SPIR-V, GLSL | WGSL only (browser doesn't ship SPIR-V parser) |
| Backend selection | Cargo features pick native backend | `webgpu` or `webgl` feature |
| Threading | Free — use `rayon` if helpful | Browser is mostly single-threaded; use Web Workers explicitly |
| Memory limits | GPU VRAM | Browser-capped (~256MB on many devices) |

**Rule:** target WGSL only for cross-platform code, and test on both a native backend and a WebGPU browser early. Don't find out WGSL translation broke on Metal after you've shipped.

## Ecosystem crates

| Crate | Purpose |
|---|---|
| **[wgpu](https://crates.io/crates/wgpu)** | The main user-facing crate |
| **[naga](https://crates.io/crates/naga)** | Shader compiler — usually transitively pulled in by wgpu |
| **[winit](https://crates.io/crates/winit)** | Cross-platform windowing; the typical pairing with wgpu |
| **[bytemuck](https://crates.io/crates/bytemuck)** | Pod/Zeroable for safe buffer/texture uploads ([rust-planning/unsafe-strategy.md](../rust-planning/unsafe-strategy.md)) |
| **[glam](https://crates.io/crates/glam)** | SIMD-accelerated math library (vec3, mat4, quat) — standard for GPU work |
| **[pollster](https://crates.io/crates/pollster)** | Minimal `block_on` for sync wgpu examples |
| **[wgpu-profiler](https://crates.io/crates/wgpu-profiler)** | GPU timing queries wrapped in an ergonomic API |
| **[Bevy](https://bevyengine.org)** | Game engine using wgpu for rendering |
| **[egui-wgpu](https://crates.io/crates/egui-wgpu)** | egui immediate-mode UI rendered through wgpu |
| **[iced_wgpu](https://crates.io/crates/iced_wgpu)** | iced Elm-architecture UI through wgpu |
| **[rend3](https://crates.io/crates/rend3)** | Higher-level "batteries-included" renderer on wgpu |
| **[rust-gpu](https://github.com/EmbarkStudios/rust-gpu)** | Write shaders in Rust, compile to SPIR-V — experimental alternative to WGSL |

## Common pitfalls (review checklist)

When reviewing wgpu code, scan for:

- [ ] Forgot `.poll()` after `map_async` on native → callback never fires
- [ ] Missed `BufferUsages` flag (commonly `COPY_SRC` / `COPY_DST`)
- [ ] `STORAGE` buffer used where `MAP_READ` needed — need a staging buffer
- [ ] Texture row pitch not 256-byte aligned on copy
- [ ] BindGroup layout doesn't match pipeline layout
- [ ] Shader expects binding X, bind group doesn't have it
- [ ] `unsafe { Instance::create_surface_unsafe(...) }` without SAFETY comment
- [ ] Using `wgpu::ShaderSource::SpirV` without the `spirv` feature (compile error, but easy to miss in multi-backend code)
- [ ] Creating the pipeline inside a hot loop instead of once at startup
- [ ] Hardcoded backend selection — missing `fallback_adapter` path
- [ ] No validation scope around failure-prone setup → error becomes uncaught panic
- [ ] Async tests without pumping `device.poll()` — test hangs
- [ ] Mixing `wgpu::Features` flags inconsistently across Device and pipeline creation
- [ ] `queue.submit([])` with no encoders — wasted call
- [ ] Holding a `BufferSlice::get_mapped_range()` across an `.await` — lifetime conflict

## Related

- [gui-wasm.md](gui-wasm.md) — egui, iced, Leptos/Yew, WASM delivery; pairs with wgpu for in-browser graphics
- [rust-planning/unsafe-strategy.md](../rust-planning/unsafe-strategy.md) — `#![forbid(unsafe_code)]` pattern that wgpu itself exemplifies; `bytemuck::Pod`/`Zeroable` for safe buffer transmutation
- [rust-planning/async-strategy.md](../rust-planning/async-strategy.md) — custom runtimes (wgpu is runtime-agnostic; GUI apps often have their own)
- [rust-planning/error-strategy.md](../rust-planning/error-strategy.md) — typed errors wrapping wgpu's failure modes
- [rust-reviewing/performance-catalog.md](../rust-reviewing/performance-catalog.md) — CPU-side hotspots; GPU ones mostly show up as "too little parallelism" or "too many pipeline changes" in a RenderDoc capture
- [c-programming](../c-programming/SKILL.md) — if you end up writing compute kernels in C/C++ via CUDA/HIP rather than wgpu

## References

- [wgpu website](https://wgpu.rs/) — official
- [wgpu crate docs](https://docs.rs/wgpu/) — API reference
- [Learn Wgpu](https://sotrh.github.io/learn-wgpu/) — tutorial-first; compute and graphics
- [wgpu repo](https://github.com/gfx-rs/wgpu) — examples, tests, the source of truth
- [WebGPU spec](https://www.w3.org/TR/webgpu/) — authoritative semantic reference
- [WGSL spec](https://www.w3.org/TR/WGSL/) — shader language reference
- [RenderDoc](https://renderdoc.org) — essential GPU debugging tool
