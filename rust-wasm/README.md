# Rust WebAssembly Skill Package

> Comprehensive skill for Rust WebAssembly development with Phoenix LiveView, Tailwind CSS, and JavaScript interop.

## Overview

This skill package provides guidance for building WebAssembly applications using Rust, with special focus on:

- **Rust WASM Frameworks**: Leptos, Yew, Dioxus, Sycamore
- **Phoenix LiveView Integration**: Hooks, hybrid applications
- **JavaScript Interop**: wasm-bindgen, web-sys, js-sys
- **Performance Optimization**: SIMD, profiling, binary size
- **Security Best Practices**: Memory safety, CSP, supply chain
- **Styling**: Tailwind CSS, CSS-in-Rust solutions
- **Testing**: wasm-bindgen-test, E2E, debugging

## Contents

### Core Documentation

| File | Description | Words |
|------|-------------|-------|
| `rust_wasm_skill.md` | Master skill file | ~1,500 |
| `rust_wasm_cheatsheet.md` | Quick reference | ~1,000 |

### Research Documents (Detailed)

| File | Topic | Words |
|------|-------|-------|
| `rust_wasm_core.md` | Toolchain & Project Setup | ~4,200 |
| `rust_wasm_frameworks.md` | Leptos, Yew, Dioxus | ~4,800 |
| `rust_wasm_interop.md` | JavaScript Interop | ~5,200 |
| `rust_wasm_liveview.md` | Phoenix LiveView | ~5,500 |
| `rust_wasm_security.md` | Security Practices | ~5,000 |
| `rust_wasm_performance.md` | Optimization | ~5,300 |
| `rust_wasm_testing.md` | Testing & Debugging | ~5,800 |
| `rust_wasm_styling.md` | CSS & Tailwind | ~7,500 |

### Subskills (Quick Reference)

Located in `subskills/`:
- `core.md` - Toolchain essentials
- `frameworks.md` - Framework comparison
- `interop.md` - JS interop patterns
- `liveview.md` - LiveView integration
- `security.md` - Security checklist
- `performance.md` - Optimization tips
- `testing.md` - Test commands
- `styling.md` - Tailwind setup

### Code Examples

Located in `examples/`:
- `minimal_leptos.rs` - Basic Leptos app
- `js_interop.rs` - wasm-bindgen patterns
- `async_fetch.rs` - Async/await with fetch
- `image_processing.rs` - Performance-critical WASM
- `liveview_hook.js` - Complete LiveView integration

## Usage

### As Claude Code Skill

1. Copy contents to your Claude Code skills directory
2. Reference `rust_wasm_skill.md` as the master skill
3. Subskills are activated based on context

### As Reference Documentation

Browse the research documents for comprehensive coverage of each topic. Each document includes:
- Conceptual explanations
- Code examples
- 5+ patterns
- 5+ anti-patterns
- 5+ common failures
- Quick reference section

## Key Information

### Important Note (2025)

**wasm-pack is archived** as of January 2025. Use `wasm-bindgen` CLI directly or Trunk as the build tool.

### Recommended Stack

```
Frontend: Leptos + Trunk + Tailwind CSS
Backend: Phoenix LiveView
Interop: wasm-bindgen + web-sys
```

### Version Information

- Rust: 1.75+ (stable WASM target)
- wasm-bindgen: 0.2.x
- Leptos: 0.7.x
- Trunk: 0.21.x
- Tailwind CSS: 4.x

## Related Skills

This skill has overlaps and integrations with:

- `phoenix-liveview` - Phoenix LiveView patterns
- `tailwind` - Tailwind CSS styling
- `rust-nif` - Rust NIFs for Elixir backend
- `elixir` - Elixir language and OTP

## License

This skill package is provided for educational use with Claude Code.
