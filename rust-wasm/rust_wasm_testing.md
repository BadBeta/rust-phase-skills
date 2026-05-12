# Rust WebAssembly Testing & Debugging

> Comprehensive guide to testing and debugging Rust WebAssembly applications.

## Table of Contents

1. [Testing Architecture](#testing-architecture)
2. [Unit Testing in Rust](#unit-testing-in-rust)
3. [wasm-bindgen-test](#wasm-bindgen-test)
4. [Browser Testing](#browser-testing)
5. [Integration Testing](#integration-testing)
6. [End-to-End Testing](#end-to-end-testing)
7. [Debugging Techniques](#debugging-techniques)
8. [Source Maps](#source-maps)
9. [Error Handling & Reporting](#error-handling--reporting)
10. [CI/CD Integration](#cicd-integration)
11. [Patterns & Anti-Patterns](#patterns--anti-patterns)
12. [Common Failures](#common-failures)
13. [Quick Reference](#quick-reference)

---

## Testing Architecture

### Testing Layers for WASM Applications

```
┌─────────────────────────────────────────────────────────────┐
│                    Testing Pyramid                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│                        ▲                                     │
│                       /│\                                    │
│                      / │ \    E2E Tests                      │
│                     /  │  \   (Playwright/Cypress)           │
│                    /───┼───\                                 │
│                   /    │    \                                │
│                  /     │     \  Integration Tests            │
│                 /      │      \ (Browser + WASM)             │
│                /───────┼───────\                             │
│               /        │        \                            │
│              /         │         \ Unit Tests                │
│             /          │          \(Pure Rust + wasm-bindgen)│
│            ─────────────────────────                         │
│                                                              │
│   Speed:   Fast ──────────────────────────────► Slow         │
│   Scope:   Narrow ────────────────────────────► Wide         │
│   Cost:    Cheap ─────────────────────────────► Expensive    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Test Environment Options

| Environment | Use Case | Speed | Browser APIs |
|-------------|----------|-------|--------------|
| Native Rust | Pure logic | Fastest | No |
| wasm-bindgen-test | WASM functions | Fast | Limited |
| Headless browser | DOM interaction | Medium | Full |
| Real browser | Visual testing | Slow | Full |

---

## Unit Testing in Rust

### Testing Pure Logic (No WASM Required)

```rust
// src/lib.rs
pub mod math {
    pub fn calculate_checksum(data: &[u8]) -> u32 {
        data.iter().fold(0u32, |acc, &b| {
            acc.wrapping_add(b as u32).wrapping_mul(31)
        })
    }

    pub fn validate_input(input: &str) -> Result<(), ValidationError> {
        if input.is_empty() {
            return Err(ValidationError::Empty);
        }
        if input.len() > 1000 {
            return Err(ValidationError::TooLong);
        }
        if !input.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidCharacters);
        }
        Ok(())
    }

    #[derive(Debug, PartialEq)]
    pub enum ValidationError {
        Empty,
        TooLong,
        InvalidCharacters,
    }
}

#[cfg(test)]
mod tests {
    use super::math::*;

    #[test]
    fn test_checksum_empty() {
        assert_eq!(calculate_checksum(&[]), 0);
    }

    #[test]
    fn test_checksum_deterministic() {
        let data = b"hello world";
        let result1 = calculate_checksum(data);
        let result2 = calculate_checksum(data);
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_checksum_different_inputs() {
        let result1 = calculate_checksum(b"abc");
        let result2 = calculate_checksum(b"abd");
        assert_ne!(result1, result2);
    }

    #[test]
    fn test_validation_success() {
        assert!(validate_input("valid_input_123").is_ok());
    }

    #[test]
    fn test_validation_empty() {
        assert_eq!(
            validate_input(""),
            Err(ValidationError::Empty)
        );
    }

    #[test]
    fn test_validation_special_chars() {
        assert_eq!(
            validate_input("invalid@input"),
            Err(ValidationError::InvalidCharacters)
        );
    }

    #[test]
    fn test_validation_too_long() {
        let long_input = "a".repeat(1001);
        assert_eq!(
            validate_input(&long_input),
            Err(ValidationError::TooLong)
        );
    }
}
```

### Running Native Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_checksum

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored

# Test with release optimizations (catches overflow bugs)
cargo test --release
```

### Property-Based Testing with proptest

```toml
[dev-dependencies]
proptest = "1.4"
```

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn checksum_never_panics(data: Vec<u8>) {
            // Should never panic regardless of input
            let _ = math::calculate_checksum(&data);
        }

        #[test]
        fn checksum_length_independent(
            data1 in prop::collection::vec(any::<u8>(), 0..100),
            data2 in prop::collection::vec(any::<u8>(), 0..100)
        ) {
            // Different data should (usually) produce different checksums
            if data1 != data2 {
                // Note: Collisions are possible, so we just check it doesn't panic
                let _ = math::calculate_checksum(&data1);
                let _ = math::calculate_checksum(&data2);
            }
        }

        #[test]
        fn validation_accepts_alphanumeric(s in "[a-zA-Z0-9_]{1,1000}") {
            prop_assert!(math::validate_input(&s).is_ok());
        }

        #[test]
        fn validation_rejects_special_chars(s in ".*[@#$%^&*].*") {
            prop_assert!(math::validate_input(&s).is_err());
        }
    }
}
```

---

## wasm-bindgen-test

### Setup

```toml
# Cargo.toml
[dev-dependencies]
wasm-bindgen-test = "0.3"

[lib]
crate-type = ["cdylib", "rlib"]  # rlib needed for tests
```

### Basic WASM Tests

```rust
// tests/wasm.rs
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;
use my_crate::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_exported_function() {
    let result = add_numbers(2, 3);
    assert_eq!(result, 5);
}

#[wasm_bindgen_test]
fn test_string_processing() {
    let result = process_text("hello");
    assert_eq!(result, "HELLO");
}

#[wasm_bindgen_test]
fn test_array_handling() {
    let input = vec![1, 2, 3, 4, 5];
    let result = sum_array(&input);
    assert_eq!(result, 15);
}
```

### Testing with Web APIs

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::{console, window};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_console_log() {
    // This should not panic
    console::log_1(&"Test message".into());
}

#[wasm_bindgen_test]
fn test_window_exists() {
    let win = window();
    assert!(win.is_some());
}

#[wasm_bindgen_test]
fn test_document_access() {
    let document = window()
        .expect("no window")
        .document()
        .expect("no document");

    let body = document.body().expect("no body");
    assert!(body.is_object());
}

#[wasm_bindgen_test]
fn test_dom_manipulation() {
    let document = window().unwrap().document().unwrap();

    // Create element
    let div = document.create_element("div").unwrap();
    div.set_id("test-element");
    div.set_inner_html("Test content");

    // Append to body
    document.body().unwrap().append_child(&div).unwrap();

    // Query and verify
    let found = document.get_element_by_id("test-element");
    assert!(found.is_some());
    assert_eq!(
        found.unwrap().inner_html(),
        "Test content"
    );
}
```

### Async WASM Tests

```rust
use wasm_bindgen_futures::JsFuture;
use js_sys::Promise;

#[wasm_bindgen_test]
async fn test_async_operation() {
    let result = async_compute(42).await;
    assert_eq!(result, 84);
}

#[wasm_bindgen_test]
async fn test_fetch_mock() {
    // Test with a mock or test server
    let window = window().unwrap();
    let response = JsFuture::from(
        window.fetch_with_str("data:text/plain,hello")
    ).await.unwrap();

    let response: web_sys::Response = response.dyn_into().unwrap();
    assert!(response.ok());
}

#[wasm_bindgen_test]
async fn test_timeout() {
    use gloo_timers::future::TimeoutFuture;

    let start = js_sys::Date::now();
    TimeoutFuture::new(100).await;
    let elapsed = js_sys::Date::now() - start;

    assert!(elapsed >= 100.0);
}
```

### Running WASM Tests

```bash
# Install test runner
cargo install wasm-bindgen-cli

# Run in headless Chrome
wasm-pack test --headless --chrome

# Run in headless Firefox
wasm-pack test --headless --firefox

# Run in Node.js (limited API support)
wasm-pack test --node

# Run specific test
wasm-pack test --headless --chrome -- --test wasm test_name
```

---

## Browser Testing

### Headless Browser Setup with wasm-pack

```toml
# Cargo.toml
[package.metadata.wasm-pack.profile.test]
# Test-specific settings
```

```bash
# Chrome
CHROME_PATH=/usr/bin/chromium wasm-pack test --headless --chrome

# Firefox
wasm-pack test --headless --firefox

# Safari (macOS)
wasm-pack test --headless --safari
```

### Custom Test Harness

```html
<!-- tests/index.html -->
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>WASM Tests</title>
</head>
<body>
    <div id="test-container"></div>
    <script type="module">
        import init, { run_tests } from './pkg/my_crate.js';

        async function main() {
            await init();

            const results = run_tests();
            console.log('Test results:', results);

            // Display results
            document.getElementById('test-container').innerHTML =
                `<pre>${JSON.stringify(results, null, 2)}</pre>`;
        }

        main().catch(console.error);
    </script>
</body>
</html>
```

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct TestResult {
    name: String,
    passed: bool,
    message: Option<String>,
}

#[wasm_bindgen]
pub fn run_tests() -> JsValue {
    let mut results = Vec::new();

    // Test 1
    results.push(TestResult {
        name: "addition".to_string(),
        passed: add(2, 2) == 4,
        message: None,
    });

    // Test 2
    let validation_result = validate_input("test");
    results.push(TestResult {
        name: "validation".to_string(),
        passed: validation_result.is_ok(),
        message: validation_result.err().map(|e| format!("{:?}", e)),
    });

    serde_wasm_bindgen::to_value(&results).unwrap()
}
```

---

## Integration Testing

### Testing WASM + JavaScript Integration

```javascript
// tests/integration.test.js
import { describe, it, expect, beforeAll } from 'vitest';
import init, { Calculator, process_data } from '../pkg/my_crate.js';

describe('WASM Integration', () => {
    beforeAll(async () => {
        await init();
    });

    describe('Calculator', () => {
        it('should add numbers correctly', () => {
            const calc = new Calculator();
            expect(calc.add(2, 3)).toBe(5);
        });

        it('should handle negative numbers', () => {
            const calc = new Calculator();
            expect(calc.add(-5, 3)).toBe(-2);
        });

        it('should handle overflow gracefully', () => {
            const calc = new Calculator();
            // Should not throw
            const result = calc.add(2147483647, 1);
            expect(typeof result).toBe('number');
        });
    });

    describe('Data Processing', () => {
        it('should process Uint8Array', () => {
            const input = new Uint8Array([1, 2, 3, 4, 5]);
            const result = process_data(input);
            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBe(5);
        });

        it('should handle empty input', () => {
            const input = new Uint8Array([]);
            const result = process_data(input);
            expect(result.length).toBe(0);
        });

        it('should handle large data', () => {
            const input = new Uint8Array(1024 * 1024);  // 1MB
            const start = performance.now();
            const result = process_data(input);
            const elapsed = performance.now() - start;

            expect(result.length).toBe(input.length);
            expect(elapsed).toBeLessThan(1000);  // Should complete in < 1s
        });
    });
});
```

### Testing LiveView Hooks with WASM

```javascript
// tests/liveview_hook.test.js
import { describe, it, expect, beforeEach, vi } from 'vitest';
import init, { WasmComponent } from '../pkg/my_crate.js';

// Mock LiveView hook context
function createMockHook() {
    return {
        el: document.createElement('div'),
        pushEvent: vi.fn(),
        pushEventTo: vi.fn(),
        handleEvent: vi.fn(),
        mounted() {},
        updated() {},
        destroyed() {},
    };
}

describe('LiveView WASM Hook', () => {
    let hook;
    let wasmComponent;

    beforeEach(async () => {
        await init();
        hook = createMockHook();
        wasmComponent = new WasmComponent(hook.el);
    });

    it('should initialize without errors', () => {
        expect(wasmComponent).toBeDefined();
    });

    it('should render to the element', () => {
        wasmComponent.render();
        expect(hook.el.innerHTML).not.toBe('');
    });

    it('should handle events from LiveView', () => {
        const eventData = { value: 42 };
        wasmComponent.handleEvent('update', eventData);
        expect(wasmComponent.getValue()).toBe(42);
    });

    it('should push events to LiveView', () => {
        wasmComponent.onPushEvent = (event, payload) => {
            hook.pushEvent(event, payload);
        };

        wasmComponent.triggerAction();

        expect(hook.pushEvent).toHaveBeenCalledWith(
            'wasm_action',
            expect.any(Object)
        );
    });

    it('should cleanup on destroy', () => {
        wasmComponent.render();
        wasmComponent.destroy();
        expect(hook.el.innerHTML).toBe('');
    });
});
```

### Vitest Configuration for WASM

```javascript
// vitest.config.js
import { defineConfig } from 'vitest/config';

export default defineConfig({
    test: {
        environment: 'jsdom',
        setupFiles: ['./tests/setup.js'],
        include: ['tests/**/*.test.js'],
        globals: true,
        deps: {
            inline: [/\.wasm$/],
        },
    },
});
```

```javascript
// tests/setup.js
import { beforeAll } from 'vitest';
import init from '../pkg/my_crate.js';

beforeAll(async () => {
    await init();
});
```

---

## End-to-End Testing

### Playwright Setup

```javascript
// playwright.config.js
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: 'html',
    use: {
        baseURL: 'http://localhost:4000',
        trace: 'on-first-retry',
    },
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],
    webServer: {
        command: 'mix phx.server',
        url: 'http://localhost:4000',
        reuseExistingServer: !process.env.CI,
    },
});
```

### E2E Tests for WASM Components

```javascript
// e2e/wasm_component.spec.js
import { test, expect } from '@playwright/test';

test.describe('WASM Image Editor', () => {
    test.beforeEach(async ({ page }) => {
        await page.goto('/editor');
        // Wait for WASM to load
        await page.waitForFunction(() => window.wasmReady === true);
    });

    test('should load WASM module', async ({ page }) => {
        const status = await page.locator('#wasm-status');
        await expect(status).toHaveText('Ready');
    });

    test('should process image', async ({ page }) => {
        // Upload image
        const fileInput = page.locator('input[type="file"]');
        await fileInput.setInputFiles('tests/fixtures/test-image.png');

        // Wait for processing
        await page.waitForSelector('#processed-image', { timeout: 10000 });

        // Verify result
        const canvas = page.locator('#processed-image');
        await expect(canvas).toBeVisible();

        // Check canvas has content
        const hasContent = await page.evaluate(() => {
            const canvas = document.getElementById('processed-image');
            const ctx = canvas.getContext('2d');
            const data = ctx.getImageData(0, 0, 1, 1).data;
            return data.some(v => v !== 0);
        });
        expect(hasContent).toBe(true);
    });

    test('should apply filter', async ({ page }) => {
        await page.locator('input[type="file"]').setInputFiles(
            'tests/fixtures/test-image.png'
        );
        await page.waitForSelector('#processed-image');

        // Apply grayscale filter
        await page.click('#filter-grayscale');

        // Wait for filter to apply
        await page.waitForTimeout(100);

        // Verify filter was applied (check WASM state)
        const filterApplied = await page.evaluate(() => {
            return window.wasmEditor.currentFilter === 'grayscale';
        });
        expect(filterApplied).toBe(true);
    });

    test('should maintain state across LiveView updates', async ({ page }) => {
        // Set some state in WASM
        await page.evaluate(() => {
            window.wasmEditor.setValue(42);
        });

        // Trigger LiveView update
        await page.click('#trigger-update');
        await page.waitForTimeout(500);

        // Verify WASM state preserved
        const value = await page.evaluate(() => {
            return window.wasmEditor.getValue();
        });
        expect(value).toBe(42);
    });
});

test.describe('WASM Performance', () => {
    test('should process large data within time limit', async ({ page }) => {
        await page.goto('/benchmark');
        await page.waitForFunction(() => window.wasmReady === true);

        // Run benchmark
        const result = await page.evaluate(async () => {
            const data = new Uint8Array(10 * 1024 * 1024);  // 10MB
            const start = performance.now();
            await window.wasmProcess(data);
            return performance.now() - start;
        });

        expect(result).toBeLessThan(5000);  // Should complete in < 5s
    });
});
```

---

## Debugging Techniques

### Console Debugging

```rust
use wasm_bindgen::prelude::*;
use web_sys::console;

// Debug logging macro
macro_rules! console_log {
    ($($arg:tt)*) => {
        console::log_1(&format!($($arg)*).into())
    };
}

macro_rules! console_warn {
    ($($arg:tt)*) => {
        console::warn_1(&format!($($arg)*).into())
    };
}

macro_rules! console_error {
    ($($arg:tt)*) => {
        console::error_1(&format!($($arg)*).into())
    };
}

// Debug helper for complex types
pub fn debug_value<T: std::fmt::Debug>(label: &str, value: &T) {
    console_log!("{}: {:?}", label, value);
}

// Conditional debug (stripped in release)
#[cfg(debug_assertions)]
macro_rules! debug_log {
    ($($arg:tt)*) => { console_log!($($arg)*) };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_log {
    ($($arg:tt)*) => {};
}

#[wasm_bindgen]
pub fn process_with_debug(input: &[u8]) -> Vec<u8> {
    debug_log!("Input length: {}", input.len());

    let result = input.iter()
        .map(|&b| {
            debug_log!("Processing byte: {}", b);
            b.wrapping_mul(2)
        })
        .collect();

    debug_log!("Processing complete");
    result
}
```

### Performance Debugging

```rust
use web_sys::{Performance, Window};

struct Timer {
    performance: Performance,
    label: String,
    start: f64,
}

impl Timer {
    fn new(label: &str) -> Self {
        let window: Window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();
        let start = performance.now();

        Self {
            performance,
            label: label.to_string(),
            start,
        }
    }

    fn lap(&self, checkpoint: &str) {
        let elapsed = self.performance.now() - self.start;
        console_log!("[{}] {}: {:.2}ms", self.label, checkpoint, elapsed);
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let elapsed = self.performance.now() - self.start;
        console_log!("[{}] Total: {:.2}ms", self.label, elapsed);
    }
}

#[wasm_bindgen]
pub fn debug_performance(data: &[u8]) -> Vec<u8> {
    let timer = Timer::new("process");

    // Phase 1
    let intermediate = phase1(data);
    timer.lap("phase1");

    // Phase 2
    let result = phase2(&intermediate);
    timer.lap("phase2");

    result
}
```

### Memory Debugging

```rust
#[wasm_bindgen]
pub fn debug_memory() -> JsValue {
    let memory = wasm_bindgen::memory()
        .dyn_into::<js_sys::WebAssembly::Memory>()
        .unwrap();

    let buffer = memory.buffer();
    let byte_length = buffer.byte_length();
    let pages = byte_length / 65536;  // 64KB pages

    let info = serde_json::json!({
        "total_bytes": byte_length,
        "total_kb": byte_length / 1024,
        "total_mb": byte_length / (1024 * 1024),
        "pages": pages
    });

    JsValue::from_str(&info.to_string())
}

// Track allocations
thread_local! {
    static ALLOCATION_COUNT: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

pub fn track_allocation<T>(f: impl FnOnce() -> T) -> T {
    let before = ALLOCATION_COUNT.with(|c| c.get());
    let result = f();
    let after = ALLOCATION_COUNT.with(|c| c.get());
    console_log!("Allocations: {}", after - before);
    result
}
```

---

## Source Maps

### Enabling Source Maps

```toml
# Cargo.toml
[profile.dev]
debug = true

[profile.release]
debug = 1  # Line tables only (smaller)
# debug = 2  # Full debug info
```

### Building with Source Maps

```bash
# Development build with full debug info
cargo build --target wasm32-unknown-unknown

# Use wasm-bindgen with debug info
wasm-bindgen \
    target/wasm32-unknown-unknown/debug/my_crate.wasm \
    --out-dir pkg \
    --target web \
    --keep-debug

# For Trunk
trunk build --features debug
```

### Chrome DevTools Debugging

```javascript
// Enable WASM debugging in Chrome
// 1. Open DevTools → Settings → Experiments
// 2. Enable "WebAssembly Debugging: Enable DWARF support"

// Set breakpoints in Rust source
// - Open Sources tab
// - Navigate to wasm file
// - If source maps work, you'll see .rs files
// - Set breakpoints on Rust lines
```

### DWARF Debug Info

```bash
# Install DWARF debugging extension
# Chrome: "C/C++ DevTools Support (DWARF)"

# Verify debug info present
wasm-objdump -h target/wasm32-unknown-unknown/debug/my_crate.wasm | grep -i debug

# Should show sections like:
# - .debug_info
# - .debug_line
# - .debug_abbrev
```

---

## Error Handling & Reporting

### Panic Handling

```rust
use std::panic;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Custom panic handler with more context
pub fn set_custom_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let message = info.to_string();
        let location = info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown".to_string());

        // Log to console
        web_sys::console::error_2(
            &format!("WASM Panic at {}", location).into(),
            &message.into()
        );

        // Optionally report to error tracking service
        #[cfg(feature = "error_reporting")]
        report_error(&message, &location);
    }));
}
```

### Result Handling for JS

```rust
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct WasmError {
    code: String,
    message: String,
    details: Option<String>,
}

impl From<&str> for WasmError {
    fn from(msg: &str) -> Self {
        WasmError {
            code: "UNKNOWN".to_string(),
            message: msg.to_string(),
            details: None,
        }
    }
}

// Return Result that JS can handle
#[wasm_bindgen]
pub fn safe_operation(input: &str) -> Result<JsValue, JsValue> {
    match parse_and_process(input) {
        Ok(result) => Ok(serde_wasm_bindgen::to_value(&result)?),
        Err(e) => {
            let error = WasmError {
                code: "PARSE_ERROR".to_string(),
                message: e.to_string(),
                details: Some(format!("Input: {}", input)),
            };
            Err(serde_wasm_bindgen::to_value(&error)?)
        }
    }
}
```

```javascript
// JavaScript error handling
try {
    const result = wasmModule.safe_operation(input);
    console.log('Success:', result);
} catch (error) {
    if (error.code) {
        // Structured error from Rust
        console.error(`[${error.code}] ${error.message}`);
        if (error.details) {
            console.error('Details:', error.details);
        }
    } else {
        // Unexpected error (panic, etc.)
        console.error('Unexpected WASM error:', error);
    }
}
```

---

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/wasm-tests.yml
name: WASM Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Install wasm-pack
        run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      # Native tests
      - name: Run native tests
        run: cargo test --all-features

      # WASM tests (headless)
      - name: Run WASM tests (Chrome)
        run: wasm-pack test --headless --chrome

      - name: Run WASM tests (Firefox)
        run: wasm-pack test --headless --firefox

      # Build WASM package
      - name: Build WASM
        run: wasm-pack build --release

      # Integration tests
      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Run integration tests
        run: npm test

      # E2E tests
      - name: Install Playwright browsers
        run: npx playwright install --with-deps

      - name: Run E2E tests
        run: npx playwright test

      - name: Upload test artifacts
        if: failure()
        uses: actions/upload-artifact@v4
        with:
          name: playwright-report
          path: playwright-report/

  lint:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          components: clippy, rustfmt

      - name: Format check
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy --target wasm32-unknown-unknown -- -D warnings

  size-check:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Build release
        run: cargo build --release --target wasm32-unknown-unknown

      - name: Install binaryen
        run: sudo apt-get install -y binaryen

      - name: Optimize WASM
        run: |
          wasm-opt -Oz \
            target/wasm32-unknown-unknown/release/*.wasm \
            -o optimized.wasm

      - name: Check size
        run: |
          SIZE=$(stat -f%z optimized.wasm 2>/dev/null || stat -c%s optimized.wasm)
          echo "WASM size: $SIZE bytes"
          if [ $SIZE -gt 500000 ]; then
            echo "Warning: WASM binary exceeds 500KB"
            exit 1
          fi
```

---

## Patterns & Anti-Patterns

### Pattern 1: Test Both Native and WASM

```rust
// Shared test logic
#[cfg(test)]
mod shared_tests {
    use super::*;

    pub fn test_checksum_impl<F: Fn(&[u8]) -> u32>(checksum: F) {
        assert_eq!(checksum(&[]), 0);
        assert_eq!(checksum(&[1, 2, 3]), checksum(&[1, 2, 3]));
        assert_ne!(checksum(&[1]), checksum(&[2]));
    }
}

// Native tests
#[cfg(test)]
mod native_tests {
    use super::*;

    #[test]
    fn test_checksum() {
        shared_tests::test_checksum_impl(calculate_checksum);
    }
}

// WASM tests
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_checksum_wasm() {
        shared_tests::test_checksum_impl(calculate_checksum);
    }
}
```

### Pattern 2: Mock Web APIs

```rust
// Trait for testable web interactions
trait Clock {
    fn now(&self) -> f64;
}

struct BrowserClock;
impl Clock for BrowserClock {
    fn now(&self) -> f64 {
        web_sys::window()
            .unwrap()
            .performance()
            .unwrap()
            .now()
    }
}

struct MockClock(f64);
impl Clock for MockClock {
    fn now(&self) -> f64 {
        self.0
    }
}

// Use trait in production code
struct Timer<C: Clock> {
    clock: C,
    start: f64,
}

impl<C: Clock> Timer<C> {
    fn new(clock: C) -> Self {
        let start = clock.now();
        Self { clock, start }
    }

    fn elapsed(&self) -> f64 {
        self.clock.now() - self.start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_elapsed() {
        let timer = Timer::new(MockClock(100.0));
        // In real test, mock would increment
        assert_eq!(timer.elapsed(), 0.0);
    }
}
```

### Pattern 3: Snapshot Testing

```rust
#[cfg(test)]
mod snapshot_tests {
    use insta::assert_json_snapshot;

    #[test]
    fn test_complex_output() {
        let result = complex_computation(&test_input());
        assert_json_snapshot!(result);
    }

    #[test]
    fn test_html_output() {
        let html = render_component(&props());
        insta::assert_snapshot!(html);
    }
}
```

### Pattern 4: Fuzz Testing

```rust
// fuzz/fuzz_targets/parse_input.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use my_crate::parse_input;

fuzz_target!(|data: &[u8]| {
    // Should never panic
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_input(s);
    }
});
```

```bash
# Run fuzzer
cargo +nightly fuzz run parse_input -- -max_len=1024
```

### Pattern 5: Test Fixtures

```rust
// tests/fixtures.rs
pub fn sample_image() -> Vec<u8> {
    include_bytes!("fixtures/test-image.png").to_vec()
}

pub fn sample_config() -> Config {
    serde_json::from_str(include_str!("fixtures/config.json")).unwrap()
}

// tests/image_tests.rs
use crate::fixtures;

#[test]
fn test_image_processing() {
    let image = fixtures::sample_image();
    let result = process_image(&image);
    assert!(result.is_ok());
}
```

### Anti-Pattern 1: Testing Implementation Details

```rust
// ANTI-PATTERN: Testing private internals
#[test]
fn test_internal_buffer_size() {
    let processor = Processor::new();
    // Testing internal state that may change
    assert_eq!(processor.internal_buffer.capacity(), 1024);
}

// CORRECT: Test observable behavior
#[test]
fn test_processor_handles_large_input() {
    let processor = Processor::new();
    let large_input = vec![0u8; 10000];
    let result = processor.process(&large_input);
    assert!(result.is_ok());
}
```

### Anti-Pattern 2: Non-Deterministic Tests

```rust
// ANTI-PATTERN: Random behavior without seed
#[test]
fn test_random_selection() {
    let result = select_random(&items);
    // May fail intermittently!
    assert!(items.contains(&result));
}

// CORRECT: Seed randomness
#[test]
fn test_random_selection_seeded() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    let result = select_random_with_rng(&items, &mut rng);
    assert_eq!(result, expected_for_seed_12345);
}
```

### Anti-Pattern 3: Test Pollution

```rust
// ANTI-PATTERN: Global state not cleaned up
static mut GLOBAL_STATE: i32 = 0;

#[test]
fn test_a() {
    unsafe { GLOBAL_STATE = 5; }
    assert_eq!(unsafe { GLOBAL_STATE }, 5);
}

#[test]
fn test_b() {
    // May fail if test_a runs first!
    assert_eq!(unsafe { GLOBAL_STATE }, 0);
}

// CORRECT: Use test fixtures with setup/teardown
struct TestContext {
    state: i32,
}

impl TestContext {
    fn new() -> Self {
        Self { state: 0 }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Cleanup
    }
}
```

### Anti-Pattern 4: Ignoring Async Complexity

```rust
// ANTI-PATTERN: Not testing async race conditions
#[wasm_bindgen_test]
async fn test_async_operation() {
    let result = async_op().await;
    assert!(result.is_ok());
    // Doesn't test concurrent access!
}

// BETTER: Test concurrent scenarios
#[wasm_bindgen_test]
async fn test_concurrent_operations() {
    let futures: Vec<_> = (0..10)
        .map(|i| async_op(i))
        .collect();

    let results = futures::future::join_all(futures).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

### Anti-Pattern 5: Over-Mocking

```rust
// ANTI-PATTERN: Mocking everything
#[test]
fn test_save_document() {
    let mock_storage = MockStorage::new();
    let mock_serializer = MockSerializer::new();
    let mock_validator = MockValidator::new();
    let mock_logger = MockLogger::new();

    // Test becomes meaningless - just testing mocks
    let service = Service::new(mock_storage, mock_serializer, mock_validator, mock_logger);
    service.save(&Document::default());

    mock_storage.verify();
}

// BETTER: Use real implementations where cheap
#[test]
fn test_save_document() {
    let storage = InMemoryStorage::new();  // Real, but in-memory
    let service = Service::new(storage);

    service.save(&test_document());

    assert!(storage.contains("doc-1"));
}
```

---

## Common Failures

### Failure 1: WASM Tests Pass, Production Fails

```rust
// FAILURE: Different behavior in tests vs production
#[wasm_bindgen_test]
fn test_works_in_test_env() {
    // Test environment has different timing
    let result = race_condition_prone_code();
    assert!(result.is_ok());  // Passes in tests, fails in prod
}

// FIX: Test with realistic conditions
#[wasm_bindgen_test]
async fn test_with_realistic_timing() {
    // Simulate production delays
    gloo_timers::future::TimeoutFuture::new(100).await;
    let result = race_condition_prone_code();
    assert!(result.is_ok());
}
```

### Failure 2: Memory Leaks Not Caught

```rust
// FAILURE: Leak not detected in short test
#[wasm_bindgen_test]
fn test_allocation() {
    let _ = create_resource();  // Leaked, but test passes
}

// FIX: Test for cleanup
#[wasm_bindgen_test]
fn test_allocation_cleanup() {
    let initial_memory = get_memory_usage();

    for _ in 0..100 {
        let resource = create_resource();
        drop(resource);  // Explicit cleanup
    }

    let final_memory = get_memory_usage();
    assert!(final_memory - initial_memory < 1024 * 1024);  // < 1MB growth
}
```

### Failure 3: Browser-Specific Bugs Missed

```bash
# FAILURE: Only testing Chrome
wasm-pack test --headless --chrome

# FIX: Test all target browsers
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
wasm-pack test --headless --safari  # macOS only
```

### Failure 4: Release Optimization Bugs

```rust
// FAILURE: Debug build works, release panics
#[cfg(debug_assertions)]
fn compute(x: i32) -> i32 {
    x.checked_mul(2).unwrap_or(0)
}

#[cfg(not(debug_assertions))]
fn compute(x: i32) -> i32 {
    x * 2  // Overflow in release!
}

// FIX: Same code path, test release builds
fn compute(x: i32) -> i32 {
    x.saturating_mul(2)
}

// Test with: cargo test --release
```

### Failure 5: Test Flakiness from Timing

```javascript
// FAILURE: Arbitrary timeout
test('wasm processes data', async () => {
    await wasmModule.process(data);
    await new Promise(r => setTimeout(r, 100));  // Hope this is enough
    expect(result).toBeDefined();
});

// FIX: Wait for actual condition
test('wasm processes data', async () => {
    await wasmModule.process(data);
    await expect(async () => {
        return wasmModule.getResult();
    }).toPass({ timeout: 5000 });
});
```

---

## Quick Reference

### Test Commands

```bash
# Native tests
cargo test
cargo test --release
cargo test -- --nocapture

# WASM tests
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox
wasm-pack test --node

# Integration tests
npm test
npx vitest

# E2E tests
npx playwright test
npx playwright test --headed
npx playwright test --debug

# Fuzz testing
cargo +nightly fuzz run target_name
```

### Debug Checklist

```
□ console_error_panic_hook enabled
□ Source maps generated (debug = true)
□ DWARF debugging extension installed
□ Performance marks added for profiling
□ Memory debugging helpers available
□ Error boundaries catching panics
```

### CI/CD Checklist

```
□ Native tests (cargo test)
□ WASM tests (wasm-pack test)
□ Multiple browsers tested
□ Integration tests
□ E2E tests
□ Size budget check
□ Clippy lints passing
□ Format check (cargo fmt)
```

### Testing Matrix

| Test Type | Native | WASM | Browser | Speed |
|-----------|--------|------|---------|-------|
| Unit | ✓ | ✓ | - | Fast |
| Integration | - | ✓ | ✓ | Medium |
| E2E | - | - | ✓ | Slow |
| Performance | ✓ | ✓ | ✓ | Varies |
| Fuzz | ✓ | - | - | Slow |

---

## Sources

- [wasm-bindgen-test Documentation](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/index.html)
- [wasm-pack Test Guide](https://rustwasm.github.io/wasm-pack/book/commands/test.html)
- [Playwright Testing](https://playwright.dev/)
- [Vitest](https://vitest.dev/)
- [Chrome DevTools WASM Debugging](https://developer.chrome.com/blog/wasm-debugging-2020/)
- [proptest Crate](https://docs.rs/proptest/)
- [insta Snapshot Testing](https://docs.rs/insta/)
