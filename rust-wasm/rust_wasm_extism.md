# Rust → Extism Plugins

Authoring Rust plugins for the [Extism](https://extism.org) WASM plugin
runtime, using the `extism-pdk` crate. Plugins compile to a single
`.wasm` file that any Extism host SDK (Elixir, Python, Go, Node, Ruby,
PHP, C#, .NET, browser, …) can load and call.

> **Verified against:** `extism-pdk` v1.4.1 (May 2025). The PDK is on
> Rust 2024 edition. If macro behaviour shifts in a later release, the
> trait/macro names below should still hold — the convert layer is
> stable since v1.0.

> **Different from raw `wasm-bindgen`:** `wasm-bindgen` targets JS/web —
> it generates JS glue and uses the `wasm32-unknown-unknown` target with
> JS-side bindings. `extism-pdk` targets the Extism runtime — any host
> language with an Extism SDK can call the plugin via a uniform
> ABI (memory + function-name lookup). Choose `extism-pdk` when the
> host is NOT JS, or when you want one plugin reused across multiple
> host languages.

## Rules for Writing Extism Rust Plugins (LLM)

1. **ALWAYS set `crate-type = ["cdylib"]` in `Cargo.toml`** — without it,
   `cargo build` produces a `.rlib` (Rust static lib) instead of a
   `.wasm` and no exports are generated.
2. **ALWAYS use `#[plugin_fn]`** on every function the host should be
   able to call. Without the macro, the function name isn't exported
   in a way Extism's lookup can find.
3. **ALWAYS return `FnResult<T>`** from `#[plugin_fn]` functions. The
   PDK uses the `?` operator inside; bare `Result` won't compose with
   the macro's error-encoding shim.
4. **NEVER `unwrap()` / `expect()` / `panic!()` inside a plugin**
   without a deliberate justification. A panic crosses the WASM
   boundary as a runtime trap and the host gets a generic `{:error, _}`
   with no message. Return `Err(...)?` with a real message so the host
   can log it. `expect("invariant: ...")` is OK for provably-infallible
   serialization of primitives.
5. **ALWAYS encode structured input/output via `Json<T>` / `Msgpack<T>`**
   when the data is non-trivial. The wrapper round-trips serde
   automatically and signals the host SDK what encoding to use. Raw
   `String` / `Vec<u8>` is fine for one-shot primitives.
6. **NEVER spawn threads from a plugin** — the Extism runtime is
   single-threaded inside one `Plugin.call`. Concurrency comes from
   the host instantiating multiple plugins or making multiple calls
   from separate host processes. `std::thread::spawn` will fail to
   compile on `wasm32-unknown-unknown` anyway.
7. **ALWAYS pick the right target:** `wasm32-unknown-unknown` (default,
   smaller, no syscalls) vs `wasm32-wasip1` (when you need `File::*`,
   environment variables, `std::time::SystemTime::now`, etc.). The host
   must enable WASI to load a wasip1 plugin — match the build target
   to the host's loader config.
8. **ALWAYS use `info!` / `warn!` / `error!` from `extism_pdk` for
   logging** — `println!` doesn't reach the host on the default
   `wasm32-unknown-unknown` target. The PDK macros funnel into Extism's
   log file (host sets it via the SDK).
9. **NEVER assume the host has implemented host functions you import
   via `#[host_fn]`.** Some host SDKs don't expose them yet (notably
   the Elixir SDK v1.0). A plugin that imports an absent host function
   fails to load. Make host imports optional via `cfg` or design
   plugins that don't need callbacks.
10. **ALWAYS optimize the binary in production** — `wasm-opt -Oz` plus
    `cargo build --release --target wasm32-unknown-unknown` typically
    shrinks output 2–5×. Big plugins slow host load times noticeably.
11. **NEVER use `std::fs` or `std::env` on the `wasm32-unknown-unknown`
    target** — they're not implemented and will trap. If you need
    them, target `wasm32-wasip1` and ensure the host enables WASI.
12. **ALWAYS test plugins via a host harness, not standalone.** A
    `.wasm` binary has no `main` — there's nothing to run. Either
    invoke from the host's SDK in an integration test, or use the
    Extism CLI: `extism call my_plugin.wasm count_vowels --input "hi"`.

## Cargo.toml — the canonical shape

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
extism-pdk = "1.4"
serde = { version = "1", features = ["derive"] }

# For Msgpack / Prost / Base64 encodings, add the relevant feature:
# extism-pdk = { version = "1.4", features = ["msgpack"] }
```

```bash
# Install the wasm target (one-time per machine):
rustup target add wasm32-unknown-unknown

# For WASI plugins:
rustup target add wasm32-wasip1

# Build:
cargo build --release --target wasm32-unknown-unknown
# or
cargo build --release --target wasm32-wasip1
```

Output lands at `target/<TARGET>/release/my_plugin.wasm`.

## The canonical hello-world

```rust
// src/lib.rs
#![no_main]

use extism_pdk::*;

#[plugin_fn]
pub fn greet(name: String) -> FnResult<String> {
    Ok(format!("Hello, {}!", name))
}
```

Compile to wasm, hand the binary to the host. The host calls the export
by name (`"greet"`) with a string input and gets a string back.

## Decision Tables

### Input/output encoding — which wrapper?

| Data shape | Plugin signature | Host sends/receives |
|---|---|---|
| Single string | `pub fn f(s: String) -> FnResult<String>` | Raw binary, decoded as UTF-8 |
| Raw bytes | `pub fn f(b: Vec<u8>) -> FnResult<Vec<u8>>` | Raw binary, no decoding |
| Structured (JSON, common case) | `pub fn f(Json(req): Json<Req>) -> FnResult<Json<Resp>>` | JSON-encoded binary |
| Structured (Msgpack — smaller, faster) | `pub fn f(Msgpack(req): Msgpack<Req>) -> FnResult<Msgpack<Resp>>` | Msgpack-encoded binary |
| Protobuf (gRPC-adjacent) | `pub fn f(Prost(req): Prost<Req>) -> FnResult<Prost<Resp>>` | Protobuf wire format |
| Base64 binary (host can't handle raw) | `pub fn f(Base64(b): Base64) -> FnResult<Base64>` | Base64-encoded string |

The wrapper IS the encoding contract. Both sides must agree — if the
host SDK does `Jason.encode!/1` on input and the plugin signature takes
`Msgpack<T>`, decoding fails.

### Error encoding

| Failure type | Return shape | Host sees |
|---|---|---|
| Expected business error | `Err(Error::msg("invalid input"))?` | `{:error, "invalid input"}` |
| Host-recoverable error | `Err(WithReturnCode::new(error, 42))?` | `{:error, _, return_code: 42}` (SDKs vary) |
| Programmer error / panic | `panic!("bug: ...")` | Generic trap, host gets `{:error, _}` with no detail |
| Impossible-here invariant | `.expect("invariant: bytes already validated")` | Same as panic — only OK when provably unreachable |

### `wasm32-unknown-unknown` vs `wasm32-wasip1`

| Plugin needs… | Target |
|---|---|
| Pure compute (parse, transform, validate) | `wasm32-unknown-unknown` |
| Random numbers | `wasm32-wasip1` (host provides) — OR use `extism_pdk::http::request` for an entropy source if you have one |
| Current time | `wasm32-wasip1` — `std::time::SystemTime::now()` works there |
| Read a file from disk | `wasm32-wasip1` + host's `allowed_paths` mounting that path |
| Environment vars | `wasm32-wasip1` + host's `config` or `allowed_envs` |
| Network access | Neither — use Extism HTTP (`extism_pdk::http::request`) with host's `allowed_hosts` allowlist |
| Spawn threads | Not supported in WASM — restructure to single-threaded |

The target is a Cargo flag, not a `Cargo.toml` switch. Add a target
key in `.cargo/config.toml` to make `cargo build` default to wasm:

```toml
# .cargo/config.toml
[build]
target = "wasm32-unknown-unknown"
```

## Patterns

### Structured request/response with `Json<T>`

```rust
#![no_main]

use extism_pdk::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct AddRequest {
    a: i64,
    b: i64,
}

#[derive(Serialize)]
struct AddResponse {
    sum: i64,
}

#[plugin_fn]
pub fn add(Json(req): Json<AddRequest>) -> FnResult<Json<AddResponse>> {
    Ok(Json(AddResponse { sum: req.a + req.b }))
}
```

Host call:

```elixir
{:ok, json} = Extism.Plugin.call(plugin, "add", Jason.encode!(%{a: 2, b: 3}))
{:ok, %{"sum" => 5}} = Jason.decode(json)
```

### Output via `ToBytes` derive (alternative to `Json<T>`)

When the response type also needs to be `Serialize` for use outside
the plugin, use the convert-crate derives directly:

```rust
#![no_main]

use extism_pdk::*;
use serde::Serialize;

#[derive(Serialize, ToBytes)]
#[encoding(Json)]
struct VowelCount {
    count: i32,
    vowels: &'static str,
}

#[plugin_fn]
pub fn count_vowels(input: String) -> FnResult<VowelCount> {
    let count = input.chars().filter(|c| "aeiouAEIOU".contains(*c)).count() as i32;
    Ok(VowelCount { count, vowels: "aeiouAEIOU" })
}
```

This is the shape used in the official `extism/rust-pdk` example
[`examples/count_vowels.rs`](https://github.com/extism/rust-pdk/blob/main/examples/count_vowels.rs).

### Plugin variables — state across calls

Variables persist across `Plugin.call` invocations on the same plugin
instance. Useful for caches, counters, accumulated state.

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn increment(_: ()) -> FnResult<i64> {
    let current = var::get::<i64>("counter")?.unwrap_or(0);
    let next = current + 1;
    var::set("counter", next)?;
    Ok(next)
}
```

The host's manifest sets `memory.max_var_bytes` to cap total variable
storage per plugin — exceeding it causes `var::set` to fail. Defaults
to 4096 bytes; raise via the manifest's `memory` block when needed.

### Plugin configuration — read-only from host

Configuration is set in the host's manifest under `config:`, immutable
from the plugin's side:

```elixir
# Host (Elixir):
manifest = %{
  wasm: [%{path: "priv/plugins/x.wasm"}],
  config: %{"api_key" => "secret", "max_iterations" => "100"}
}
```

```rust
// Plugin (Rust):
#[plugin_fn]
pub fn run(_: ()) -> FnResult<String> {
    let api_key = config::get("api_key")?
        .ok_or_else(|| Error::msg("config: api_key missing"))?;
    let max = config::get("max_iterations")?
        .map(|v| v.parse().unwrap_or(10))
        .unwrap_or(10);

    // ... use api_key and max ...

    Ok(format!("ran {} iterations", max))
}
```

**Config values are always strings.** Numbers, booleans, JSON — all
encoded as strings, parsed by the plugin. The host's manifest map
serializes via `Map.new(String.Chars)`.

### Logging from plugins

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    info!("plugin received {} bytes", input.len());

    if input.len() > 1_000_000 {
        warn!("input is suspiciously large: {}", input.len());
    }

    Ok(input.to_uppercase())
}
```

The host receives these via Extism's log file. From Elixir, the host
opens it with `Extism.set_log_file("/tmp/extism.log", "info")` before
loading plugins.

### Calling host functions (when the host supports them)

```rust
#![no_main]

