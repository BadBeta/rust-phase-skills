---
name: rust-wasm
description: Rust WebAssembly development with Phoenix LiveView, Tailwind CSS, JavaScript interop, and modern web patterns. Use for Rust WASM projects, Leptos/Yew/Dioxus apps, and hybrid architectures.
---

# Rust WebAssembly Skill

> Master skill for Rust WebAssembly development with Phoenix LiveView, Tailwind CSS, JavaScript interop, and modern web patterns.

## Skill Overview

This skill provides comprehensive guidance for building WebAssembly applications using Rust, with special focus on integration with Phoenix LiveView, Tailwind CSS, and JavaScript ecosystems.

## When to Use This Skill

Use this skill when the user is working on:

- **Rust WASM Projects**: Compiling Rust to WebAssembly for browser/Node.js
- **Frontend Frameworks**: Leptos, Yew, Dioxus, Sycamore applications
- **Phoenix LiveView Integration**: WASM hooks, hybrid applications
- **JavaScript Interop**: wasm-bindgen, web-sys, js-sys usage
- **Performance-Critical Web Apps**: Image processing, games, crypto, simulations
- **Hybrid Architectures**: Combining server-rendered and client-side WASM

## Subskills

| Subskill | File | Use When |
|----------|------|----------|
| Core & Toolchain | `rust_wasm_core.md` | Project setup, build configuration, wasm-bindgen CLI |
| Frameworks | `rust_wasm_frameworks.md` | Leptos, Yew, Dioxus, Sycamore questions |
| JavaScript Interop | `rust_wasm_interop.md` | wasm-bindgen, web-sys, async, closures |
| LiveView Integration | `rust_wasm_liveview.md` | Phoenix hooks, phx-update="ignore", Orb |
| Security | `rust_wasm_security.md` | Memory safety, CSP, supply chain, validation |
| Performance | `rust_wasm_performance.md` | Optimization, SIMD, profiling, Web Workers |
| Testing | `rust_wasm_testing.md` | wasm-bindgen-test, E2E, debugging |
| Styling | `rust_wasm_styling.md` | Tailwind, CSS-in-Rust, theming |
| Extism plugins | `rust_wasm_extism.md` | Writing Rust plugins for the Extism runtime (any host language). `#[plugin_fn]`, FnResult, Json/Msgpack wrappers, host_fn imports, build targets, anti-patterns |

## Key Concepts

### WebAssembly in 2025

- **wasm-pack is archived** (January 2025) - Use `wasm-bindgen` CLI directly
- **Trunk** is the recommended build tool for WASM web apps
- **WASI** and **Component Model** are production-ready
- **SIMD (128-bit)** is widely supported in browsers

### Recommended Stack

```
┌─────────────────────────────────────────────────────┐
│                   Frontend Stack                     │
├─────────────────────────────────────────────────────┤
│                                                      │
│  Framework:  Leptos (recommended) / Yew / Dioxus    │
│  Build:      Trunk                                   │
│  Styling:    Tailwind CSS                           │
│  Interop:    wasm-bindgen + web-sys                 │
│                                                      │
├─────────────────────────────────────────────────────┤
│                   Backend Stack                      │
├─────────────────────────────────────────────────────┤
│                                                      │
│  Framework:  Phoenix LiveView                        │
│  Language:   Elixir                                  │
│  Integration: LiveView Hooks + phx-update="ignore"  │
│  Optional:   Orb (Elixir DSL for WASM)              │
│                                                      │
└─────────────────────────────────────────────────────┘
```

### Project Structure

```
my_wasm_app/
├── Cargo.toml              # Rust dependencies
├── Trunk.toml              # Build configuration
├── index.html              # Entry point
├── input.css               # Tailwind input
├── tailwind.config.js      # Tailwind configuration
├── src/
│   ├── lib.rs              # WASM exports
│   ├── app.rs              # Main component
│   └── components/         # UI components
├── pkg/                    # Generated WASM output
└── dist/                   # Production build
```

### Cargo.toml Template

```toml
[package]
name = "my_wasm_app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console", "Document", "Window"] }
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"

# Choose one framework:
leptos = { version = "0.7", features = ["csr"] }
# yew = { version = "0.21", features = ["csr"] }
# dioxus = { version = "0.5", features = ["web"] }

[dependencies.getrandom]
version = "0.2"
features = ["js"]

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

## Common Patterns

### 1. WASM Entry Point

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}
```

