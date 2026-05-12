# Rust WebAssembly Security Practices

> **Version**: 2025
> **Status**: Complete Reference

## Table of Contents
1. [WebAssembly Security Model](#1-webassembly-security-model)
2. [Memory Safety in WASM](#2-memory-safety-in-wasm)
3. [Rust-Specific Security](#3-rust-specific-security)
4. [Input Validation](#4-input-validation)
5. [Cryptographic Considerations](#5-cryptographic-considerations)
6. [Supply Chain Security](#6-supply-chain-security)
7. [Content Security Policy](#7-content-security-policy)
8. [Common Vulnerabilities](#8-common-vulnerabilities)
9. [Security Tools](#9-security-tools)
10. [Patterns](#10-patterns)
11. [Anti-Patterns](#11-anti-patterns)
12. [Common Failures & Solutions](#12-common-failures--solutions)
13. [Security Checklist](#13-security-checklist)

---

## 1. WebAssembly Security Model

### 1.1 Sandbox Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Browser Process                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    JavaScript Engine                      │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │              WASM Sandbox (per module)               │ │   │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │ │   │
│  │  │  │   Linear    │  │   Tables    │  │   Globals   │ │ │   │
│  │  │  │   Memory    │  │  (indirect  │  │             │ │ │   │
│  │  │  │             │  │   calls)    │  │             │ │ │   │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘ │ │   │
│  │  │                                                      │ │   │
│  │  │  ┌─────────────────────────────────────────────────┐│ │   │
│  │  │  │         Protected Call Stack (shadow)           ││ │   │
│  │  │  └─────────────────────────────────────────────────┘│ │   │
│  │  └──────────────────────────────────────────────────────┘ │   │
│  └───────────────────────────────────────────────────────────┘   │
│                              │                                    │
│                              ▼                                    │
│               ┌──────────────────────────────┐                   │
│               │         Web APIs             │                   │
│               │  (DOM, fetch, etc. via JS)   │                   │
│               └──────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Built-in Security Features

| Feature | Protection Provided |
|---------|---------------------|
| **Sandboxed Execution** | Module isolated from host system |
| **Memory Isolation** | Cannot access browser internals |
| **Bounds Checking** | Memory access validated at runtime |
| **Protected Call Stack** | Separate from linear memory |
| **No Direct Syscalls** | Must go through imports |
| **Type Safety** | Validated at load time |
| **Same-Origin Policy** | Inherits from embedding |

### 1.3 What WASM Cannot Do (Without Imports)

- Access the DOM
- Make network requests
- Read/write files
- Access other browser tabs
- Execute arbitrary JavaScript
- Access hardware directly

### 1.4 Security Guarantees vs Limitations

**Guarantees:**
- Control flow integrity (no arbitrary jumps)
- Type-safe function calls
- Memory bounds enforcement
- Isolated linear memory

**Limitations:**
- No ASLR (addresses are deterministic)
- No stack canaries by default
- No W^X (memory is both readable and writable)
- No read-only memory sections
- Bugs within sandbox can still corrupt sandbox memory

---

## 2. Memory Safety in WASM

### 2.1 Linear Memory Vulnerabilities

Unlike native code, WASM memory corruption is contained within the sandbox. However, vulnerabilities within the sandbox can still be severe:

```
Linear Memory Layout (typical):
┌────────────────────────────────────────┐  High address
│              Heap (grows up)           │
├────────────────────────────────────────┤
│                   ...                  │
├────────────────────────────────────────┤
│           Stack (grows down)           │
├────────────────────────────────────────┤
│          Static Data / Strings         │
├────────────────────────────────────────┤
│              Globals                   │
└────────────────────────────────────────┘  Address 0
```

### 2.2 Buffer Overflow in WASM

**The Problem:**
```rust
// UNSAFE: Buffer overflow possible
#[wasm_bindgen]
pub fn copy_data(src: &[u8], dest_ptr: usize, dest_len: usize) {
    // If src.len() > dest_len, this overflows!
    unsafe {
        let dest = std::slice::from_raw_parts_mut(dest_ptr as *mut u8, src.len());
        dest.copy_from_slice(src);
    }
}
```

**Safe Alternative:**
```rust
#[wasm_bindgen]
pub fn copy_data_safe(src: &[u8], dest_ptr: usize, dest_len: usize) -> Result<(), JsError> {
    if src.len() > dest_len {
        return Err(JsError::new("Source larger than destination"));
    }

    // Now safe to copy
    unsafe {
        let dest = std::slice::from_raw_parts_mut(dest_ptr as *mut u8, dest_len);
        dest[..src.len()].copy_from_slice(src);
    }
    Ok(())
}
```

### 2.3 Use-After-Free

**The Problem:**
```rust
// UNSAFE: Use after free
static mut GLOBAL_PTR: *mut Data = std::ptr::null_mut();

#[wasm_bindgen]
pub fn store_data(data: Data) {
    unsafe {
        let boxed = Box::new(data);
        GLOBAL_PTR = Box::into_raw(boxed);
    }
}

#[wasm_bindgen]
pub fn free_data() {
    unsafe {
        if !GLOBAL_PTR.is_null() {
            drop(Box::from_raw(GLOBAL_PTR));
            // BUG: Didn't set to null!
        }
    }
}

#[wasm_bindgen]
pub fn use_data() {
    unsafe {
        // Use after free if free_data was called!
        (*GLOBAL_PTR).do_something();
    }
}
```

**Safe Alternative:**
```rust
use std::sync::OnceLock;
use std::cell::RefCell;

thread_local! {
    static DATA: RefCell<Option<Data>> = RefCell::new(None);
}

#[wasm_bindgen]
pub fn store_data(data: Data) {
    DATA.with(|d| {
        *d.borrow_mut() = Some(data);
    });
}

#[wasm_bindgen]
pub fn free_data() {
    DATA.with(|d| {
        *d.borrow_mut() = None;
    });
}

#[wasm_bindgen]
pub fn use_data() -> Result<(), JsError> {
    DATA.with(|d| {
        match d.borrow().as_ref() {
            Some(data) => {
                data.do_something();
                Ok(())
            }
            None => Err(JsError::new("Data not initialized or already freed"))
        }
    })
}
```

### 2.4 Integer Overflow

```rust
// UNSAFE: Integer overflow can cause issues
#[wasm_bindgen]
pub fn allocate_buffer(count: u32, size: u32) -> Vec<u8> {
    let total = count * size; // Can overflow!
    vec![0u8; total as usize]
}

// SAFE: Use checked arithmetic
#[wasm_bindgen]
pub fn allocate_buffer_safe(count: u32, size: u32) -> Result<Vec<u8>, JsError> {
    let total = count
        .checked_mul(size)
        .ok_or_else(|| JsError::new("Integer overflow"))?;

    Ok(vec![0u8; total as usize])
}
```

---

## 3. Rust-Specific Security

### 3.1 Minimizing Unsafe Code

```rust
// Count unsafe blocks with cargo-geiger
// cargo install cargo-geiger
// cargo geiger

// BAD: Large unsafe block
unsafe {
    // Hundreds of lines of code
    // Hard to audit
}

// GOOD: Minimal unsafe with safety invariants documented
/// # Safety
/// - `ptr` must be valid for reads of `len` bytes
/// - `ptr` must be properly aligned for T
/// - The memory must not be mutated during this call
unsafe fn read_slice<T>(ptr: *const T, len: usize) -> &[T] {
    std::slice::from_raw_parts(ptr, len)
}
```

### 3.2 Safe Abstractions Over Unsafe

```rust
/// A safe wrapper around raw memory access
pub struct SafeBuffer {
    data: Vec<u8>,
}

impl SafeBuffer {
    pub fn new(size: usize) -> Self {
        SafeBuffer {
            data: vec![0; size],
        }
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    pub fn set(&mut self, index: usize, value: u8) -> Result<(), &'static str> {
        if index >= self.data.len() {
            return Err("Index out of bounds");
        }
        self.data[index] = value;
        Ok(())
    }

    pub fn slice(&self, start: usize, end: usize) -> Option<&[u8]> {
        self.data.get(start..end)
    }
}

// Export safe interface
#[wasm_bindgen]
impl SafeBuffer {
    #[wasm_bindgen(constructor)]
    pub fn js_new(size: usize) -> Self {
        Self::new(size)
    }

    #[wasm_bindgen]
    pub fn js_get(&self, index: usize) -> Option<u8> {
        self.get(index)
    }

    #[wasm_bindgen]
    pub fn js_set(&mut self, index: usize, value: u8) -> Result<(), JsError> {
        self.set(index, value).map_err(JsError::new)
    }
}
```

### 3.3 Panic Handling

```rust
use std::panic;

#[wasm_bindgen(start)]
pub fn init() {
    // Always set panic hook for debugging
    console_error_panic_hook::set_once();

    // Optionally catch panics
    panic::set_hook(Box::new(|info| {
        // Log panic info
        web_sys::console::error_1(&format!("Panic: {}", info).into());

        // Could also send to error tracking service
    }));
}

// For critical functions, catch panics
#[wasm_bindgen]
pub fn critical_operation(data: &[u8]) -> Result<Vec<u8>, JsError> {
    match panic::catch_unwind(|| {
        process_data(data)
    }) {
        Ok(result) => result,
        Err(_) => Err(JsError::new("Internal error: operation panicked"))
    }
}
```

### 3.4 Cargo.toml Security Settings

```toml
[profile.release]
# Enable overflow checks even in release
overflow-checks = true

# Abort on panic (smaller binary, no unwinding)
panic = "abort"

# LTO for better optimization and smaller attack surface
lto = true

[dependencies]
# Use exact versions for security-critical deps
ring = "=0.17.7"

# Audit-friendly dependencies
[dependencies.serde]
version = "1.0"
default-features = false
features = ["derive"]  # Only what you need
```

---

## 4. Input Validation

### 4.1 Validate All JavaScript Input

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_user_input(input: &str) -> Result<String, JsError> {
    // Validate length
    if input.is_empty() {
        return Err(JsError::new("Input cannot be empty"));
    }
    if input.len() > 10000 {
        return Err(JsError::new("Input too long (max 10000 chars)"));
    }

    // Validate content
    if !input.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err(JsError::new("Input contains invalid characters"));
    }

    // Now safe to process
    Ok(process_safe_input(input))
}

#[wasm_bindgen]
pub fn process_numeric_input(value: f64) -> Result<f64, JsError> {
    // Check for special values
    if value.is_nan() {
        return Err(JsError::new("NaN is not allowed"));
    }
    if value.is_infinite() {
        return Err(JsError::new("Infinite values not allowed"));
    }

    // Check range
    if value < 0.0 || value > 1000000.0 {
        return Err(JsError::new("Value out of range [0, 1000000]"));
    }

    Ok(compute(value))
}
```

### 4.2 Sanitize Output for DOM

```rust
/// Escape HTML special characters
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[wasm_bindgen]
pub fn safe_render(user_content: &str) -> String {
    // Always escape before sending to DOM
    escape_html(user_content)
}
```

### 4.3 Type-Safe Parsing

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct UserInput {
    #[serde(default)]
    name: String,
    #[serde(default)]
    age: u32,
    #[serde(default)]
    email: Option<String>,
}

impl UserInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() {
            return Err("Name required".into());
        }
        if self.name.len() > 100 {
            return Err("Name too long".into());
        }
        if self.age > 150 {
            return Err("Invalid age".into());
        }
        if let Some(ref email) = self.email {
            if !email.contains('@') {
                return Err("Invalid email".into());
            }
        }
        Ok(())
    }
}

#[wasm_bindgen]
pub fn parse_and_validate(json: &str) -> Result<JsValue, JsError> {
    let input: UserInput = serde_json::from_str(json)
        .map_err(|e| JsError::new(&format!("Parse error: {}", e)))?;

    input.validate()
        .map_err(|e| JsError::new(&format!("Validation error: {}", e)))?;

    // Process validated input...
    Ok(JsValue::TRUE)
}
```

---

## 5. Cryptographic Considerations

### 5.1 Timing Attack Resistance

```rust
// BAD: Variable-time comparison
fn insecure_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for i in 0..a.len() {
        if a[i] != b[i] {
            return false; // Early exit leaks timing info!
        }
    }
    true
}

// GOOD: Constant-time comparison
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for i in 0..a.len() {
        result |= a[i] ^ b[i];
    }
    result == 0
}

// BEST: Use a library
use subtle::ConstantTimeEq;

fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    a.ct_eq(b).into()
}
```

### 5.2 Secure Random Numbers

```rust
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;

#[wasm_bindgen]
pub fn generate_random_bytes(len: usize) -> Result<Vec<u8>, JsError> {
    let array = Uint8Array::new_with_length(len as u32);

    web_sys::window()
        .ok_or_else(|| JsError::new("No window"))?
        .crypto()
        .map_err(|_| JsError::new("No crypto API"))?
        .get_random_values_with_array_buffer_view(&array)
        .map_err(|_| JsError::new("Failed to get random values"))?;

    Ok(array.to_vec())
}

// For cryptographic operations, use established libraries
// with getrandom feature for WASM
// getrandom = { version = "0.2", features = ["js"] }
```

### 5.3 Recommended Crypto Libraries

```toml
[dependencies]
# For general crypto
ring = "0.17"  # Note: Larger binary but battle-tested

# For password hashing
argon2 = "0.5"

# For constant-time operations
subtle = "2.5"

# For random numbers in WASM
getrandom = { version = "0.2", features = ["js"] }

# Enable JS feature for WASM compatibility
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

### 5.4 Key Handling

```rust
use zeroize::Zeroize;

/// Secure key container that zeros memory on drop
pub struct SecretKey {
    key: Vec<u8>,
}

impl SecretKey {
    pub fn new(key: Vec<u8>) -> Self {
        SecretKey { key }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }
}

impl Drop for SecretKey {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

// Don't implement Clone, Debug, or Display for secrets!
```

---

## 6. Supply Chain Security

### 6.1 Dependency Auditing

```bash
# Install audit tools
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-geiger

# Check for known vulnerabilities
cargo audit

# Check licenses and sources
cargo deny check

# Count unsafe code in dependencies
cargo geiger
```

### 6.2 cargo-audit Configuration

```toml
# .cargo/audit.toml
[advisories]
ignore = []  # Don't ignore any advisories

[database]
path = "~/.cargo/advisory-db"
url = "https://github.com/RustSec/advisory-db"

[output]
deny = ["unmaintained", "yanked"]
```

### 6.3 cargo-deny Configuration

```toml
# deny.toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
default = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
    # Crates with known issues
]

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

### 6.4 Lockfile Best Practices

```toml
# Cargo.toml
[package]
# Always commit Cargo.lock for applications
# This ensures reproducible builds

# Use exact versions for security-critical deps
[dependencies]
ring = "=0.17.7"

# Review before updating
# cargo update --dry-run
```

---

## 7. Content Security Policy

### 7.1 Recommended CSP for WASM

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self' 'wasm-unsafe-eval';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data:;
  connect-src 'self' https://api.example.com;
  object-src 'none';
  base-uri 'self';
  frame-ancestors 'none';
```

### 7.2 Phoenix Plug Configuration

```elixir
# lib/my_app_web/plugs/security_headers.ex
defmodule MyAppWeb.Plugs.SecurityHeaders do
  import Plug.Conn

  def init(opts), do: opts

  def call(conn, _opts) do
    conn
    |> put_resp_header("content-security-policy", csp_header())
    |> put_resp_header("x-content-type-options", "nosniff")
    |> put_resp_header("x-frame-options", "DENY")
    |> put_resp_header("x-xss-protection", "1; mode=block")
    |> put_resp_header("referrer-policy", "strict-origin-when-cross-origin")
  end

  defp csp_header do
    [
      "default-src 'self'",
      "script-src 'self' 'wasm-unsafe-eval'",
      "style-src 'self' 'unsafe-inline'",
      "img-src 'self' data: blob:",
      "connect-src 'self' wss://#{host()}",
      "font-src 'self'",
      "object-src 'none'",
      "base-uri 'self'",
      "frame-ancestors 'none'"
    ]
    |> Enum.join("; ")
  end

  defp host, do: Application.get_env(:my_app, :host, "localhost")
end
```

### 7.3 CSP Directive Reference

| Directive | For WASM | Notes |
|-----------|----------|-------|
| `'wasm-unsafe-eval'` | Required | Allows WASM compilation |
| `'unsafe-eval'` | Avoid | Also allows JS eval() |
| `'self'` | Recommended | Same-origin only |
| `'none'` | For unused | Block entirely |

---

## 8. Common Vulnerabilities

### 8.1 CVE Examples (2024-2025)

| CVE | Type | Impact | Lesson |
|-----|------|--------|--------|
| CVE-2025-5419 | V8 OOB R/W | Code execution | Keep browsers updated |
| CVE-2021-38297 | Go WASM overflow | Memory corruption | Validate loader inputs |
| Various wabt | Buffer overflows | Denial of service | Use stable tool versions |

### 8.2 Vulnerability Categories

```
1. Memory Corruption (within sandbox)
   ├── Buffer overflow
   ├── Use-after-free
   ├── Double-free
   └── Integer overflow

2. Logic Errors
   ├── Authentication bypass
   ├── Authorization flaws
   └── Input validation failures

3. Information Disclosure
   ├── Timing side-channels
   ├── Error message leakage
   └── Memory content exposure

4. Denial of Service
   ├── Infinite loops
   ├── Memory exhaustion
   └── Stack overflow (recursion)

5. Supply Chain
   ├── Malicious dependencies
   ├── Compromised build tools
   └── Typosquatting
```

### 8.3 Browser-Specific Concerns

```rust
// Check for feature support before using
#[wasm_bindgen]
pub fn check_crypto_support() -> bool {
    web_sys::window()
        .and_then(|w| w.crypto().ok())
        .is_some()
}

// Don't assume all browsers behave identically
// Test across Chrome, Firefox, Safari, Edge
```

---

## 9. Security Tools

### 9.1 Static Analysis

```bash
# Clippy with security lints
cargo clippy -- \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  -W clippy::panic \
  -W clippy::todo \
  -W clippy::unimplemented \
  -D clippy::mem_forget

# Miri for undefined behavior detection (nightly)
cargo +nightly miri test

# cargo-fuzz for fuzzing
cargo install cargo-fuzz
cargo fuzz run my_fuzz_target
```

### 9.2 Runtime Checks

```rust
// Enable debug assertions in tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_bounds() {
        // Will panic in debug if bounds violated
        let arr = vec![1, 2, 3];
        assert!(arr.get(5).is_none());
    }
}

// Use debug_assert! for expensive checks
fn process(data: &[u8]) {
    debug_assert!(!data.is_empty(), "Data should not be empty");
    debug_assert!(data.len() <= MAX_SIZE, "Data too large");
    // ...
}
```

### 9.3 Fuzzing Setup

```rust
// fuzz/fuzz_targets/process_input.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // This will be called with random inputs
    let _ = my_crate::process_input(data);
});
```

```toml
# fuzz/Cargo.toml
[package]
name = "my-fuzz"
version = "0.0.0"
publish = false
edition = "2021"

[package.metadata]
cargo-fuzz = true

[[bin]]
name = "process_input"
path = "fuzz_targets/process_input.rs"

[dependencies]
libfuzzer-sys = "0.4"
my_crate = { path = ".." }
```

---

## 10. Patterns

### Pattern 1: Defense in Depth

```rust
#[wasm_bindgen]
pub fn secure_operation(input: &str) -> Result<String, JsError> {
    // Layer 1: Input validation
    validate_input(input)?;

    // Layer 2: Sanitization
    let sanitized = sanitize(input);

    // Layer 3: Safe processing
    let result = process_safely(&sanitized)?;

    // Layer 4: Output encoding
    Ok(encode_output(&result))
}
```

### Pattern 2: Fail-Safe Defaults

```rust
#[derive(Default)]
pub struct Config {
    /// Maximum input size (default: 1MB)
    pub max_input_size: usize,
    /// Enable strict validation (default: true)
    pub strict_mode: bool,
    /// Allow untrusted sources (default: false)
    pub allow_untrusted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_input_size: 1024 * 1024,
            strict_mode: true,        // Safe default
            allow_untrusted: false,   // Safe default
        }
    }
}
```

### Pattern 3: Capability-Based Security

```rust
/// Marker trait for capabilities
pub trait Capability {}

pub struct ReadCapability;
pub struct WriteCapability;
pub struct NetworkCapability;

impl Capability for ReadCapability {}
impl Capability for WriteCapability {}
impl Capability for NetworkCapability {}

/// Only allow operations if capability is provided
pub fn read_file<C: Capability>(
    _cap: &ReadCapability,
    path: &str,
) -> Result<Vec<u8>, Error> {
    // Can only be called with ReadCapability
    do_read(path)
}
```

### Pattern 4: Secure Initialization

```rust
use std::sync::Once;

static INIT: Once = Once::new();
static mut INITIALIZED: bool = false;

#[wasm_bindgen(start)]
pub fn init() {
    INIT.call_once(|| {
        // Set panic hook
        console_error_panic_hook::set_once();

        // Initialize crypto
        if !check_crypto_available() {
            web_sys::console::error_1(
                &"Crypto API not available - some features disabled".into()
            );
        }

        unsafe { INITIALIZED = true; }
    });
}

fn require_init() -> Result<(), JsError> {
    if unsafe { !INITIALIZED } {
        return Err(JsError::new("Module not initialized"));
    }
    Ok(())
}
```

### Pattern 5: Audit Logging

```rust
#[cfg(feature = "audit")]
fn audit_log(event: &str, details: &str) {
    let timestamp = js_sys::Date::now();
    web_sys::console::log_1(
        &format!("[AUDIT {}] {}: {}", timestamp, event, details).into()
    );
}

#[cfg(not(feature = "audit"))]
fn audit_log(_event: &str, _details: &str) {}

#[wasm_bindgen]
pub fn sensitive_operation(data: &str) -> Result<(), JsError> {
    audit_log("SENSITIVE_OP_START", "Beginning sensitive operation");

    let result = do_sensitive_work(data);

    match &result {
        Ok(_) => audit_log("SENSITIVE_OP_SUCCESS", "Operation completed"),
        Err(e) => audit_log("SENSITIVE_OP_FAILURE", &e.to_string()),
    }

    result
}
```

---

## 11. Anti-Patterns

### Anti-Pattern 1: Trusting Client Input

```rust
// BAD: No validation
#[wasm_bindgen]
pub fn process(data: JsValue) {
    let obj: MyStruct = serde_wasm_bindgen::from_value(data).unwrap();
    // Direct use without validation!
}

// GOOD: Validate everything
#[wasm_bindgen]
pub fn process(data: JsValue) -> Result<(), JsError> {
    let obj: MyStruct = serde_wasm_bindgen::from_value(data)
        .map_err(|e| JsError::new(&e.to_string()))?;

    obj.validate()?;
    // Now safe to use
    Ok(())
}
```

### Anti-Pattern 2: Excessive Unsafe

```rust
// BAD: Entire function is unsafe
pub unsafe fn bad_process(ptr: *mut u8, len: usize) {
    // 50 lines of code, all unsafe
}

// GOOD: Minimal unsafe, maximum safe
pub fn good_process(ptr: *mut u8, len: usize) -> Result<(), Error> {
    // Validate inputs
    if ptr.is_null() {
        return Err(Error::NullPointer);
    }

    // Minimal unsafe block
    let slice = unsafe {
        std::slice::from_raw_parts_mut(ptr, len)
    };

    // Rest is safe Rust
    process_slice(slice)
}
```

### Anti-Pattern 3: Leaking Sensitive Info

```rust
// BAD: Leaks sensitive info in errors
#[wasm_bindgen]
pub fn authenticate(password: &str) -> Result<(), JsError> {
    if password != SECRET_PASSWORD {
        return Err(JsError::new(&format!(
            "Wrong password. Expected: {}", SECRET_PASSWORD // NEVER DO THIS!
        )));
    }
    Ok(())
}

// GOOD: Generic error messages
#[wasm_bindgen]
pub fn authenticate(password: &str) -> Result<(), JsError> {
    if !verify_password(password) {
        return Err(JsError::new("Authentication failed"));
    }
    Ok(())
}
```

### Anti-Pattern 4: Ignoring Errors

```rust
// BAD: Ignoring potential errors
let _ = risky_operation();  // Error silently ignored!

// GOOD: Handle or propagate
risky_operation()?;  // Propagate to caller

// Or explicitly handle
match risky_operation() {
    Ok(result) => process(result),
    Err(e) => {
        log_error(&e);
        return default_value();
    }
}
```

### Anti-Pattern 5: Hardcoded Secrets

```rust
// BAD: Hardcoded secrets
const API_KEY: &str = "sk_live_abc123";  // Compiled into WASM!

// GOOD: Receive secrets at runtime from secure source
#[wasm_bindgen]
pub fn init_with_config(config: JsValue) -> Result<(), JsError> {
    let config: Config = serde_wasm_bindgen::from_value(config)?;
    // API key comes from secure storage, not compiled in
    Ok(())
}
```

---

## 12. Common Failures & Solutions

### Failure 1: WASM Blocked by CSP

```
Refused to compile WebAssembly module because 'wasm-unsafe-eval' is not in script-src
```

**Solution:**
```
Content-Security-Policy: script-src 'self' 'wasm-unsafe-eval';
```

### Failure 2: Panic in Production

```
RuntimeError: unreachable
```

**Solution:**
```rust
// Always set panic hook
console_error_panic_hook::set_once();

// Use Result instead of unwrap/expect
value.ok_or_else(|| JsError::new("Value missing"))?
```

### Failure 3: Memory Exhaustion

```
Out of memory
```

**Solution:**
```rust
// Limit allocations
const MAX_ALLOCATION: usize = 100 * 1024 * 1024; // 100MB

pub fn allocate(size: usize) -> Result<Vec<u8>, JsError> {
    if size > MAX_ALLOCATION {
        return Err(JsError::new("Allocation too large"));
    }
    Ok(vec![0; size])
}
```

### Failure 4: Timing Attack Possible

```
Password comparison leaks timing information
```

**Solution:**
```rust
use subtle::ConstantTimeEq;

fn verify_password(input: &[u8], stored: &[u8]) -> bool {
    input.ct_eq(stored).into()
}
```

### Failure 5: Vulnerable Dependency

```
cargo audit: 1 vulnerability found
```

**Solution:**
```bash
# Update the specific dependency
cargo update -p vulnerable-crate

# Or if locked to old version, check for patches
cargo audit fix
```

---

## 13. Security Checklist

### Pre-Development
- [ ] Define security requirements
- [ ] Choose audited dependencies
- [ ] Set up cargo-audit in CI
- [ ] Configure clippy security lints

### During Development
- [ ] Minimize unsafe code
- [ ] Validate all inputs from JavaScript
- [ ] Use checked arithmetic
- [ ] Handle all errors explicitly
- [ ] Use constant-time comparisons for secrets
- [ ] Zeroize sensitive data on drop

### Pre-Release
- [ ] Run `cargo audit`
- [ ] Run `cargo clippy` with security lints
- [ ] Run fuzzer on input handling
- [ ] Review all unsafe blocks
- [ ] Test with restrictive CSP
- [ ] Check binary for debug symbols

### Deployment
- [ ] Set appropriate CSP headers
- [ ] Enable HTTPS only
- [ ] Configure security headers
- [ ] Set up vulnerability monitoring
- [ ] Document security considerations

### Maintenance
- [ ] Regularly update dependencies
- [ ] Monitor for new CVEs
- [ ] Re-audit after major changes
- [ ] Review access to build systems

---

## Sources

- [WebAssembly Security](https://webassembly.org/docs/security/)
- [Memory Corruption in WebAssembly](https://medium.com/@instatunnel/memory-corruption-in-webassembly-native-exploits-in-your-browser-f587d8938511)
- [WebAssembly and Security Review](https://arxiv.org/html/2407.12297v1)
- [CT-Wasm: Constant-Time WebAssembly](https://github.com/PLSysSec/ct-wasm)
- [Security and Correctness in Wasmtime](https://bytecodealliance.org/articles/security-and-correctness-in-wasmtime)
- [CSP script-src wasm-unsafe-eval](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/script-src)
- [Rust Security Best Practices](https://www.mayhem.security/blog/best-practices-for-secure-programming-in-rust)
- [Cargo Audit](https://github.com/rustsec/cargo-audit)
- [OWASP XSS Prevention](https://cheatsheetseries.owasp.org/cheatsheets/Cross_Site_Scripting_Prevention_Cheat_Sheet.html)

---

*Document completed: Step 5 of Rust WebAssembly Skill Research*