use extism_pdk::*;

#[host_fn]
extern "ExtismHost" {
    fn fetch_user_name(user_id: i64) -> String;
}

#[plugin_fn]
pub fn greet_user(user_id: i64) -> FnResult<String> {
    // host_fn calls are `unsafe` — the boundary is opaque to Rust's
    // type system. Wrap in `unsafe { ... }` and propagate `?`.
    let name = unsafe { fetch_user_name(user_id)? };
    Ok(format!("Hello, {}!", name))
}
```

**Compatibility caveat:** the Elixir host SDK (v1.0) **does not support
host functions yet** (per the `extism/elixir-sdk` README). A plugin
with `#[host_fn]` imports will fail to load. Other host SDKs (Python,
Go, Rust-as-host) do support them. Make host functions optional via
`#[cfg(feature = "host_fns")]` if you target multiple host SDKs.

### HTTP from plugins — Extism's own client

Plugins can make outbound HTTP calls via Extism's HTTP API (NOT
`reqwest` — that won't compile to wasm32 without significant work).
The host's manifest gates allowed hosts:

```elixir
manifest = %{
  wasm: [%{path: "priv/plugins/fetcher.wasm"}],
  allowed_hosts: ["api.example.com", "*.cdn.example.com"]
}
```

```rust
use extism_pdk::*;

#[plugin_fn]
pub fn fetch_user(user_id: String) -> FnResult<String> {
    let req = HttpRequest::new("https://api.example.com/users")
        .with_method("GET")
        .with_header("Authorization", "Bearer ...");

    let resp = http::request::<()>(&req, None)?;

    Ok(String::from_utf8(resp.body())?)
}
```

