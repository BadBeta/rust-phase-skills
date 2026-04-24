# Cargo Workspace Layout and Feature Flag Architecture

Planning-phase reference for structuring Cargo projects: single crate → lib+bin → workspace. Covers `[workspace.dependencies]`, `[workspace.lints]`, feature flag architecture, and feature-gated server roles.

For the architectural principles behind workspace layout (dependency direction, trait placement), see [architecture-patterns.md](architecture-patterns.md). For the matching implementation-side code patterns (`[[bin]]`, cargo commands, visibility modifiers), see [rust-implementing/SKILL.md §Modules & Cargo](../rust-implementing/SKILL.md#modules--cargo).

## When to promote to a Cargo workspace

A workspace is warranted when ANY of:

- **Crate-level dependency enforcement needed.** You want it structurally impossible for the domain crate to import `sqlx` or `axum`. A workspace makes this enforceable by `cargo check`.
- **Multiple binaries with different feature sets.** `my-app-server` uses `postgres` + `redis`; `my-app-cli` doesn't need either. Feature-gated binaries in a workspace avoid pulling in unneeded deps at compile time.
- **Multi-team ownership.** Clear ownership boundaries reduce merge conflicts and enable parallel development.
- **Independent publishing.** One or more crates will be published to crates.io (or an internal registry) separately.
- **Incremental compile time.** A mega-crate of 100K+ lines rebuilds everything on any change. Splitting into crates gives dependency-aware incremental compilation.
- **Plugin architecture.** Core crate + multiple optional adapter crates, each its own published unit.

A workspace is NOT warranted when:

- **"Feels big."** Use modules.
- **"Better separation of concerns."** Modules + `pub(crate)` give that.
- **Solo project, one binary, < 10K lines.** Overhead not yet justified.

## Workspace root `Cargo.toml`

```toml
[workspace]
members = ["crates/*"]
resolver = "2"   # REQUIRED for workspaces — enables v2 feature resolution

# Centralized version pinning for all shared deps
[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
axum = "0.8"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }

# Workspace-wide lint configuration (axum pattern, stable since Rust 1.74)
[workspace.lints.rust]
missing_docs = "warn"
missing_debug_implementations = "warn"
unreachable_pub = "warn"
unsafe_op_in_unsafe_fn = "deny"

[workspace.lints.clippy]
dbg_macro = "warn"
print_stdout = "warn"
needless_pass_by_value = "warn"
unwrap_used = "warn"         # Catch unwraps; override per-module where needed
# Allow type_complexity — generic-heavy frameworks like axum need this
type_complexity = "allow"
# Aggressive project example: nushell sets `unwrap_used = "deny"` at workspace
# level (CI fails on any unwrap). This pairs with #[cfg(test)] overrides.
# Library/no_std example (rustls): enforces no_std discipline at workspace level
# alloc_instead_of_core = "warn"    # bans `alloc::Vec` in favor of `alloc::`-free core
# std_instead_of_core = "warn"       # bans `std::` imports in no_std crates
# clippy::exhaustive_enums = "warn"  # demands #[non_exhaustive] on public enums
# clippy::exhaustive_structs = "warn" # demands #[non_exhaustive] on public structs
# CLI-app output boundary (cargo pattern): route all user output through a
# formatter layer rather than println!/eprintln!.
# print_stdout = "warn"
# print_stderr = "warn"
# Game/engine example (Bevy): deny unsafe workspace-wide with per-crate opt-in
# (bevy uses `[workspace.lints.rust] unsafe_code = "deny"`, then individual
# crates add `#[allow(unsafe_code)]` where needed. Different from rustls's
# per-crate `#![forbid(unsafe_code)]` — deny+allow is escapable, forbid is not.)
# Graded-severity pattern (rust-analyzer): promote clippy categories to
# different severity levels rather than hand-picking individual lints. This
# is the fourth lint-strategy pattern alongside nushell's "aggressive deny",
# rustls's "extensive curated warn", and cargo's "all=allow + correctness=warn
# + specific denies":
#   [workspace.lints.clippy]
#   correctness = { level = "deny", priority = -1 }
#   perf = { level = "deny", priority = -1 }
#   style = { level = "warn", priority = -1 }
#   suspicious = { level = "warn", priority = -1 }
#   restriction = { level = "allow", priority = -1 }
#   # then hand-pick specific restriction lints back up to warn/deny
# Kernel/safety-critical variant (Redox kernel): emphasizes panic prevention
# and overflow:
#   arithmetic_side_effects = "warn"  # flag integer overflow risk
#   indexing_slicing = "warn"          # flag [i] that can panic — use .get()
#   unwrap_used = "warn"               # require expect() with rationale
#   not_unsafe_ptr_arg_deref = "deny"  # fn taking *ptr must be unsafe fn

# Workspace may patch itself (rustls pattern) — ensures downstream ecosystem
# crates that depend on `rustls` via crates.io actually use THIS workspace's
# local copy during development.
[patch.crates-io]
# rustls = { path = "rustls" }

# Common profiles
[profile.release]
lto = "thin"                  # Fast LTO by default
debug = 1                     # Line-table debug info for backtraces
codegen-units = 16

# Some projects keep FULL debug symbols in release (debug = 2). Example:
# rust-postgres / tokio-postgres sets [profile.release] debug = 2 so that
# panic backtraces and production core-dumps contain rich local-variable
# info. Trade-off: larger binaries (sometimes 3-10x). Worth it for
# long-running services where post-hoc debugging of production issues is
# a requirement.

# Rust-analyzer pattern: `dev-rel` profile that inherits from release but
# keeps full debug symbols, for use when debugging optimized code:
#   [profile.dev-rel]
#   inherits = "release"
#   debug = 2
# Invoke with: cargo build --profile dev-rel

# Per-dependency opt-level in dev (rust-analyzer, also common elsewhere):
# speed up dev builds by optimizing hot-path dependencies even in dev mode:
#   [profile.dev.package.rowan]     opt-level = 3
#   [profile.dev.package.rustc-hash] opt-level = 3
#   [profile.dev.package.smol_str]  opt-level = 3
#   [profile.dev.package.salsa]     opt-level = 3
# Or the blanket form covering all deps (seen in many projects):
#   [profile.dev.package."*"]       opt-level = 3
# Kernel pattern: `panic = "abort"` is required (not optional) in bare-metal
# / no_std environments without unwinding support:
#   [profile.release]  panic = "abort"
#   [profile.dev]      panic = "abort"

# Packaging profile — slower compile, smallest+fastest binary
[profile.release-lto]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
debug = false
debug-assertions = false
overflow-checks = false
```

## Member crate `Cargo.toml`

```toml
# crates/domain/Cargo.toml
[package]
name = "my-app-domain"
version = "0.1.0"
edition = "2024"

[dependencies]
# Inherits from workspace — no version duplication
serde = { workspace = true }
thiserror = { workspace = true }
uuid = { version = "1", features = ["v4", "serde"] }

# Domain crate has ZERO framework dependencies — this is the boundary proof
# NO sqlx, NO axum, NO reqwest, NO redis
# If you need to add one here, the architecture has a boundary problem.

[lints]
workspace = true   # Inherit the workspace lint configuration

[dev-dependencies]
mockall = { workspace = true }
proptest = { workspace = true }
```

```toml
# crates/infra/Cargo.toml
[package]
name = "my-app-infra"
version = "0.1.0"
edition = "2024"

[dependencies]
my-app-domain = { path = "../domain" }      # depends on domain
my-app-app = { path = "../app" }            # and application

# Infrastructure dependencies live HERE
sqlx = { workspace = true }
redis = "0.26"
reqwest = { version = "0.12", features = ["json"] }

[lints]
workspace = true
```

## Trait placement in workspaces

**The trait lives in the crate that USES it, not the crate that implements it.**

```
crates/
├── domain/
│   └── src/
│       ├── order.rs        # entity Order
│       └── ports.rs        # pub trait OrderRepository (USED by app, IMPLEMENTED by infra)
├── app/
│   └── src/
│       └── use_cases.rs    # struct PlaceOrderUseCase<R: OrderRepository>
└── infra/
    └── src/
        └── postgres.rs     # struct PgOrderRepository impl OrderRepository
```

The trait is in `domain` because the domain defines the contract. `infra` depends on `domain` and implements the contract.

## Feature flag architecture

Feature flags are a **compile-time architectural mechanism.** They let you swap adapters, enable optional subsystems, and gate infrastructure without runtime cost.

### Patterns

**Swap an adapter:**
```toml
[features]
default = ["postgres"]
postgres = ["dep:sqlx-postgres"]
mysql = ["dep:sqlx-mysql"]
# The consumer picks exactly one at compile time
```

**Optional subsystem:**
```toml
[features]
default = []
metrics = ["dep:prometheus"]
tracing-otel = ["dep:opentelemetry"]
# Not enabled by default — users opt in
```

**Feature-gated binary (server roles):**
```toml
[package]
name = "my-app"

[[bin]]
name = "api-server"
required-features = ["api"]

[[bin]]
name = "worker"
required-features = ["worker"]

[features]
default = []
api = ["dep:axum", "dep:sqlx"]
worker = ["dep:sqlx", "dep:lapin"]
full = ["api", "worker"]
```

Build one specific binary:
```sh
cargo build --bin api-server --features api --no-default-features
cargo build --bin worker --features worker --no-default-features
```

**Pure-Rust vs C-binding dual stack (image-rs pattern):**

When a library's function can be implemented in pure Rust (safer, smaller dep graph) AND via a mature C library (faster, more features), offer both and let users pick via features:

```toml
[features]
default = ["ravif"]         # pure-Rust AVIF encoder/decoder
ravif = ["dep:ravif"]
avif-native = ["dep:dav1d"] # C-based AVIF via libdav1d
```

`image-rs/image` defaults to pure-Rust decoders across almost every format (`zune-jpeg`, `png`, `gif`, `qoi`, `ravif`) with C-backed options as opt-in features. Users weigh pure-Rust safety vs C performance per format.

**Feature composition for compound capabilities:**

When one feature implies another, express the dependency in the feature list:

```toml
[features]
bmp = []
png = ["dep:png"]
ico = ["bmp", "png"]    # ICO format needs BMP + PNG to decode its embedded sub-images
```

This keeps the user's mental model simple ("enable `ico` to get ICO support") while the crate internally composes the required primitive features.

**Feature-as-license-decision (ffmpeg-next pattern):**

When a library has components under different licenses, gate them behind features so users must explicitly opt in:

```toml
[features]
default = ["codec", "format", "filter"]
# Licensing variants — user must explicitly opt into GPL-linked components
gpl = []               # enable GPL-only FFmpeg components (libx264, libx265)
nonfree = []           # enable non-free FFmpeg components (fdk-aac, nvenc)
v3 = []                # use LGPL v3 rather than v2.1
```

ffmpeg-next does this for GPL, nonfree, and LGPL-v3-vs-v2.1. The feature isn't about functionality — it's about license commitment. Compliance is now an explicit build-time decision rather than a hidden transitive consequence.

**Massive feature surface for component libraries (ffmpeg-next):**

Some wrappers over large C libraries expose 30+ features — one per codec, one per filter, one per protocol. This is appropriate when:
- The underlying library has independent optional components
- Users typically want only a subset
- Binary size / license / CVE exposure all benefit from explicit opt-in

The counter-pattern (single `full` feature) pulls in everything including vulnerabilities in components you don't use.

**Orthogonal-axis features (tokio-modbus pattern):**

When a protocol or library has multiple independent dimensions — transport × mode × sync-vs-async, or backend × logger × allocator — organize features along orthogonal axes rather than hierarchical tiers:

```toml
[features]
# Transport axis (Modbus variants)
rtu = ["dep:tokio-serial"]
tcp = []
rtu-over-tcp-server = ["rtu", "tcp"]

# Mode axis (sync wrappers, server support)
sync = []
rtu-sync = ["rtu", "sync"]
tcp-sync = ["tcp", "sync"]
server = ["dep:socket2"]
rtu-server = ["rtu", "server"]
tcp-server = ["tcp", "server"]
```

The named features (`rtu-sync`, `tcp-server`, ...) compose the two axes (transport + mode). Users enable what they need; internal base features (`sync`, `server`) are shared implementation. This gives 8 meaningful configurations with 3 axis-aware features.

**When this pattern fits:** axes are genuinely independent, user may want any combination, no axis is "optional extension" of another.

**Tiered feature architecture (nushell pattern):**

When a large application has many optional capabilities, tier them:

```toml
[features]
default = ["plugin", "trash", "sqlite", "network", "rustls-tls", "mcp"]
stable = ["default"]                # alias of default for consumers pinning "stability"
full = [                            # everything non-mutually-exclusive
    "default",
    "dataframe",
    "experimental-profile-runner",
    # ...
]

plugin = ["dep:nu-plugin-core"]
trash = ["dep:trash"]
# ... per-capability features
```

Benefits:
- Users pick a tier (`default`, `full`) without enumerating individual features
- `stable` alias signals stability commitment separately from feature composition
- Individual features remain granular for build-size tuning

**Platform-selection via mutually-exclusive features (embassy pattern):**

For code that must compile differently per architecture or chip, use feature flags as compile-time selectors. Embassy organizes architecture selection this way:

```toml
[features]
# Exactly one of these should be enabled
platform-cortex-m = ["dep:cortex-m"]
platform-riscv32 = ["dep:riscv"]
platform-avr = ["dep:portable-atomic"]
platform-wasm = []
platform-std = []
# ... plus executor mode selection:
executor-thread = []
executor-interrupt = []
```

For chip-level selection within a single architecture, embassy-rp uses:

```toml
[features]
rp2040 = ["_rp2040"]
rp235xa = ["_rp235x"]
rp235xb = ["_rp235x"]
_rp2040 = []   # internal, implementation detail
_rp235x = []
```

The `_prefix` convention marks internal features that users shouldn't enable directly — they're implementation shared between public chip features.

**Hardware-variant features**: same repo also exposes `W25Q080`, `GD25Q64C`, etc. — different flash chips need different bootloaders. Board-level BSP crates pick exactly one. This is the "feature-as-hardware-selector" pattern, common wherever the same HAL crate targets boards with different silicon.

**Facade crate with feature-gated subcrates (ripgrep pattern):**
```toml
# grep/Cargo.toml — single entry point re-exporting subcrates
[dependencies]
grep-cli = { version = "0.2", path = "../grep-cli" }
grep-matcher = { version = "0.2", path = "../grep-matcher" }
grep-printer = { version = "0.2", path = "../grep-printer" }
grep-pcre2 = { version = "0.2", path = "../grep-pcre2", optional = true }

[features]
pcre2 = ["dep:grep-pcre2"]
```

```rust
// grep/src/lib.rs
pub use grep_cli as cli;
pub use grep_matcher as matcher;
pub use grep_printer as printer;
#[cfg(feature = "pcre2")]
pub use grep_pcre2 as pcre2;
```

### CI build matrix via `[package.metadata.*]` (embassy pattern)

For cross-platform libraries that need to verify every combination of target triples × feature flags × chip variants, encode the build matrix as Cargo.toml metadata rather than scattered CI scripts:

```toml
[package.metadata.embassy]
# CI reads this to generate a multi-architecture build/test matrix
targets = ["thumbv7em-none-eabi", "riscv32imac-unknown-none-elf", "wasm32-unknown-unknown"]
features = [
  ["platform-cortex-m", "executor-thread"],
  ["platform-riscv32", "executor-interrupt"],
  # ... 40+ combinations total
]
```

Keeps the source of truth for "what must compile" inside the Cargo.toml alongside the features themselves. CI tooling consumes the metadata to generate the matrix.

### Anti-patterns

- **`#[cfg(feature = "...")]` scattered through domain logic.** Feature gates belong at the composition root, selecting trait implementations. Domain code should compile identically regardless of features.
- **Feature gates that change public API shape.** Downstream users can't reliably depend on a shape that appears/disappears.
- **Too-granular features.** `feature = "serde-derive"` + `feature = "serde-json"` + `feature = "serde-toml"` is hard to document; consolidate to `feature = "serialization"`.
- **Default features that cascade broadly.** A single crate enabling `"full"` shouldn't pull in 50 transitive deps for users who want a small footprint.

## Visibility boundaries

Within a crate:
- `pub` — crate's public API
- `pub(crate)` — visible within the crate only (aggregate roots, internal helpers)
- `pub(super)` — visible to parent module only
- `pub(in path::to::module)` — visible to a specific module subtree
- (default) — private to the defining module

In a workspace, crate boundaries give you an additional enforceable layer. `pub(crate)` items are invisible to other crates entirely.

## Edition and MSRV in a workspace

- **`edition`** is per-crate. New crates should use `edition = "2024"`. Older crates on `edition = "2021"` can coexist; `edition` affects syntax / resolver per-crate.
- **`rust-version`** (MSRV) is per-crate. In a workspace, the MSRV of the whole distribution is the max of individual crate MSRVs.
- **CI must test MSRV.** Use a matrix: `rust: [stable, 1.85]` (or whatever your MSRV is). Otherwise drift is invisible.
- **MSRV split (cargo pattern):** cargo declares `rust-version = "1.92"` at workspace level but `rust-version = "1.95"` on the main `cargo` binary itself. The workspace floor is what downstream consumers see; the main package can require newer. Use this split when some workspace crates are consumed externally (lower MSRV for adoption) while the application binary can use newer features.

## Growing path

| From | To | Trigger |
|---|---|---|
| Single file | `src/main.rs` + modules | Multiple source files warranted |
| Single crate, flat `src/` | Single crate with `src/lib.rs` + `src/main.rs` | Need tests/benches to link the library; need a second binary |
| Single crate, modules | Workspace with one crate | Starting point if you KNOW you'll grow |
| Workspace, one crate | Workspace with split crates | Any "when to split" trigger above |

See [rust-planning/SKILL.md §5 Project Layout](SKILL.md#5-project-layout-decisions) for the full stage-by-stage progression.

## Cross-references

- [architecture-patterns.md](architecture-patterns.md) — full workspace organization patterns, hexagonal/onion layering, member Cargo.toml examples with DI traits
- [rust-implementing/SKILL.md §Modules & Cargo](../rust-implementing/SKILL.md) — implementation-side: `[[bin]]` syntax, `mod`/`pub use`/prelude patterns, `#[cfg(feature = "...")]` in code
- [rust-implementing/architecture-examples.md](../rust-implementing/architecture-examples.md) — complete worked examples with full directory trees
