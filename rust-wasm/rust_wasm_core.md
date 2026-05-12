# Rust WebAssembly Core: Toolchain & Project Setup

> **Version**: 2025
> **Status**: Complete Reference

## Table of Contents
1. [Overview](#1-overview)
2. [Toolchain Setup](#2-toolchain-setup)
3. [Project Structure](#3-project-structure)
4. [Cargo Configuration](#4-cargo-configuration)
5. [Build Workflow](#5-build-workflow)
6. [wasm-bindgen CLI](#6-wasm-bindgen-cli)
7. [Trunk Build System](#7-trunk-build-system)
8. [wasm-opt Optimization](#8-wasm-opt-optimization)
9. [Binary Size Reduction](#9-binary-size-reduction)
10. [WASI & Component Model](#10-wasi--component-model)
11. [Patterns](#11-patterns)
12. [Anti-Patterns](#12-anti-patterns)
13. [Common Failures & Solutions](#13-common-failures--solutions)
14. [Quick Reference](#14-quick-reference)

---

## 1. Overview

Rust compiles to WebAssembly via the `wasm32-unknown-unknown` target, producing `.wasm` binaries that run in browsers and other WebAssembly runtimes. The 2025 ecosystem has evolved significantly with the sunsetting of the rustwasm organization and wasm-pack, replaced by direct use of `wasm-bindgen-cli` and build tools like Trunk.

### Core Components

| Component | Purpose | Status (2025) |
|-----------|---------|---------------|
| `wasm32-unknown-unknown` | Browser WASM target | Stable, Tier 2 |
| `wasm32-wasip1` | WASI target (server-side) | Stable, replaces wasm32-wasi |
| `wasm-bindgen` | JS/Rust interop | Active (new org) |
| `wasm-bindgen-cli` | Build tool | Primary build method |
| `wasm-opt` | Binary optimizer | Essential for production |
| `Trunk` | Build/bundle system | Recommended for apps |
| `wasm-pack` | Legacy build tool | Archived (July 2025) |

---

## 2. Toolchain Setup

### 2.1 Installing the Target

```bash
# Add the WebAssembly target
rustup target add wasm32-unknown-unknown

# For WASI applications
rustup target add wasm32-wasip1

# Verify installation
rustup target list --installed | grep wasm
```

### 2.2 rust-toolchain.toml

Pin your toolchain and targets for reproducible builds:

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
targets = ["wasm32-unknown-unknown"]
```

### 2.3 Installing Build Tools

**Using Cargo:**
```bash
# Install wasm-bindgen CLI (match version to Cargo.toml!)
cargo install wasm-bindgen-cli --version 0.2.100

# Install Trunk (recommended for web apps)
cargo install trunk

# Install wasm-opt via Binaryen
# On macOS:
brew install binaryen
# On Ubuntu:
apt install binaryen
# Or via cargo:
cargo install wasm-opt
```

**Using Mise (Recommended for Teams):**
```toml
# mise.toml
[tools]
rust = "stable"
wasm-bindgen-cli = "0.2.100"
binaryen = "latest"
```

```bash
mise install
```

---

## 3. Project Structure

### 3.1 Minimal Library Project

```
my-wasm-lib/
├── Cargo.toml
├── src/
│   └── lib.rs
└── .cargo/
    └── config.toml
```

### 3.2 Full Application Project

```
my-wasm-app/
├── Cargo.toml
├── Trunk.toml                    # Trunk configuration
├── rust-toolchain.toml           # Pinned toolchain
├── index.html                    # Entry point for Trunk
├── input.css                     # Tailwind input (optional)
├── tailwind.config.js            # Tailwind config (optional)
├── .cargo/
│   └── config.toml               # Cargo configuration
├── src/
│   ├── lib.rs                    # WASM entry point
│   ├── app.rs                    # Application logic
│   ├── components/               # UI components
│   │   ├── mod.rs
│   │   └── button.rs
│   └── utils/                    # Utility functions
│       ├── mod.rs
│       └── helpers.rs
├── tests/
│   ├── unit_tests.rs             # Native Rust tests
│   └── wasm_tests.rs             # Browser tests
└── dist/                         # Build output (gitignored)
    ├── index.html
    ├── my-wasm-app.js
    └── my-wasm-app_bg.wasm
```

### 3.3 lib.rs Entry Point

```rust
// src/lib.rs
use wasm_bindgen::prelude::*;

// Initialize panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

// Export functions to JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Export structs
#[wasm_bindgen]
pub struct Calculator {
    value: f64,
}

#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn add(&mut self, n: f64) {
        self.value += n;
    }

    pub fn result(&self) -> f64 {
        self.value
    }
}
```

---

## 4. Cargo Configuration

### 4.1 Cargo.toml for WASM Library

```toml
[package]
name = "my-wasm-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]  # cdylib for WASM, rlib for tests

[dependencies]
wasm-bindgen = "0.2.100"
console_error_panic_hook = "0.1"

# Optional: JavaScript standard library bindings
js-sys = "0.3"

# Optional: Web API bindings (enable specific features)
[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlElement",
]

[dev-dependencies]
wasm-bindgen-test = "0.3"

# Release profile optimized for WASM
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Single codegen unit for better optimization
panic = "abort"          # No unwinding (smaller binary)
strip = true             # Strip symbols
```

### 4.2 Target-Specific Dependencies

```toml
# Dependencies only for WASM builds
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2.100"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

# Dependencies only for native builds (tests, etc.)
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
tokio = { version = "1", features = ["full"] }
```

### 4.3 .cargo/config.toml

```toml
# .cargo/config.toml

# Default to WASM target
[build]
target = "wasm32-unknown-unknown"

# WASM-specific settings
[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"

# Enable SIMD (if needed)
# rustflags = ["-C", "target-feature=+simd128"]

# For WASI targets
[target.wasm32-wasip1]
runner = "wasmtime run --dir ."

# Alias for convenience
[alias]
wasm-build = "build --target wasm32-unknown-unknown --release"
wasm-test = "test --target wasm32-unknown-unknown"
```

---

## 5. Build Workflow

### 5.1 Post wasm-pack Era (2025)

With wasm-pack archived, the recommended workflow uses wasm-bindgen-cli directly:

```bash
# Step 1: Build the WASM binary
cargo build --target wasm32-unknown-unknown --release

# Step 2: Generate JavaScript bindings
wasm-bindgen \
    --target web \
    --out-dir ./dist \
    ./target/wasm32-unknown-unknown/release/my_wasm_lib.wasm

# Step 3: Optimize the binary (production)
wasm-opt -Oz \
    -o ./dist/my_wasm_lib_bg.wasm \
    ./dist/my_wasm_lib_bg.wasm
```

### 5.2 Build Script (build.sh)

```bash
#!/bin/bash
set -e

PROJECT_NAME="my_wasm_lib"
TARGET_DIR="./target/wasm32-unknown-unknown/release"
OUT_DIR="./dist"

echo "🔨 Building WASM..."
cargo build --target wasm32-unknown-unknown --release

echo "🔗 Generating bindings..."
wasm-bindgen \
    --target web \
    --out-dir "$OUT_DIR" \
    "$TARGET_DIR/${PROJECT_NAME}.wasm"

echo "📦 Optimizing..."
wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-mutable-globals \
    -o "$OUT_DIR/${PROJECT_NAME}_bg.wasm" \
    "$OUT_DIR/${PROJECT_NAME}_bg.wasm"

# Report sizes
echo "📊 Build complete!"
ls -lh "$OUT_DIR"/*.wasm
```

### 5.3 Makefile Alternative

```makefile
# Makefile
.PHONY: build dev clean test

PROJECT := my_wasm_lib
TARGET := wasm32-unknown-unknown
OUT_DIR := dist

build:
	cargo build --target $(TARGET) --release
	wasm-bindgen --target web --out-dir $(OUT_DIR) \
		./target/$(TARGET)/release/$(PROJECT).wasm
	wasm-opt -Oz -o $(OUT_DIR)/$(PROJECT)_bg.wasm \
		$(OUT_DIR)/$(PROJECT)_bg.wasm

dev:
	cargo build --target $(TARGET)
	wasm-bindgen --target web --out-dir $(OUT_DIR) \
		./target/$(TARGET)/debug/$(PROJECT).wasm

test:
	cargo test
	wasm-pack test --headless --chrome

clean:
	cargo clean
	rm -rf $(OUT_DIR)
```

---

## 6. wasm-bindgen CLI

### 6.1 Target Options

| Target | Use Case | Output |
|--------|----------|--------|
| `--target web` | Modern browsers (ESM) | ES module, async loading |
| `--target bundler` | Webpack/Parcel | Requires bundler |
| `--target nodejs` | Node.js | CommonJS module |
| `--target no-modules` | Legacy browsers | Global `wasm_bindgen` |
| `--target deno` | Deno runtime | Deno-compatible ESM |

### 6.2 Common Options

```bash
wasm-bindgen \
    --target web \                    # Output target
    --out-dir ./dist \                # Output directory
    --out-name my_module \            # Custom output name
    --typescript \                    # Generate .d.ts (default)
    --no-typescript \                 # Skip .d.ts generation
    --reference-types \               # Enable reference types
    --weak-refs \                     # Enable weak references
    --split-linked-modules \          # Lazy-load linked modules
    ./target/wasm32-unknown-unknown/release/my_lib.wasm
```

### 6.3 Using in HTML (--target web)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>WASM App</title>
</head>
<body>
    <script type="module">
        import init, { greet, Calculator } from './dist/my_wasm_lib.js';

        async function run() {
            // Initialize the WASM module
            await init();

            // Use exported functions
            console.log(greet("World"));

            // Use exported classes
            const calc = new Calculator();
            calc.add(10);
            calc.add(5);
            console.log("Result:", calc.result());
        }

        run();
    </script>
</body>
</html>
```

---

## 7. Trunk Build System

### 7.1 Overview

Trunk is the recommended build system for Rust WASM applications. It handles building, bundling, and serving with hot reload.

### 7.2 Installation & Basic Usage

```bash
# Install
cargo install trunk

# Development server with hot reload
trunk serve

# Production build
trunk build --release

# Build to specific directory
trunk build --release --dist ./public
```

### 7.3 Trunk.toml Configuration

```toml
# Trunk.toml

# Require minimum Trunk version
trunk-version = ">=0.20.0"

[build]
# Source HTML file
target = "index.html"
# Output directory
dist = "dist"
# Public URL prefix
public_url = "/"
# Enable file hashing for cache busting
filehash = true
# Minification: "never", "on_release", "always"
minify = "on_release"
# Inject scripts into HTML
inject_scripts = true

[watch]
# Files to watch for changes
watch = ["src", "index.html", "input.css"]
# Files to ignore
ignore = ["dist", "target"]

[serve]
# Development server settings
address = "127.0.0.1"
port = 8080
# Open browser on start
open = false
# Proxy API requests
# [[serve.proxy]]
# rewrite = "/api/"
# backend = "http://localhost:4000/api/"

# Pre-build hook (e.g., Tailwind CSS)
[[hooks]]
stage = "pre_build"
command = "npx"
command_arguments = ["tailwindcss", "-i", "input.css", "-o", "dist/app.css", "--minify"]

# Post-build hook (e.g., custom optimization)
[[hooks]]
stage = "post_build"
command = "echo"
command_arguments = ["Build complete!"]
```

### 7.4 index.html for Trunk

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>My WASM App</title>

    <!-- Trunk will inject CSS here -->
    <link data-trunk rel="css" href="dist/app.css">

    <!-- Or inline CSS -->
    <link data-trunk rel="inline" type="css" href="src/styles.css">
</head>
<body>
    <div id="app"></div>

    <!-- Trunk will inject the WASM loader script -->
    <link data-trunk rel="rust" href="Cargo.toml"
          data-wasm-opt="z"
          data-bindgen-target="web" />

    <!-- Copy static assets -->
    <link data-trunk rel="copy-dir" href="assets" />

    <!-- Include external JS -->
    <link data-trunk rel="js" href="src/helpers.js" />
</body>
</html>
```

### 7.5 Trunk Asset Types

| Type | Attribute | Purpose |
|------|-----------|---------|
| `rust` | `rel="rust"` | Compile Rust to WASM |
| `sass`/`scss` | `rel="sass"` | Compile Sass |
| `css` | `rel="css"` | Include CSS |
| `js` | `rel="js"` | Include JavaScript |
| `copy-file` | `rel="copy-file"` | Copy single file |
| `copy-dir` | `rel="copy-dir"` | Copy directory |
| `icon` | `rel="icon"` | Set favicon |
| `inline` | `rel="inline"` | Inline file contents |

---

## 8. wasm-opt Optimization

### 8.1 Optimization Levels

| Level | Focus | Use Case |
|-------|-------|----------|
| `-O` | Balanced | General use |
| `-O1` | Light optimization | Fast builds |
| `-O2` | Standard | Good balance |
| `-O3` | Aggressive speed | Performance-critical |
| `-Os` | Size (light) | Size-conscious |
| `-Oz` | Size (aggressive) | Minimum size |

### 8.2 Common Optimization Commands

```bash
# Optimize for size (recommended for web)
wasm-opt -Oz -o output.wasm input.wasm

# Optimize for speed
wasm-opt -O3 -o output.wasm input.wasm

# Enable additional features
wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-mutable-globals \
    --enable-simd \
    -o output.wasm input.wasm

# Maximum size reduction (iterative)
wasm-opt -Oz --converge -o output.wasm input.wasm

# Strip debug info
wasm-opt -Oz --strip-debug -o output.wasm input.wasm

# Dead code elimination after snipping
wasm-opt --dce -o output.wasm input.wasm
```

### 8.3 Size Analysis with twiggy

```bash
# Install twiggy
cargo install twiggy

# Show top functions by size
twiggy top my_lib_bg.wasm

# Show dominators (what keeps code in binary)
twiggy dominators my_lib_bg.wasm

# Show paths to a specific function
twiggy paths my_lib_bg.wasm "function_name"

# Show garbage (unused code)
twiggy garbage my_lib_bg.wasm
```

---

## 9. Binary Size Reduction

### 9.1 Cargo.toml Optimizations

```toml
[profile.release]
# Optimization level: 's' or 'z' for size
opt-level = "z"

# Enable link-time optimization (15-20% smaller)
lto = true

# Single codegen unit (better optimization, slower build)
codegen-units = 1

# Abort on panic (smaller than unwinding)
panic = "abort"

# Strip symbols
strip = true

# Disable incremental compilation
incremental = false
```

### 9.2 Size Reduction Techniques

| Technique | Impact | Trade-off |
|-----------|--------|-----------|
| `opt-level = "z"` | 10-20% smaller | Slower runtime |
| `lto = true` | 15-20% smaller | Longer build time |
| `panic = "abort"` | ~10KB smaller | No backtraces |
| `codegen-units = 1` | 5-10% smaller | Longer build time |
| `wasm-opt -Oz` | 10-20% smaller | Additional build step |
| Remove `std` | ~10KB+ smaller | Limited functionality |
| `wasm-snip` | Variable | Manual intervention |

### 9.3 Removing Panic Infrastructure

```bash
# Snip panic formatting (saves ~20KB)
wasm-snip --snip-rust-panicking-code input.wasm -o output.wasm

# Run DCE after snipping
wasm-opt --dce -Oz -o final.wasm output.wasm
```

### 9.4 Example Size Journey

```
Starting point:                    120 KB
+ opt-level = "z":                  98 KB  (-18%)
+ lto = true:                       82 KB  (-16%)
+ panic = "abort":                  72 KB  (-12%)
+ wasm-opt -Oz:                     58 KB  (-19%)
+ wasm-snip + DCE:                  45 KB  (-22%)
+ gzip compression:                 18 KB  (-60%)
```

---

## 10. WASI & Component Model

### 10.1 WASI Overview

WASI (WebAssembly System Interface) provides system-level APIs for non-browser environments.

| Version | Rust Target | Status |
|---------|-------------|--------|
| WASIp1 (Preview 1) | `wasm32-wasip1` | Stable, maintenance mode |
| WASIp2 (Preview 2) | `wasm32-wasip2` | Stable (Jan 2024) |
| WASIp3 (Preview 3) | Coming | Expected late 2025 |

### 10.2 Building for WASI

```bash
# Add target
rustup target add wasm32-wasip1

# Build
cargo build --target wasm32-wasip1 --release

# Run with Wasmtime
wasmtime run --dir . ./target/wasm32-wasip1/release/my_app.wasm
```

### 10.3 WASI Cargo.toml

```toml
[package]
name = "wasi-app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "wasi-app"
path = "src/main.rs"

[dependencies]
# For WASIp1
# (no special dependencies needed for basic I/O)

# For WASIp2 with wit-bindgen
# wit-bindgen = "0.36"
```

### 10.4 Component Model (wit-bindgen)

For Component Model applications:

```toml
[dependencies]
wit-bindgen = "0.36"
```

```rust
// src/lib.rs
wit_bindgen::generate!({
    world: "my-component",
});

struct MyComponent;

impl Guest for MyComponent {
    fn process(input: String) -> String {
        format!("Processed: {}", input)
    }
}

export!(MyComponent);
```

---

## 11. Patterns

### Pattern 1: Layered Architecture

Separate WASM-specific code from pure Rust logic:

```
src/
├── lib.rs              # WASM bindings only
├── bindings/
│   ├── mod.rs          # wasm-bindgen exports
│   └── types.rs        # Type conversions
└── core/
    ├── mod.rs          # Pure Rust logic
    ├── algorithm.rs    # Business logic
    └── types.rs        # Domain types
```

```rust
// src/core/algorithm.rs (pure Rust, testable without WASM)
pub fn compute(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

// src/lib.rs (WASM bindings)
mod core;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn average(data: &[f64]) -> f64 {
    core::algorithm::compute(data)
}
```

### Pattern 2: Feature Flags for WASM

```toml
# Cargo.toml
[features]
default = []
wasm = ["wasm-bindgen", "console_error_panic_hook"]

[dependencies]
wasm-bindgen = { version = "0.2", optional = true }
console_error_panic_hook = { version = "0.1", optional = true }
```

```rust
// src/lib.rs
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn compute(x: i32) -> i32 {
    x * 2
}
```

### Pattern 3: Initialization with Error Handling

```rust
use wasm_bindgen::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

#[wasm_bindgen(start)]
pub fn init() -> Result<(), JsValue> {
    INIT.call_once(|| {
        // Set panic hook
        console_error_panic_hook::set_once();

        // Any other one-time initialization
        web_sys::console::log_1(&"WASM initialized".into());
    });
    Ok(())
}
```

### Pattern 4: Async Initialization

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

#[wasm_bindgen]
pub async fn init_with_config(config_url: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_str(config_url)).await?;
    let resp: Response = resp_value.dyn_into()?;
    let json = JsFuture::from(resp.json()?).await?;
    // Process config...
    Ok(())
}
```

### Pattern 5: Resource Cleanup

```rust
use wasm_bindgen::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub struct ResourceManager {
    resources: Rc<RefCell<Vec<String>>>,
}

#[wasm_bindgen]
impl ResourceManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            resources: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn allocate(&mut self, name: &str) {
        self.resources.borrow_mut().push(name.to_string());
    }

    // Explicit cleanup method for JS to call
    pub fn cleanup(&mut self) {
        self.resources.borrow_mut().clear();
    }
}

// Automatic cleanup when dropped
impl Drop for ResourceManager {
    fn drop(&mut self) {
        web_sys::console::log_1(&"ResourceManager dropped".into());
    }
}
```

---

## 12. Anti-Patterns

### Anti-Pattern 1: Version Mismatch

**Problem:** wasm-bindgen CLI version doesn't match Cargo dependency.

```
Error: it looks like the Rust project used to create this wasm file
was linked against version 0.2.92 of wasm-bindgen, but this binary
uses version 0.2.100
```

**Solution:** Always pin and sync versions:

```toml
# Cargo.toml
wasm-bindgen = "=0.2.100"  # Exact version
```

```bash
cargo install wasm-bindgen-cli --version 0.2.100 --force
```

### Anti-Pattern 2: Bloated Dependencies

**Problem:** Including heavy crates that inflate binary size.

```toml
# BAD: Full regex crate adds ~100KB
regex = "1"

# BAD: serde_json adds significant size
serde_json = "1"
```

**Solution:** Use lightweight alternatives or feature flags:

```toml
# GOOD: Minimal regex
regex-lite = "0.1"

# GOOD: Minimal JSON
miniserde = "0.1"

# GOOD: Enable only needed serde features
serde = { version = "1", default-features = false, features = ["derive"] }
```

### Anti-Pattern 3: Debug Builds in Production

**Problem:** Shipping debug builds with full debug info.

```bash
# BAD: Debug build (huge binary, slow)
cargo build --target wasm32-unknown-unknown
```

**Solution:** Always use release builds with proper profile:

```bash
# GOOD: Optimized release build
cargo build --target wasm32-unknown-unknown --release
```

### Anti-Pattern 4: Ignoring Binary Size

**Problem:** Not monitoring binary growth during development.

**Solution:** Add size checks to CI:

```yaml
# .github/workflows/size-check.yml
- name: Check WASM size
  run: |
    trunk build --release
    SIZE=$(stat -f%z dist/*.wasm || stat -c%s dist/*.wasm)
    if [ $SIZE -gt 500000 ]; then
      echo "WASM binary exceeds 500KB limit: $SIZE bytes"
      exit 1
    fi
```

### Anti-Pattern 5: Not Using wasm-opt

**Problem:** Skipping post-build optimization.

**Solution:** Always run wasm-opt for production:

```bash
# Include in build pipeline
wasm-opt -Oz -o output.wasm input.wasm
```

---

## 13. Common Failures & Solutions

### Failure 1: Missing Target

```
error[E0463]: can't find crate for `std`
  |
  = note: the `wasm32-unknown-unknown` target may not be installed
```

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

### Failure 2: Version Mismatch

```
it looks like the Rust project used to create this wasm file was linked
against version X.X.XX of wasm-bindgen, but this binary uses version Y.Y.YY
```

**Solution:**
```bash
# Check installed version
wasm-bindgen --version

# Reinstall matching version
cargo install wasm-bindgen-cli --version X.X.XX --force

# Or update Cargo.toml to match installed version
```

### Failure 3: Missing cdylib

```
error: cannot produce cdylib for `my-app v0.1.0` as the target `wasm32-unknown-unknown`
does not support these crate types
```

**Solution:** Add to Cargo.toml:
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

### Failure 4: Incompatible Crate

```
error: the crate `some_crate` cannot be compiled for `wasm32-unknown-unknown`
```

**Solution:** Use target-specific dependencies:
```toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
some_crate = "1.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm_alternative = "1.0"
```

### Failure 5: Trunk Hook Failure

```
error: hook "tailwindcss" returned a bad exit code: 127
```

**Solution:** Ensure hook dependencies are installed:
```bash
npm install -D tailwindcss
# Or use npx in hook
[[hooks]]
command = "npx"
command_arguments = ["tailwindcss", "-i", "input.css", "-o", "dist/app.css"]
```

### Failure 6: LLVM/Bulk Memory Error (2025)

```
LLVM ERROR: Do not know how to split this operator's operand!
```

**Solution:** Add to Cargo.toml:
```toml
[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O", "--enable-bulk-memory", "--enable-mutable-globals"]
```

Or set RUSTFLAGS:
```bash
RUSTFLAGS="-Ctarget-cpu=mvp" cargo build --target wasm32-unknown-unknown
```

### Failure 7: Panic Without Message

```
RuntimeError: unreachable
    at wasm-function[123]:0x4567
```

**Solution:** Add panic hook and debug info:
```rust
// In lib.rs
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}
```

```toml
# In Cargo.toml
[profile.release]
debug = 1  # Include some debug info
```

---

## 14. Quick Reference

### Build Commands

```bash
# Development build
cargo build --target wasm32-unknown-unknown

# Release build
cargo build --target wasm32-unknown-unknown --release

# With Trunk
trunk serve          # Dev server
trunk build --release # Production build

# Generate bindings
wasm-bindgen --target web --out-dir dist ./target/wasm32-unknown-unknown/release/app.wasm

# Optimize
wasm-opt -Oz -o dist/app_bg.wasm dist/app_bg.wasm

# Analyze size
twiggy top dist/app_bg.wasm
```

### Cargo.toml Template

```toml
[package]
name = "my-wasm-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2.100"
console_error_panic_hook = "0.1"
js-sys = "0.3"
web-sys = { version = "0.3", features = ["console"] }

[dev-dependencies]
wasm-bindgen-test = "0.3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Trunk.toml Template

```toml
trunk-version = ">=0.20.0"

[build]
target = "index.html"
dist = "dist"
filehash = true
minify = "on_release"

[serve]
address = "127.0.0.1"
port = 8080
```

---

## Sources

- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [wasm-bindgen CLI Reference](https://rustwasm.github.io/docs/wasm-bindgen/reference/cli.html)
- [Trunk Documentation](https://trunkrs.dev/)
- [Rust and WebAssembly Book](https://rustwasm.github.io/book/)
- [Shrinking .wasm Size](https://rustwasm.github.io/book/reference/code-size.html)
- [Leptos Binary Size Guide](https://book.leptos.dev/deployment/binary_size.html)
- [wasm32-unknown-unknown Target](https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html)
- [wasm32-wasip1 Target](https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html)
- [Life After wasm-pack](https://nickb.dev/blog/life-after-wasm-pack-an-opinionated-deconstruction/)
- [min-sized-rust](https://github.com/johnthagen/min-sized-rust)

---

*Document completed: Step 1 of Rust WebAssembly Skill Research*