The host's `memory.max_http_response_bytes` caps response size. Without
the host's `allowed_hosts` allowlist, the call fails.

## Anti-patterns (BAD/GOOD)

**`unwrap()` on plugin input:**

```rust
// BAD — input is host-supplied; unwrap = panic = generic trap with no
// host-visible reason
#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    let parsed: serde_json::Value = serde_json::from_str(&input).unwrap();
    Ok(parsed["field"].to_string())
}

// GOOD — propagate via `?` so host sees the real error
#[plugin_fn]
pub fn run(Json(req): Json<MyRequest>) -> FnResult<String> {
    Ok(format!("got {:?}", req.field))
}

// GOOD (if you must hand-decode) — Error::msg with real context
#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    let parsed: serde_json::Value = serde_json::from_str(&input)
        .map_err(|e| Error::msg(format!("invalid JSON input: {}", e)))?;
    Ok(parsed.to_string())
}
```

**Bare `Result` instead of `FnResult`:**

```rust
// BAD — won't compose with `#[plugin_fn]` macro
#[plugin_fn]
pub fn run(input: String) -> Result<String, std::io::Error> {
    Ok(input)
}

// GOOD — FnResult uses an erased Error type that the PDK encodes
// across the WASM boundary
#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    Ok(input)
}
```

**Missing `crate-type = ["cdylib"]`:**

```toml
# BAD — produces a .rlib, no wasm symbols exported
[lib]
# (default: rlib)

