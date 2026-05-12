# Rust WebAssembly Cheatsheet

> Quick reference for Rust WASM development (2025)

---

## Project Setup

```bash
# Create project
cargo new --lib my_app && cd my_app
rustup target add wasm32-unknown-unknown

# Install tools
cargo install trunk
cargo install wasm-bindgen-cli
```

### Cargo.toml

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
web-sys = { version = "0.3", features = ["console", "Window", "Document"] }
js-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde-wasm-bindgen = "0.6"
console_error_panic_hook = "0.1"
leptos = { version = "0.7", features = ["csr"] }  # Or yew/dioxus

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

### Trunk.toml

```toml
[build]
target = "index.html"

[[hooks]]
stage = "pre_build"
command = "sh"
command_arguments = ["-c", "npx tailwindcss -i input.css -o output.css"]
```

---

## Build Commands

```bash
# Development
trunk serve              # Dev server + hot reload
cargo test               # Native tests

# Production
trunk build --release    # Optimized build
wasm-opt -Oz dist/*.wasm -o dist/app.wasm  # Further optimize

# Testing
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Analysis
twiggy top app.wasm      # Binary size breakdown
cargo bloat --release --target wasm32-unknown-unknown
```

---

## wasm-bindgen Basics

### Exports

```rust
use wasm_bindgen::prelude::*;

// Function
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 { a + b }

// Struct + methods
#[wasm_bindgen]
pub struct Counter { value: i32 }

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Self { value: 0 } }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> i32 { self.value }

    pub fn increment(&mut self) { self.value += 1; }
}
```

### Imports

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;

    fn alert(s: &str);
}
```

### Types

| Rust | JS | Notes |
|------|-----|-------|
| `&str`, `String` | string | |
| `i32`, `u32`, `f64` | number | |
| `bool` | boolean | |
| `Vec<u8>` | Uint8Array | Copy |
| `&[u8]` | Uint8Array | View |
| `JsValue` | any | |
| `Option<T>` | T \| undefined | |
| `Result<T, E>` | T (throws) | |

---

## Async/Await

```rust
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub async fn fetch_data(url: String) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_str(&url)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    JsFuture::from(resp.json()?).await
}
```

---

## Leptos Quick Start

```rust
use leptos::*;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <button on:click=move |_| set_count.update(|c| *c += 1)>
            "Count: " {count}
        </button>
    }
}
```

---

## Phoenix LiveView Hook

```javascript
// hooks.js
const WasmHook = {
    async mounted() {
        const wasm = await import('./pkg/app.js');
        await wasm.default();
        this.component = wasm.Component.new(this.el);

        this.handleEvent("update", (data) => {
            this.component.update(data);
        });
    },
    destroyed() {
        this.component?.destroy();
    }
};
```

```elixir
# template.ex
<div id="wasm" phx-hook="WasmHook" phx-update="ignore"></div>
```

---

## Tailwind Setup

```javascript
// tailwind.config.js
module.exports = {
    content: ["./src/**/*.rs", "./index.html"],
    darkMode: 'class',
}
```

```rust
// In component
view! {
    <button class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">
        "Click"
    </button>
}
```

---

## Common Patterns

### Batch JS Calls

```rust
// BAD: Many boundary crossings
for item in items {
    js_process(item);
}

// GOOD: Single call
#[wasm_bindgen]
pub fn process_batch(items: &[u8]) -> Vec<u8> {
    items.iter().map(process).collect()
}
```

### Pre-allocate Buffers

```rust
thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(4096));
}
```

### Lazy Initialization

```rust
use once_cell::sync::Lazy;

static DATA: Lazy<ExpensiveData> = Lazy::new(|| {
    ExpensiveData::compute()
});
```

---

## Anti-Patterns

| Avoid | Do Instead |
|-------|------------|
| Many small JS calls | Batch operations |
| Allocate in hot loops | Pre-allocate buffers |
| `format!` in release | Simple panics or `expect` |
| Debug builds for perf | Always `--release` |
| Unbounded caches | Limit memory growth |
| Trust JS input | Validate at boundary |

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Binary size | < 500KB gzipped |
| Initial load | < 100ms |
| Frame budget | < 16ms |
| JS interop | < 1% of frame |

---

## Security Checklist

- [ ] `console_error_panic_hook` for debug
- [ ] Validate all JS inputs
- [ ] `overflow-checks = true` for security code
- [ ] `cargo audit` in CI
- [ ] CSP: `'wasm-unsafe-eval'`
- [ ] Minimize unsafe blocks

---

## Debugging

```rust
// Console log
web_sys::console::log_1(&"Debug".into());

// Panic hook
console_error_panic_hook::set_once();

// Performance timing
web_sys::console::time_with_label("op");
// ...code...
web_sys::console::time_end_with_label("op");
```

---

## Quick Links

- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/)
- [web-sys Docs](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
- [Leptos Book](https://leptos.dev/docs/)
- [Trunk Docs](https://trunkrs.dev/)
- [Tailwind Docs](https://tailwindcss.com/docs)

---

**Note:** wasm-pack is archived as of January 2025. Use `wasm-bindgen` CLI directly.
