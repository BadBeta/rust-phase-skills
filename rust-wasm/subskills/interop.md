# JavaScript Interop Subskill

> Quick reference for wasm-bindgen, web-sys, and js-sys.

## When to Activate

Activate when user asks about:
- Calling JavaScript from Rust WASM
- Exposing Rust functions to JavaScript
- DOM manipulation from WASM
- Async/await with JavaScript Promises
- Closures and callbacks
- Type conversions between Rust and JS
- Memory management for interop

## Full Reference

See `rust_wasm_interop.md` for complete documentation.

## Essential Exports

```rust
use wasm_bindgen::prelude::*;

// Simple function export
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Struct with methods
#[wasm_bindgen]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> i32 {
        self.value
    }
}
```

## JavaScript Imports

```rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}
```

## Async/Promises

```rust
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub async fn fetch_json(url: String) -> Result<JsValue, JsValue> {
    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_str(&url)).await?;
    let resp: web_sys::Response = resp.dyn_into()?;
    JsFuture::from(resp.json()?).await
}
```

## Type Mapping

| Rust | JavaScript |
|------|------------|
| `&str`, `String` | string |
| `i32`, `u32`, `f64` | number |
| `bool` | boolean |
| `Vec<u8>` | Uint8Array |
| `JsValue` | any |
| `Option<T>` | T \| undefined |
| `Result<T, E>` | T (throws on Err) |

## Key Patterns

1. **Minimize boundary crossings** - batch operations
2. **Use typed arrays** for binary data (zero-copy possible)
3. **Handle closures carefully** - use `Closure::once` or `.forget()`
4. **Prefer `&str` over `String`** for input parameters