# GOOD — cdylib produces the .wasm artifact the host can load
[lib]
crate-type = ["cdylib"]
```

**Spawning threads:**

```rust
// BAD — std::thread doesn't compile on wasm32-unknown-unknown
#[plugin_fn]
pub fn parallel(_: ()) -> FnResult<()> {
    std::thread::spawn(|| do_work());
    Ok(())
}

// GOOD — concurrency lives on the host side; plugins are single-threaded
#[plugin_fn]
pub fn work_chunk(Json(req): Json<ChunkRequest>) -> FnResult<Json<ChunkResult>> {
    Ok(Json(process_chunk(req)))
}
// Host then calls `work_chunk` N times concurrently from N host processes.
```

**`println!` for logging:**

```rust
// BAD — wasm32-unknown-unknown has no stdout; println! is a no-op
// (or worse, traps depending on runtime)
#[plugin_fn]
pub fn run(_: ()) -> FnResult<()> {
    println!("processing started");
    Ok(())
}

// GOOD — Extism PDK macros funnel to the host's log file
#[plugin_fn]
pub fn run(_: ()) -> FnResult<()> {
    info!("processing started");
    Ok(())
}
```

**Importing `#[host_fn]` without checking host support:**

```rust
// BAD — fails to LOAD on Elixir host (v1.0 doesn't support host_fn)
#[host_fn]
extern "ExtismHost" {
    fn elixir_callback(s: String) -> String;
}

#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    Ok(unsafe { elixir_callback(input)? })
}

// GOOD — feature-gate host functions; ship a fallback for hosts that
// don't support them
#[cfg(feature = "host_fns")]
#[host_fn]
extern "ExtismHost" {
    fn host_callback(s: String) -> String;
}

#[plugin_fn]
pub fn run(input: String) -> FnResult<String> {
    #[cfg(feature = "host_fns")]
    {
        Ok(unsafe { host_callback(input)? })
    }
    #[cfg(not(feature = "host_fns"))]
    {
        Ok(input.to_uppercase()) // sensible default
    }
}
```

## Testing plugins

A `.wasm` binary has no `main` — running it standalone does nothing.
Three viable test approaches:

