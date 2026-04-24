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

# Common profiles
[profile.release]
lto = "thin"                  # Fast LTO by default
debug = 1                     # Line-table debug info for backtraces
codegen-units = 16

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