### 2. Phoenix LiveView Hook

```javascript
// JavaScript hook
const WasmHook = {
    async mounted() {
        const wasm = await import('./pkg/app.js');
        await wasm.default();
        this.component = wasm.Component.new(this.el);
    },
    updated() {
        // LiveView updated, but WASM manages phx-update="ignore" regions
    },
    destroyed() {
        this.component?.destroy();
    }
};

// Elixir template
<div id="wasm-mount" phx-hook="WasmHook" phx-update="ignore"></div>
```

### 3. Type-Safe JS Interop

```rust
#[wasm_bindgen]
pub struct DataProcessor {
    buffer: Vec<u8>,
}

#[wasm_bindgen]
impl DataProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn process(&mut self, data: &[u8]) -> Vec<u8> {
        data.iter().map(|b| b.wrapping_mul(2)).collect()
    }
}
```

### 4. Async Operations

```rust
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub async fn fetch_data(url: String) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let response = JsFuture::from(window.fetch_with_str(&url)).await?;
    let response: web_sys::Response = response.dyn_into()?;
    JsFuture::from(response.json()?).await
}
```

### 5. Tailwind with Leptos

```rust
#[component]
fn Button(
    #[prop(default = "primary")] variant: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = match variant {
        "primary" => "bg-blue-600 text-white hover:bg-blue-700",
        "secondary" => "bg-gray-200 text-gray-900 hover:bg-gray-300",
        _ => "",
    };

    view! {
        <button class=format!("px-4 py-2 rounded-md font-medium {}", classes)>
            {children()}
        </button>
    }
}
```

## Anti-Patterns to Avoid

1. **Excessive JS Boundary Crossings** - Batch operations, minimize interop calls
2. **Blocking Main Thread** - Use Web Workers for heavy computation
3. **Unbounded Memory Growth** - Pre-allocate, clean up resources
4. **Debug Builds in Production** - Always use `--release` and `wasm-opt`
5. **Ignoring WASM Security Model** - Validate all inputs from JS
6. **String Formatting in Loops** - Avoid allocations in hot paths

## Quick Commands

```bash
# Development
trunk serve                    # Start dev server with hot reload
cargo test                     # Run native tests
wasm-pack test --headless --chrome  # Run WASM tests

# Production
trunk build --release          # Build optimized WASM
wasm-opt -Oz app.wasm -o app.wasm  # Further optimize

# Analysis
twiggy top app.wasm            # Analyze binary size
cargo bloat --release --target wasm32-unknown-unknown  # Find bloat
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `wasm-pack` errors | Use `wasm-bindgen` CLI directly (wasm-pack archived) |
| Large binary size | Enable LTO, opt-level="z", strip, wasm-opt -Oz |
| JS interop type errors | Check wasm-bindgen version compatibility |
| Memory leaks | Properly clean up Closures, drop resources |
| Slow startup | Lazy initialization, streaming instantiation |
| Tailwind classes missing | Add `.rs` files to tailwind.config.js content |

## Version Information

- **Rust**: 1.75+ (stable WASM target)
- **wasm-bindgen**: 0.2.x
- **Leptos**: 0.7.x
- **Yew**: 0.21.x
- **Dioxus**: 0.5.x
- **Trunk**: 0.21.x
- **Tailwind CSS**: 4.x

## Related Skills

| Skill | Use When |
|-------|----------|
| `rust-programming` (webassembly subskill) | Server-side WASM (Wasmtime/Wasmer), distributed WASM execution, C ABI memory patterns, WASI |
| `phoenix-liveview` | Phoenix LiveView patterns beyond WASM integration |
| `tailwind` | Tailwind CSS styling (general) |
| `rust-nif` | Rust NIFs for Elixir (server-side native performance) |
| `elixir` | Elixir language and OTP |

**This skill focuses on:** Browser WASM, Rust frontend frameworks (Leptos/Yew/Dioxus), Phoenix LiveView integration, and modern web patterns.

**For server-side WASM:** See `rust-programming` skill's `webassembly` subskill for Wasmtime/Wasmer runtimes, WASI, distributed WASM execution with coroutines, and low-level memory management patterns.