### 1. Pure Rust unit tests for non-plugin functions

Extract the plugin's logic into pure functions; test them with normal
`#[cfg(test)] mod tests`. The `#[plugin_fn]` wrapper is just a thin
adapter — what matters is the logic underneath.

```rust
fn count_vowels_inner(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

#[plugin_fn]
pub fn count_vowels(input: String) -> FnResult<Json<i32>> {
    Ok(Json(count_vowels_inner(&input) as i32))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_basic() {
        assert_eq!(count_vowels_inner("hello"), 2);
    }
}
```

Run with `cargo test` on the **default host target** (NOT
`wasm32-unknown-unknown`) — `cargo test --target wasm32-...` won't find
a test runner.

### 2. The Extism CLI

```bash
extism call target/wasm32-unknown-unknown/release/my_plugin.wasm \
    count_vowels --input "hello world"
# => {"count":3}
```

Useful for smoke-testing the WASM output without setting up a host
harness.

### 3. Host integration test

A Rust integration test that loads the plugin via `extism` (Rust host
SDK) and calls into it. The `extism` crate's `Plugin::new` mirrors the
Elixir SDK's API:

```rust
// tests/plugin_integration.rs
use extism::{Manifest, Plugin, Wasm};

#[test]
fn count_vowels_works() {
    let wasm = Wasm::file("target/wasm32-unknown-unknown/release/my_plugin.wasm");
    let manifest = Manifest::new([wasm]);
    let mut plugin = Plugin::new(&manifest, [], false).unwrap();
    let out = plugin.call::<&str, &str>("count_vowels", "hello").unwrap();
    assert!(out.contains(r#""count":2"#));
}
```

Requires building the plugin first; chain via a `cargo make` task or
a shell script.

## Common pitfalls

- **`println!` is silent on wasm32-unknown-unknown.** Use `info!` /
  `warn!` / `error!` from `extism_pdk`. The output goes to the host's
  log file, set via the host SDK.
- **`SystemTime::now()` traps on wasm32-unknown-unknown.** Either
  switch to `wasm32-wasip1` (host provides time) or accept a timestamp
  from the host as input.
- **`reqwest` / `hyper` / `ureq` don't compile to wasm32.** Use
  `extism_pdk::http::request` with host-managed `allowed_hosts`.
- **The `.wasm` artifact path varies by target.** `wasm32-unknown-unknown`
  produces `target/wasm32-unknown-unknown/release/<name>.wasm`;
  `wasm32-wasip1` produces `target/wasm32-wasip1/release/<name>.wasm`.
  Scripts that copy the artifact need to match the configured target.
- **`#![no_main]` is required at the top of `src/lib.rs`.** Plugins
  don't have a Rust `main`; the host calls exports directly. Forgetting
  this gives a confusing linker error.
- **Build artifacts grow.** A release plugin is typically 50–300 KB
  for simple compute, but pulling in `serde_json` + heavy deps can push
  it past 1 MB. Run `wasm-opt -Oz` and `cargo build --release` together
  for production. Strip debug info via `[profile.release] strip = true`.
- **Plugins shouldn't allocate huge amounts of memory.** The host's
  `memory.max_pages` (default 4 pages = 256 KiB) gates plugin memory.
  Big plugins need a manifest bump on the host side — coordinate.

## Related

- **[rust_wasm_core.md](rust_wasm_core.md)** — wasm targets, toolchain
  setup, build pipeline. Key: extism plugins ship as `.wasm` artifacts;
  same toolchain (`rustup target add ...`).
- **[rust_wasm_security.md](rust_wasm_security.md)** — sandboxing
  guarantees, memory safety in WASM. Key: Extism inherits WASM's
  sandboxing — no syscalls without WASI + host opt-in.
- **[../extism-elixir/SKILL.md](../extism-elixir/SKILL.md)** — the host
  side. Key: Elixir SDK v1.0 doesn't support host functions yet; design
  plugins around pure transformation in/out.
- **[../rust-nif/SKILL.md](../rust-nif/SKILL.md)** — alternative path
  when you don't need user-supplied plugins. Key: Rustler is faster but
  trusts the code; Extism sandboxes everything.
