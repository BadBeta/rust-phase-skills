# Testing Subskill

> Quick reference for WASM testing and debugging.

## When to Activate

Activate when user asks about:
- Testing WASM applications
- wasm-bindgen-test usage
- Browser testing for WASM
- Debugging WASM in browser
- Source maps for WASM
- CI/CD for WASM projects
- E2E testing with Playwright

## Full Reference

See `rust_wasm_testing.md` for complete documentation.

## Test Layers

```
         ▲ E2E Tests (Playwright)
        /│\
       / │ \ Integration Tests (Vitest)
      /──┴──\
     /       \ Unit Tests (cargo test + wasm-bindgen-test)
    ───────────
```

## Native Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic() {
        assert_eq!(add(2, 3), 5);
    }
}
```

```bash
cargo test
```

## WASM Tests

```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_in_browser() {
    let window = web_sys::window().unwrap();
    assert!(window.document().is_some());
}

#[wasm_bindgen_test]
async fn test_async() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

```bash
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
```

## Debugging

```rust
// Console logging
web_sys::console::log_1(&"Debug message".into());

// Panic hook for better errors
console_error_panic_hook::set_once();

// Performance timing
web_sys::console::time_with_label("operation");
// ... code ...
web_sys::console::time_end_with_label("operation");
```

## Source Maps

```toml
# Enable debug info
[profile.dev]
debug = true

[profile.release]
debug = 1  # Line tables only
```

```bash
# Build with debug info
wasm-bindgen --keep-debug ...
```

## CI/CD Commands

```yaml
# GitHub Actions
- name: Run native tests
  run: cargo test

- name: Run WASM tests
  run: wasm-pack test --headless --chrome

- name: Build release
  run: trunk build --release
```

## Key Patterns

1. **Test pure logic natively** - Faster iteration
2. **Use wasm-bindgen-test** for browser API tests
3. **Set panic hook** for readable error messages
4. **Enable source maps** in development
5. **Test multiple browsers** in CI
