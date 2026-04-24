# Unsafe Strategy

Planning-phase decisions about `unsafe`: is it justified, how isolated, what's the review process, what's the FFI architecture, and what CI hardening to run.

For implementation-side code (raw pointers, `repr(C)`, CString/CStr, bindgen/cbindgen, `AbortIfPanic` guard), see [rust-implementing/ffi-patterns.md](../rust-implementing/ffi-patterns.md). For planning-rules summary, see [rust-planning/SKILL.md §9 Unsafe Strategy](SKILL.md#9-unsafe-strategy).

## Decision 1 — Is unsafe justified at all?

Most Rust code should be 100% safe. `#![forbid(unsafe_code)]` in `lib.rs` is the default stance.

Acceptable reasons for `unsafe`:

- **FFI boundary.** Calling into or being called from C/C++. Unavoidable.
- **Hardware access.** Embedded, memory-mapped I/O, MMIO. Wrap in typed safe abstractions.
- **Measured performance.** Benchmarks show the safe version is insufficient AND the unsafe version is correct. Requires explicit justification in the safety comment.
- **Interop with a safe abstraction that requires it.** E.g., implementing a trait that has unsafe methods (`Send`, `Sync`, `AsRawFd`).
- **Building a safe abstraction.** Sometimes the standard library / a core crate has to be unsafe internally so users don't have to be.

Unacceptable reasons:

- "The borrow checker is annoying." — Restructure ownership. Ask for help in reviews.
- "It's faster" (no benchmarks). — Prove it. Usually the compiler optimizes the safe version to the same code.
- "Other languages do this" — This is Rust; use Rust patterns.
- "Just this once" — A single `unsafe` attracts others. Budget carefully.

## Decision 2 — Unsafe budget and isolation

If unsafe is allowed in this crate:

- **Concentrate it in one module** with a safe public API. `unsafe_impls.rs`, `raw.rs`, `ffi.rs` — one module owns the unsafe surface.
- **Public API is safe.** Callers should never see `unsafe fn` in the public interface. If they must, it's a separate sub-crate or a feature-gated `unsafe-api` module.
- **Every `unsafe` block has `// SAFETY:` comment** explaining why invariants are upheld. No exceptions.
- **Minimize unsafe blocks' size.** One `unsafe { ... }` per logical operation, not one around a whole function.

### `#![forbid(unsafe_code)]` — the strongest isolation

The cleanest pattern: forbid unsafe in the crate entirely and delegate unsafety to dedicated provider crates. rustls does this for a security-critical TLS library:

```rust
// rustls/src/lib.rs — TOP of the crate
#![no_std]
#![forbid(unsafe_code, unused_must_use)]
#![warn(missing_docs, clippy::exhaustive_enums, clippy::exhaustive_structs)]
```

Rustls delegates cryptographic primitives (which do need unsafe for performance / FFI) to provider crates like `aws-lc-rs` and `ring`. The TLS protocol logic stays in safe Rust; the unsafe surface is isolated to externally-audited crypto implementations.

**When this works:**
- Pure algorithmic / protocol / state-machine code
- Library where callers need strong safety guarantees
- Projects where unsafety can be pushed into well-audited providers (crypto, system calls, raw hardware)

The `clippy::exhaustive_enums` + `clippy::exhaustive_structs` pair is worth adopting too in libraries: they demand `#[non_exhaustive]` on every public enum/struct, turning extensibility discipline into a compile-time lint check.

### The invariant contract

Every `unsafe fn` has preconditions that make calling it safe. Document these in `# Safety`:

```rust
/// # Safety
///
/// - `ptr` must be a valid pointer to a null-terminated UTF-8 string.
/// - The string at `ptr` must remain valid for the lifetime `'a`.
/// - `len` must be the byte length excluding the null terminator, and
///   must match the actual length at `ptr`.
pub unsafe fn from_raw_parts<'a>(ptr: *const u8, len: usize) -> &'a str {
    // SAFETY: caller has upheld the invariants above.
    let slice = std::slice::from_raw_parts(ptr, len);
    std::str::from_utf8_unchecked(slice)
}
```

## Decision 3 — FFI architecture

| Direction | Tooling |
|---|---|
| Call C from Rust | `bindgen` — generates Rust bindings from C headers |
| Call Rust from C | `cbindgen` — generates C headers from Rust |
| Call C++ from Rust | `cxx` — safe C++ interop using a shared IDL |
| Embed in BEAM | `rustler` — Erlang/Elixir NIFs |
| Embed in Python | `pyo3` — Python extensions |
| Embed in Java / Android | `jni` |
| Embed in Node | `neon`, `napi-rs` |

### Layout

```
src/
├── lib.rs              (safe public API; no unsafe visible)
├── ffi/
│   ├── mod.rs          (safe wrappers)
│   ├── bindings.rs     (bindgen-generated, #[allow(non_snake_case, ...)])
│   └── extern_c.rs     (#[no_mangle] pub extern "C" fns if exposing to C)
└── core/               (pure Rust logic, testable, no FFI)
```

### FFI-specific unsafe discipline

- **`catch_unwind` at the Rust-to-C boundary.** Unwinding into C is UB. Always wrap `extern "C"` functions' work in `catch_unwind` and convert panics to error returns.

```rust
#[no_mangle]
pub extern "C" fn do_work(input: *const u8, len: usize) -> i32 {
    let result = std::panic::catch_unwind(|| {
        // ... work ...
        0
    });
    result.unwrap_or(-1)  // Panic ⇒ -1, not UB
}
```

- **`AbortIfPanic` guard for critical sections.** When a panic in the middle of an `unsafe` block would leave state inconsistent in ways callers can't recover from, abort instead. rayon uses this:

```rust
struct AbortIfPanic;
impl Drop for AbortIfPanic {
    fn drop(&mut self) {
        std::process::abort();
    }
}

// Inside critical section
let guard = AbortIfPanic;
// ... unsafe work that must not panic-and-unwind ...
std::mem::forget(guard);  // Completed successfully; cancel the abort
```

- **`#[repr(C)]` on types crossing FFI.** Default Rust layout may differ. Use `#[repr(C)]` for structs, `#[repr(u8)]` / `#[repr(i32)]` for enums.

- **CString / CStr.** Null-terminated strings. `CString::new(s)` creates owned; `CStr::from_ptr(p)` borrows (unsafe — you guarantee the pointer is valid).

## Decision 4 — Concurrency + unsafe

- **`Send` and `Sync` are `unsafe` trait impls.** Only implement manually when you've proven thread-safety. Usually use `SyncUnsafeCell` or wrap in `Mutex`.
- **`Pin` for self-referential or must-not-move types.** Essential for futures. See `pin-project-lite` for safe pin projection.
- **Atomic primitives** (`AtomicU64`, etc.) are safe; ordering is a correctness question. Default to `Ordering::SeqCst` unless you've studied the memory model.
- **`UnsafeCell<T>`** is the primitive for interior mutability. Usually wrap in `Mutex`, `RwLock`, or an atomic.

### `bytemuck` for safe transmutation

Many "unsafe" needs in data-processing code are really "I need to reinterpret these bytes as a `T`" — `bytemuck` replaces hand-rolled `std::mem::transmute` with a marker-trait system that's checked at compile time.

```rust
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Rgba { r: u8, g: u8, b: u8, a: u8 }

// Safe: the compiler verified Pod + Zeroable are correctly implementable
let bytes: &[u8] = bytemuck::cast_slice(&rgba_pixels);
let back: &[Rgba] = bytemuck::cast_slice(bytes);

// Zero-init a buffer of Pod type:
let buf = vec![Rgba::zeroed(); 1024];
```

Constraints for `Pod`:
- Type must be inhabited (no `Infallible`)
- All bit patterns must be valid (no `bool`, `char`, `NonZeroU8`)
- No padding bytes

**When to use:** reading file formats, parsing wire protocols, GPU buffer uploads, arrow/columnar data (this is why polars and wgpu both depend on bytemuck). Replaces a lot of `unsafe { std::mem::transmute(...) }` with compile-time-verified safe calls.

## Decision 5 — CI hardening for unsafe crates

Every crate with `unsafe` blocks should have these in CI:

| Tool | What it catches | How |
|---|---|---|
| **Miri** (`cargo +nightly miri test`) | UB: aliasing violations, out-of-bounds, use-after-free, data races in single-threaded code | Slow (10-50x); valuable. Run on unit tests. |
| **AddressSanitizer** (`RUSTFLAGS="-Zsanitizer=address"`) | Heap UB, use-after-free, when linking C | Run integration tests |
| **ThreadSanitizer** (`-Zsanitizer=thread`) | Data races in multithreaded code | Run concurrent tests |
| **MemorySanitizer** (`-Zsanitizer=memory`) | Reads of uninitialized memory | Less common; when linking C |
| **Loom** (`loom::model::test`) | Concurrency bugs in lock-free code via model checking | For lock-free / atomic-heavy code |
| **cargo-fuzz** | Crashes and UB on untrusted input | For parsers, deserializers, format handlers |

### Example CI excerpt

```yaml
jobs:
  miri:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup toolchain install nightly --component miri
      - run: cargo +nightly miri test
  
  sanitizer:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: |
          RUSTFLAGS="-Zsanitizer=address" \
          RUSTDOCFLAGS="-Zsanitizer=address" \
            cargo +nightly test --target x86_64-unknown-linux-gnu
```

## Decision 6 — Unsafe review checklist (planning-time commitment)

If unsafe is in the crate, commit at planning time to reviewing every `unsafe` line. Pair-review unsafe additions. See [rust-reviewing/SKILL.md §7.6](../rust-reviewing/SKILL.md#76-unsafe-and-ffi) for the per-PR checklist.

## Related

- [rust-implementing/ffi-patterns.md](../rust-implementing/ffi-patterns.md) — implementation: raw pointer arithmetic, repr(C), bindgen/cbindgen, CString/CStr patterns, network protocol parsing
- [rust-planning/SKILL.md §9](SKILL.md#9-unsafe-strategy) — planning rules summary
- [rust-reviewing/SKILL.md §7.6](../rust-reviewing/SKILL.md#76-unsafe-and-ffi) — unsafe review checklist
