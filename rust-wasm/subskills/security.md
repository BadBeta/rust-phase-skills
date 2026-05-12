# Security Subskill

> Quick reference for WASM security practices.

## When to Activate

Activate when user asks about:
- WASM security model and sandbox
- Memory safety in WASM
- Input validation for WASM
- Content Security Policy for WASM
- Supply chain security (cargo-audit, cargo-deny)
- Cryptography in WASM
- Unsafe Rust in WASM context

## Full Reference

See `rust_wasm_security.md` for complete documentation.

## WASM Security Model

```
┌─────────────────────────────────────────┐
│           Browser Sandbox               │
│  ┌───────────────────────────────────┐  │
│  │         WASM Sandbox              │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │     Linear Memory          │  │  │
│  │  │   (bounds-checked)         │  │  │
│  │  └─────────────────────────────┘  │  │
│  │  • No direct DOM access           │  │
│  │  • No network access              │  │
│  │  • Only imported functions        │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Input Validation

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn process_input(input: &str) -> Result<String, JsValue> {
    // Validate length
    if input.len() > 10_000 {
        return Err("Input too large".into());
    }

    // Validate content
    if !input.chars().all(|c| c.is_alphanumeric() || c.is_whitespace()) {
        return Err("Invalid characters".into());
    }

    Ok(process_safe(input))
}
```

## CSP Headers

```
Content-Security-Policy:
  script-src 'self' 'wasm-unsafe-eval';
  object-src 'none';
```

## Supply Chain Security

```bash
# Audit dependencies
cargo audit

# Check for known vulnerabilities
cargo deny check

# Verify crate integrity
cargo vet
```

## Cargo.toml Security Settings

```toml
[profile.release]
overflow-checks = true  # Keep integer overflow checks
debug-assertions = false

[dependencies]
# Pin exact versions for security-critical deps
ring = "=0.17.5"
```

## Key Patterns

1. **Never trust JS input** - Validate everything at WASM boundary
2. **Minimize unsafe** - Every unsafe block is a security surface
3. **Use constant-time comparisons** for secrets
4. **Audit dependencies regularly** - cargo audit in CI
5. **Enable overflow checks** in release for security-critical code
