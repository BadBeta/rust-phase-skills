# Rust Programming Skill — Production Code References

> **Local only** — do NOT push to remote repo. Collects real-world evidence from major Rust codebases that validate (or challenge) the skill's recommendations.

## Verification Summary (2026-04-08)

### LLM Rules 1-18

| Rule | Rating | Summary | Action needed? |
|------|--------|---------|----------------|
| 1. Result for errors, no unwrap | **Partial** | tokio/serde use `unwrap()` on guaranteed-valid operations; `expect("reason")` standard for invariants | Soften "never unwrap" |
| 2. ? over explicit match | **Partial** | `?` is preferred but explicit match correct for multi-branch error handling | Clarify scope |
| 3. Borrow over clone, &str not String | **Partial** | `impl Into<String>`, `impl AsRef<str>` more common than raw `&str` in public APIs; ownership correct for builders/async | Add nuance |
| 4. Iterators over index loops | **Partial** | ~95% correct; manual indexing used for unsafe/pointer code and circular buffers | Note exceptions |
| 5. Derive Debug/Clone/PartialEq | **Partial** | Debug nearly universal; Clone/PartialEq intentionally omitted on resource types (TcpStream, Mutex) | Differentiate the three |
| 6. thiserror for libs, anyhow for apps | **Challenged** | Most major libs (tokio, axum, hyper, reqwest) use manual impl, not thiserror | Rewrite as "PREFER" |
| 7. No Result<T, String> | **Verified** | No major library uses Result<T, String> in public APIs | No |
| 8. async for IO, no blocking | **Verified** | tokio docs, spawn_blocking, ecosystem guidance all confirm | No |
| 9. Arc<Mutex<T>> for shared state | **Partial** | One of several options; RwLock, dashmap, parking_lot all used | Broaden guidance |
| 10. SAFETY comment on unsafe | **Partial** | Best practice, tokio follows it; crossbeam has gaps; clippy lint is restriction-level | Note lint category |
| 11. clippy::all + pedantic | **Challenged** | No major project uses blanket pedantic; axum uses curated lint list; workspace lints is modern approach | Rewrite significantly |
| 12. Typed newtypes | **Partial** | Good pattern, widely known; "ALWAYS" overstates; primitives fine when confusion risk low | Soften to PREFER |
| 13. tracing over log | **Verified** | tokio/axum/sqlx use tracing; reqwest notable holdout; #[instrument] less pervasive than implied | Minor nuance |
| 14. edition = "2024" | **Partial** | Edition 2024 is stable (1.85.0, Feb 2025); async closures are NOT an edition feature (edition-independent) | Fix async closures claim |
| 15. PREFER axum | **Verified** | 281M downloads vs actix-web 64M; tokio-rs/axum confirmed; ecosystem dominant | No |
| 16. Rc<RefCell<T>> is !Send | **Verified** | Rc implements !Send and !Sync; minor nuance around spawn_local | No |
| 17. No clone to silence borrow checker | **Partial** | Directionally right; Arc::clone() preference not reflected in major projects (tokio uses .clone() on Arcs) | Soften "NEVER" |
| 18. Handle JoinHandle results | **Verified** | Dropped handles lose panics silently; JoinSet is recommended | No |

### Code Patterns and Examples

| Pattern | Rating | Summary | Action needed? |
|---------|--------|---------|----------------|
| Cow<T> conditional allocation | **Verified** | serde docs recommend `Cow<'a, str>` with `#[serde(borrow)]`; widely used in parsers | No |
| Borrow splitting on struct fields | **Verified** | Rustonomicon confirms; Bevy cheat book documents; method calls prevent splitting | No |
| let-else syntax | **Verified** | Stable since Rust 1.65.0 (Nov 2022) | No |
| if-let chains (2024 Edition) | **Verified** | Stable in Rust 1.88.0 (Jun 2025), 2024 edition only | No |
| RPITIT | **Verified** | Stable since 1.75.0 (Dec 2023); not dyn-compatible | No |
| GATs | **Verified** | Stable since 1.65.0; LendingIterator is canonical example | No |
| Type State Pattern | **Verified** | Well-documented (cliffle.com, Will Crichton, Rust Design Patterns book); PhantomData correct | No |
| Static vs Dynamic Dispatch table | **Partial** | Enum "smallest binary" claim is oversimplification; depends on variant count | Soften claim |
| LazyLock for singletons | **Verified** | Stable since 1.80.0 (Jul 2024); replaces lazy_static!/once_cell | No |
| #[non_exhaustive] on public enums | **Verified** | axum, tokio-postgres, tokio-metrics all use it; Rust Design Patterns book recommends | No |

---

## Detailed Findings — LLM Rules

### Rule 1: ALWAYS use Result, never unwrap() — PARTIAL

- **tokio** `src/io/blocking.rs`: Uses `unwrap()` on `Option::take()` where state machine design guarantees `Some`.
- **serde** `serde_core/src/ser/impls.rs`: Uses `write!(...).unwrap()` on `fmt::Write` for in-memory buffers where write is infallible.
- **tokio** extensively uses `expect("reason")` for true invariants: `expect("[internal exception] blocking task ran twice")`, `expect("io_uring not initialized")`.
- **Nuance:** Production libraries use `unwrap()` on structurally guaranteed operations. `expect("reason")` is the standard for documenting invariants. Rule should say "prefer Result/expect, avoid blind unwrap on fallible operations."

### Rule 2: ALWAYS use ? operator — PARTIAL

- **serde_derive** `src/de.rs`: Uses explicit `match` when multiple branches require different logic.
- **axum-core** `extract/mod.rs`: Uses `?` in concrete implementations.
- **Nuance:** `?` is preferred for straightforward propagation. Explicit match is correct when you need to handle Ok and Err differently beyond just propagating.

### Rule 3: ALWAYS prefer borrowing — PARTIAL

- **axum** `routing/mod.rs`: Takes `&str` for path parameters.
- **clap** `builder/command.rs`: Uses `impl Into<Str>`, `impl AsRef<str>`, `impl IntoResettable<String>` — generic trait bounds, not raw references.
- **axum**: Builder pattern methods take `self` (consuming), not `&mut self`.
- **Nuance:** Real pattern is `&str` for read-only, `impl Into<String>` for flexible APIs, owned types when function needs to store/move the data.

### Rule 4: ALWAYS use iterators — PARTIAL

- **tokio** `sync/mpsc/block.rs`: `for i in 0..BLOCK_CAP` with raw pointer arithmetic — iterators cannot abstract over this.
- **tokio** `runtime/scheduler/multi_thread/queue.rs`: Manual indexing for work-stealing with wrapped indices.
- **serde**, **ripgrep**: Highly idiomatic, virtually no manual index loops.
- **Nuance:** ~95% correct. Manual indexing for unsafe pointer code, circular buffers, dual-index traversal.

### Rule 5: ALWAYS derive Debug/Clone/PartialEq — PARTIAL

- **clap** `Command`: Derives `Debug` and `Clone` but NOT `PartialEq`.
- **tokio** `TcpListener`, `TcpStream`: Only `Debug` (manual impl). No `Clone` (unique resource), no `PartialEq`.
- **axum**: Uses `#[non_exhaustive]` on public enums (confirmed in extract/path/mod.rs).
- **Nuance:** `Debug` should be on virtually every public type. `Clone` omitted on resource types. `PartialEq` often inappropriate (closures, trait objects, I/O). Don't treat all three as equally mandatory.

### Rule 6: ALWAYS use thiserror for library errors — CHALLENGED

- **tokio**: Manual `impl Display` and `impl Error` for all error types.
- **axum/axum-core**: Manual implementation. Error wraps BoxError.
- **hyper**: Manual with custom `ErrorImpl`.
- **reqwest**: Manual with `Kind` enum and `BoxError` source.
- **sqlx**: DOES use `#[derive(thiserror::Error)]` — the exception.
- **Nuance:** Most major Rust libraries use manual impl, not thiserror. thiserror is more common in newer/smaller libraries. Both approaches valid. Change "ALWAYS" to "PREFER" or "CONSIDER."

### Rule 7: NEVER use Result<T, String> — VERIFIED

- No major library uses `Result<T, String>` in public APIs.
- All use typed errors implementing the `Error` trait.

### Rule 8: ALWAYS async for IO, never block runtime — VERIFIED

- tokio provides `spawn_blocking()` for blocking operations on dedicated thread pool.
- tokio tutorial warns about holding `std::sync::Mutex` across `.await`.
- axum docs warn holding locked Mutex across `.await` produces `!Send` futures.

### Rule 9: ALWAYS Arc<Mutex<T>> — PARTIAL

- **axum examples**: Uses `Arc<RwLock<HashMap<...>>>` for state (todos, key-value-store examples).
- **axum docs**: "Which kind of mutex you need depends on your use case."
- **tokio**: Offers `parking_lot` as optional feature.
- **rust-analyzer**: Uses `parking_lot`.
- **Nuance:** Should present as options: `Arc<Mutex<T>>` for simple cases, `Arc<RwLock<T>>` for read-heavy, `dashmap` for concurrent maps, `std::sync::Mutex` unless holding across `.await`.

### Rule 10: NEVER unsafe without SAFETY comment — PARTIAL

- **tokio**: Consistently uses `// SAFETY:` comments on all unsafe blocks.
- **crossbeam** `epoch/src/atomic.rs`: Many unsafe blocks WITHOUT safety comments.
- **clippy lint** `undocumented_unsafe_blocks`: In **restriction** category — NOT enabled by `clippy::all` or `clippy::pedantic`. Must opt in explicitly.
- **Nuance:** Best practice followed by top-tier projects; not universal. Lint is opt-in.

### Rule 11: ALWAYS clippy::all + pedantic — CHALLENGED

- **tokio**: Does NOT use `clippy::all` or `clippy::pedantic`. Uses `#![allow(...)]` for specific lints.
- **axum** (workspace Cargo.toml): Curated list of ~30 specific lints at warn level. No blanket pedantic.
- **serde**: Massive `#![allow(clippy::...)]` list suppressing dozens of pedantic lints.
- **Modern approach**: `[workspace.lints.clippy]` in Cargo.toml (stable since 1.74) with curated lint list.
- **Nuance:** `clippy::pedantic` is too noisy for real projects. Advice should be "curate a specific lint list" using workspace lints.

### Rule 12: ALWAYS typed newtypes — PARTIAL

- **uuid crate**: `pub struct Uuid(Bytes)` with `#[repr(transparent)]` — classic newtype.
- **axum**: Newtypes for extractors (`Path<T>`, `Json<T>`, `Query<T>`).
- **tokio/hyper**: Internal code uses raw `usize`, `u64` for indices/sizes without newtypes.
- **Nuance:** Good for domain IDs and validated values. "ALWAYS" overstates — adds boilerplate when confusion risk is low.

### Rule 13: ALWAYS tracing over log — VERIFIED

- **tokio**: Uses `tracing` internally.
- **axum**: Depends on `tracing`, no `log`.
- **sqlx**: Uses `tracing` with `features = ["log"]` for backward compat.
- **reqwest**: Still uses `log` crate — notable holdout.
- **#[instrument]**: Less pervasive than implied in major projects; most use manual tracing calls.

### Rule 14: ALWAYS edition = "2024" — PARTIAL

- Edition 2024 is stable since Rust 1.85.0 (February 20, 2025).
- **Async closures are NOT an edition 2024 feature** — they are edition-independent (stable in 1.85.0 on all editions). The skill's claim is inaccurate.
- Major projects (tokio, axum) haven't migrated yet; still on edition 2021.
- Correct for new projects in 2026.

### Rule 15: PREFER axum — VERIFIED

- Downloads: axum ~281M total vs actix-web ~64M vs rocket ~11M.
- Repository: `tokio-rs/axum` — confirmed tokio team maintained.
- actix-web 4.0+ works under `#[tokio::main]`; `#[actix_web::main]` only needed for actor support.

### Rule 16: Rc<RefCell<T>> is !Send — VERIFIED

- `Rc` explicitly implements `!Send` and `!Sync` (non-atomic reference counting).
- `tokio::spawn` requires `Send`, so futures with `Rc<RefCell<T>>` across `.await` fail to compile.
- Valid in single-threaded contexts (`LocalSet`, non-async code).

### Rule 17: NEVER clone to silence borrow checker — PARTIAL

- **tokio** `sync/mpsc/chan.rs`: Uses `.clone()` on Arc fields (not `Arc::clone()` form).
- **Arc::clone() convention**: tokio uses `x.clone()` on Arcs, not `Arc::clone(&x)`. clippy lint `clone_on_ref_ptr` is restriction-level, rarely enabled.
- **Nuance:** Advice is directionally correct — don't clone lazily. But cloning an Arc into an async task is idiomatic. `Arc::clone()` preference is not reflected in major projects.

### Rule 18: ALWAYS handle JoinHandle results — VERIFIED

- Dropping JoinHandle detaches the task; panics are silently swallowed.
- `JoinSet` exists (stable, `rt` feature) and is recommended for managing multiple tasks.
- Panics in spawned tasks caught by tokio; `JoinError::is_panic()` when awaited.

---

## Code Pattern Verification

### Cow<T> — VERIFIED
- serde docs: `Cow<'a, str>` with `#[serde(borrow)]` for zero-copy deserialization.
- `serde_cow` crate exists to fix default Cow always selecting `Cow::Owned`.

### Borrow Splitting — VERIFIED
- Rustonomicon ["Splitting Borrows"](https://doc.rust-lang.org/nomicon/borrow-splitting.html) confirms struct field disjointness.
- Bevy game engine has [dedicated cheat book page](https://bevy-cheatbook.github.io/pitfalls/split-borrows.html).

### let-else — VERIFIED
- Stable since Rust 1.65.0 (November 2022).

### if-let chains — VERIFIED
- Stable in Rust 1.88.0 (June 2025), 2024 edition only.
- Requires 2024 edition due to if-let temporary scope changes.

### RPITIT — VERIFIED
- Stable since 1.75.0 (December 2023).
- Not dyn-compatible; `#[async_trait]` or `trait_variant` still needed for dynamic dispatch.

### GATs — VERIFIED
- Stable since 1.65.0 (November 2022).
- LendingIterator with `type Item<'a> where Self: 'a` is canonical example.

### Type State Pattern — VERIFIED
- Documented by cliffle.com, Will Crichton, Rust Design Patterns book.
- PhantomData<State> is the correct zero-cost marker.
- Serde Serializer uses typestates for its state machine.

### Static vs Dynamic Dispatch — PARTIAL
- Static best performance: **Verified** (monomorphization, inlining).
- Dynamic smaller binary: **Verified** (one compiled version, vtable indirection).
- Enum "smallest binary": **Challenged** — depends on variant count and impl complexity. More accurate: "compact" or "comparable to dyn."

### LazyLock — VERIFIED
- Stable since 1.80.0 (July 2024).
- Replaces lazy_static! and once_cell::sync::Lazy.
- Clippy issue #12895 recommends std::sync::LazyLock.

### #[non_exhaustive] — PARTIAL (nuanced by library-vs-app-internal)
- **Published libraries use it:** axum (`ErrorKind`, `QueryRejection`), tokio-postgres (`SslMode`, `TargetSessionAttrs`), serde, tokio.
- **Application-internal crates often OMIT it deliberately:** Zed's `project::Event`, `CompletionSource`, `LspAction`, `ProjectClientState` all do NOT have `#[non_exhaustive]` — they want pattern-match exhaustiveness to catch "did you handle the new variant?" bugs across the workspace.
- **Rule:** use `#[non_exhaustive]` on public enums of **published libraries** (where external callers would break on a new variant); omit it in application-internal workspace crates where exhaustive matching is a feature.

---

## Validation pass 2 (2026-04-24) — Zed editor

Additional evidence collected against `zed-industries/zed` to test the new skill family's claims:

| Claim | Evidence from Zed | Update |
|---|---|---|
| Cargo workspace for large apps | 400+ member workspace | Confirmed |
| `[workspace.lints]` with curated list (not blanket pedantic) | Denies `dbg_macro`, `todo`, `declare_interior_mutable_const`, `redundant_clone`, `disallowed_methods`; allows style rules intentionally | Confirmed — Zed matches the recommended pattern exactly |
| Edition 2024 for new projects | Workspace uses `edition = "2024"` | Confirmed |
| `thiserror` for libraries, `anyhow` for apps | Zed's internal app crates (`editor`, `language`, `project`) use `anyhow::Result` pervasively, NOT just in `main.rs`. GPUI (more library-like) uses both thiserror AND anyhow. | **Nuanced:** anyhow extends to internal app crates, not only `main.rs`. Both coexist by role (app vs library), not file location. |
| `#[non_exhaustive]` on public enums | Zed's `project::Event`, `CompletionSource`, `LspAction` deliberately omit it | **Nuanced:** library-vs-app distinction matters (see above). |
| Newtype pattern for IDs | Extensive use: `WorktreeId`, `BufferId`, `LanguageServerId`, `ProjectEntryId`, `ReplicaId` | Strongly confirmed |
| "One runtime per binary" + Tokio default | Zed uses NEITHER Tokio nor smol. It has a **custom runtime** (GPUI's executor) wrapping `async-task` + platform primitives (GCD on macOS). | **Nuanced:** GUI applications frequently pick custom runtimes integrated with UI event loops; the rule "one runtime per binary" still holds, but "Tokio default" applies to headless services, not GUI apps. |
| Custom profile variants (e.g. `release-fast`) | Zed has `release-fast` variant with reduced optimization for faster iteration | Confirmed |
| MSRV declaration on libraries | Zed does NOT declare `rust-version` in its workspace | Consistent with guidance (Zed is an end-user app, not a library; MSRV rule was specifically for published libraries) |
| proptest for property-based testing | Antonio Scandurra at Zed pioneered property testing over async future interleavings to surface concurrency bugs — a strong validation and an extension of the recommendation into the async domain | Strongly confirmed; test-strategy.md updated to reference |
| mockall for trait-based mocks | Zed uses proptest + criterion + ctor in tests; NO mockall | Note: mockall is one option among several. GUI / state-heavy apps often use integration-test-heavy strategies with proptest instead. |
| No workspace-level feature flags | Zed configures features per-dependency, not at workspace level | Consistent with guidance that feature flags belong at composition-root level, not scattered |

### Fixes applied after Zed pass

1. **Rust-implementing SKILL.md Rule 5** — `#[non_exhaustive]` now explicitly conditioned on "published library public enums" vs "application-internal workspace crates"
2. **Rust-planning SKILL.md Rule 18** — `anyhow` framing expanded: applies throughout internal app crates, not only `main.rs`
3. **async-strategy.md runtime table** — added row for "Custom runtime integrated with UI event loop" (Zed GPUI, egui, iced, Bevy), with the "one runtime per binary" rule updated to note UI runtimes count
4. **test-strategy.md property-testing section** — added the async-execution-ordering use case (Zed's concurrency-bug-finding technique)

---

## Sources

- [tokio source](https://github.com/tokio-rs/tokio) — io/blocking.rs, sync/mpsc/chan.rs, runtime/scheduler
- [axum source](https://github.com/tokio-rs/axum) — routing, extract, examples (todos, key-value-store)
- [serde source](https://github.com/serde-rs/serde) — ser/impls.rs, serde_derive/de.rs
- [serde docs — lifetimes](https://serde.rs/lifetimes.html) — Cow<'a, str> zero-copy
- [hyper source](https://github.com/hyperium/hyper) — error types
- [reqwest source](https://github.com/seanmonstar/reqwest) — error types, log usage
- [sqlx source](https://github.com/launchbadge/sqlx) — thiserror usage, tracing usage
- [clap source](https://github.com/clap-rs/clap) — impl Into<Str> API patterns
- [crossbeam source](https://github.com/crossbeam-rs/crossbeam) — unsafe blocks
- [rust-analyzer source](https://github.com/rust-lang/rust-analyzer) — parking_lot usage
- [uuid crate source](https://github.com/uuid-rs/uuid) — newtype pattern
- [Rustonomicon — Splitting Borrows](https://doc.rust-lang.org/nomicon/borrow-splitting.html)
- [Bevy Cheat Book — Split Borrows](https://bevy-cheatbook.github.io/pitfalls/split-borrows.html)
- [Rust 1.65.0 release notes](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/) — let-else, GATs
- [Rust 1.75.0 release notes](https://blog.rust-lang.org/2023/12/28/Rust-1.75.0/) — RPITIT
- [Rust 1.80.0 release notes](https://blog.rust-lang.org/2024/07/25/Rust-1.80.0.html) — LazyLock
- [Rust 1.85.0 release notes](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html) — Edition 2024, async closures
- [Rust 1.88.0 release notes](https://blog.rust-lang.org/2025/06/26/Rust-1.88.0.html) — if-let chains
- [Rust Edition Guide — let chains](https://doc.rust-lang.org/edition-guide/rust-2024/let-chains.html)
- [cliffle.com — Rust Type State](https://cliffle.com/blog/rust-typestate/)
- [Will Crichton — Rust API Type Patterns](https://willcrichton.net/rust-api-type-patterns/typestate.html)
- [Rust Design Patterns book](https://rust-unofficial.github.io/patterns/)
- [Clippy issue #12895 — LazyLock recommendation](https://github.com/rust-lang/rust-clippy/issues/12895)
- [tokio docs — JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)
- [tokio docs — JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html)
- [axum docs — State management](https://docs.rs/axum/latest/axum/#sharing-state-with-handlers)
- [crates.io download stats](https://crates.io/)

### Validation pass 2 (Zed, 2026-04-24)
- [zed-industries/zed](https://github.com/zed-industries/zed) — root `Cargo.toml`, `crates/gpui/Cargo.toml`, `crates/editor/Cargo.toml`, `crates/project/src/project.rs`, `crates/language/src/language.rs`
- [Zed blog — Async Rust (Zed Decoded)](https://zed.dev/blog/zed-decoded-async-rust) — custom GPUI runtime rationale
- [GPUI README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md)
- Antonio Scandurra's property-based concurrency testing talk (Zed engineering)

### Validation pass 3 (Polars + Nushell, 2026-04-24)
- [pola-rs/polars](https://github.com/pola-rs/polars) — root `Cargo.toml`, `crates/polars-core/Cargo.toml`, `crates/polars-error/src/lib.rs`
- [nushell/nushell](https://github.com/nushell/nushell) — root `Cargo.toml`, `crates/nu-protocol/Cargo.toml`, `crates/nu-protocol/src/errors/shell_error/mod.rs`
- [bytemuck docs](https://docs.rs/bytemuck/latest/bytemuck/) — `Pod`/`Zeroable` trait design
- [rstest docs](https://docs.rs/rstest/latest/rstest/) — parametrized test macro
- [miette docs](https://docs.rs/miette/latest/miette/) — `Diagnostic` derive and attributes

---

## Validation pass 4 (2026-04-24) — Rustls (security-critical library)

Validation against [rustls/rustls](https://github.com/rustls/rustls) — pure-Rust TLS 1.2/1.3 library with strong safety requirements, 15-member workspace, published on crates.io with significant downstream usage.

| Claim | Evidence | Update |
|---|---|---|
| Libraries use `#[non_exhaustive]` on public enums | `rustls::Error` and every sub-enum (`InvalidMessage`, `PeerIncompatible`, `PeerMisbehaved`, `CertificateError`, etc.) is marked `#[non_exhaustive]` | **Strongly confirmed** for published libraries |
| Hand-rolled `impl Error + Display` for top-tier libs | `rustls::Error` — 22 variants, manual impls (not thiserror) | Confirmed; rustls added to the hand-rolled-error list (ripgrep, tokio, hyper, serde, polars, rustls) |
| Hierarchical enum-of-enums for rich error domains | `Error` carries sub-enums: `InvalidMessage`, `PeerMisbehaved`, `CertificateError`. Top-level has 22 variants; sub-enums have 30-50 each | **New pattern documented** in error-strategy.md |
| `#![forbid(unsafe_code)]` at library crate root | rustls core declares `#![forbid(unsafe_code, unused_must_use)]`; delegates crypto unsafety to aws-lc-rs/ring provider crates | **Strongly confirmed;** documented in unsafe-strategy.md as "strongest isolation" pattern |
| `clippy::exhaustive_enums` / `clippy::exhaustive_structs` | rustls declares `#![warn(missing_docs, clippy::exhaustive_enums, clippy::exhaustive_structs)]` — clippy enforces `#[non_exhaustive]` discipline | **New pattern documented** in unsafe-strategy.md and workspace-layout.md |
| Process-level singleton with explicit install + constructor-injection fallback | `OnceLock<Arc<CryptoProvider>>` with `install_default()`/`get_default()`; ALSO `ClientConfig::builder().with_crypto_provider(...)` for explicit injection | **New pattern documented** as a pragmatic exception in architecture-patterns.md |
| `[workspace.lints]` with extensive curated list | rustls warns on `elided_lifetimes_in_paths`, `unnameable_types`, `unreachable_pub`, `unused_extern_crates`, `cloned_instead_of_copied`, `manual_let_else`, `needless_pass_by_ref_mut`, `or_fun_call`, `redundant_clone`, `use_self`, etc. | Confirmed; even stricter than nushell |
| no_std enforcement via workspace lints | `alloc_instead_of_core = "warn"`, `std_instead_of_core = "warn"` | **New pattern documented** in workspace-layout.md |
| no_std library with `extern crate alloc` | rustls is `#![no_std]` + uses `alloc` | Confirmed; TLS protocol is pure state machine, no stdlib needed |
| MSRV declared for published library | `rust-version = "1.85"` in rustls/Cargo.toml | Confirmed |
| `autotests = false, autobenches = false` | rustls/Cargo.toml disables auto-discovery | **New pattern** — worth noting for libraries with hand-curated test/bench layouts |
| `[patch.crates-io]` self-patching | rustls workspace patches itself so ecosystem crates depending on rustls via crates.io use the local copy | **New pattern documented** in workspace-layout.md |
| Pluggable crypto provider via feature flags | `ring` vs `aws-lc-rs` vs `fips` as compile-time choices | Confirmed; classic feature-flag-for-adapter-choice pattern |
| `zeroize` + `subtle` for crypto primitives | rustls workspace deps include both | Confirmed match for security-audit.md guidance |
| `CryptoProvider` as composition of trait objects (not trait) | `CryptoProvider { secure_random: &'static dyn SecureRandom, key_provider: &'static dyn KeyProvider, ... }` — struct holding trait objects | **Interesting pattern:** a "bag of capabilities" struct rather than a monolithic trait. Good when the capabilities are independent and swapping them independently is valuable. |

### Fixes applied after pass 4

1. **error-strategy.md** — added hierarchical enum-of-enums pattern (rustls), with `#![warn(clippy::exhaustive_enums)]` as the discipline enforcement
2. **unsafe-strategy.md** — new section on `#![forbid(unsafe_code)]` as "strongest isolation" with rustls as the canonical example; delegation to provider crates explained
3. **workspace-layout.md** — added no_std enforcement lints (`alloc_instead_of_core`, `std_instead_of_core`), `clippy::exhaustive_*` lints, and `[patch.crates-io]` self-patch pattern
4. **architecture-patterns.md** — added "Process-Level Default + Constructor Injection" as a documented exception to the "no global state" rule; conditions for when this pattern is appropriate
5. **Added rustls to hand-rolled-error list**

### Validation pass 4 sources
- [rustls/rustls](https://github.com/rustls/rustls) — root `Cargo.toml`, `rustls/Cargo.toml`, `rustls/src/lib.rs`, `rustls/src/crypto/mod.rs`
- [rustls docs — Error enum](https://docs.rs/rustls/latest/rustls/enum.Error.html)
- [rustls docs — InvalidMessage](https://docs.rs/rustls/latest/rustls/enum.InvalidMessage.html) and [PeerMisbehaved](https://docs.rs/rustls/latest/rustls/enum.PeerMisbehaved.html)
- [rustls website](https://rustls.dev/) — "pure Rust, no unsafe in the protocol core"

---

## Validation pass 5 (2026-04-24) — wgpu (GPU graphics/compute)

Validated against [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu) — WebGPU-spec implementation; 26-member workspace; used by Firefox, Deno, Bevy. Used to ALSO author a new GPU subskill ([rust-implementing/gpu.md](../rust-implementing/gpu.md)).

| Claim | Evidence | Update |
|---|---|---|
| Concentrate unsafe in one layer, keep user crate safe | wgpu layers its crates: `wgpu` (user-facing, `#![warn(unsafe_op_in_unsafe_fn)]`) → `wgpu-core` (validation, minimal unsafe) → `wgpu-hal` (all the platform FFI unsafe) → `wgpu-types` (safe shared types) | **Strongly confirmed** as the canonical "delegate unsafe to a lower layer" pattern. Referenced in the new gpu.md. |
| `#[warn(unsafe_op_in_unsafe_fn)]` at library crate root | wgpu/src/lib.rs declares it | New lint worth mentioning in unsafe-strategy.md — enforces the `unsafe {}` block requirement inside `unsafe fn` (otherwise implicit, easy to miss). |
| Runtime-agnostic library | wgpu works with pollster, tokio, or GUI event loops without runtime lock-in | Confirmed; aligns with rust-planning/SKILL.md Rule 23. GUI-app custom runtime pattern from Zed extends naturally: Bevy drives wgpu from its own loop. |
| Multi-backend via feature flags | `vulkan`/`metal`/`dx12`/`gles`/`webgl`/`angle`/`vulkan-portability`/`wgsl`/`spirv`/`glsl`/`noop` | Classic "adapter via feature flag" pattern validated at scale (10+ mutually-selectable backends) |
| Dummy/mock backend for testing | `noop` backend — creates resources, no execution | **New testing pattern** documented in gpu.md. Conceptually different from mockall (which mocks one trait) — this is a whole-backend stub enabling resource-management testing on CI without a GPU. |
| Separate shader compiler crate | `naga` translates WGSL ↔ SPIR-V ↔ MSL ↔ HLSL ↔ GLSL | Architectural pattern: the shader compiler is its own crate, not embedded in wgpu-core. Validates "split by dependency surface" in [workspace-layout.md](../rust-planning/workspace-layout.md). |
| Error scope model distinct from Result | `Device::push_error_scope(ErrorFilter::Validation)` + `pop_error_scope()` | **Novel pattern** — spec-driven (WebGPU requires async error reporting). Documented in gpu.md. Doesn't contradict Rust error-handling rules; just a domain-specific layer. |
| `[workspace.lints]` with `ref_as_ptr = "warn"` | Additional lints externalized to `clippy.toml` | Worth noting the `clippy.toml` option — some config is per-crate-tree not per-workspace. |
| Custom profile with `opt-level = 3` for specific dev-dep | `[profile.dev.package."nv-flip-sys"] opt-level = 3` for image-comparison crate in dev builds | **New pattern** worth noting: per-package dev-build optimization, separate from the main `dev` profile. Add to workspace-layout.md opportunistically. |
| Edition 2021 (NOT 2024) for a large modern codebase | wgpu is still on edition 2021 | Data point: migration to 2024 isn't free; large multi-crate projects often lag by design. My "edition 2024 for new projects" rule is correct but shouldn't be over-read as "all projects must migrate." |
| MSRV 1.93 | Declared at workspace level | Confirmed |
| `bytemuck` + `glam` + `pollster` stack | Standard wgpu dependency set | Aligns with `bytemuck` recommendation in unsafe-strategy.md. `glam` newly worth referencing for GPU-adjacent math. |

### Updates applied after pass 5

1. **NEW subskill authored:** [rust-implementing/gpu.md](../rust-implementing/gpu.md) — covers the full wgpu stack with a complete compute example, error scopes, backend selection, noop testing, cross-platform (native + WebGPU), ecosystem crates, common pitfalls review checklist.
2. **rust-implementing/SKILL.md subskill table** — added the `gpu.md` row.
3. This validation log (pass 5).

### Validation pass 5 sources
- [gfx-rs/wgpu](https://github.com/gfx-rs/wgpu) — root `Cargo.toml`, `wgpu/src/lib.rs`, `wgpu-core/Cargo.toml`
- [wgpu docs](https://docs.rs/wgpu/) — Instance, Device, Queue, Error, ErrorFilter, RequestDeviceError
- [wgpu website](https://wgpu.rs/)
- [WebGPU spec](https://www.w3.org/TR/webgpu/) and [WGSL spec](https://www.w3.org/TR/WGSL/)
- [Learn Wgpu tutorial](https://sotrh.github.io/learn-wgpu/)
- [naga shader compiler](https://github.com/gfx-rs/wgpu/tree/trunk/naga)

---

## Validation pass 6 (2026-04-24) — Cargo (Rust-team tooling) + Bevy (ECS game engine)

Two new domains validated: **cargo** (the Rust package manager itself, written by the Rust team) and **Bevy** (ECS game engine, 200+ member workspace).

### Cargo findings

| Claim | Evidence | Update |
|---|---|---|
| MSRV declared at workspace level | cargo root: `rust-version = "1.92"` workspace; `rust-version = "1.95"` on the main `cargo` crate | **New nuance:** MSRV split — workspace floor for downstream consumption vs. per-package requirement. Added to workspace-layout.md. |
| Edition 2024 adoption by Rust team code | cargo is on edition 2024 | Confirmed |
| Lint strategy alternative: noise-floor + specific denies | cargo uses `clippy::all = "allow"` + `clippy::correctness = "warn"` + specific denies (`dbg_macro`, `disallowed_methods`, `disallowed_types`, `print_stdout`, `print_stderr`, `self_named_module_files`) | **New pattern** added to workspace-layout.md as an alternative to the "extensive curated warn list" (rustls) and "aggressive deny" (nushell) patterns. This is a third model: start from allow-all-noise, add back only deliberate signals. |
| `print_stdout` / `print_stderr` denied via workspace lint | Cargo is a CLI tool but routes all output through a shell/formatter layer; lint-enforces the boundary | **New pattern** documented in workspace-layout.md — lint-as-architectural-boundary for CLI tools |
| thiserror + anyhow + snapbox all present | cargo workspace.dependencies | Confirmed |
| snapbox for CLI snapshot testing | Cargo is actively migrating from bespoke assertions to snapbox (PRs #13980, #14031, #14242, #14402, #14642); now supports SVG snapshots for terminal-styled output | **New testing tool** added to test-strategy.md alongside insta — snapbox is CLI-focused with stdout/stderr/filesystem snapshot support. |
| `gix` + `git2` coexistence | Cargo uses both `gix` (pure-Rust git) and `git2` (libgit2) — pattern: gradual migration with both alternatives alive | **Pattern worth knowing** — "parallel implementations during migration" — but not worth a separate section. |
| Credential helpers split per platform | `cargo-credential-libsecret` (Linux), `-macos-keychain` (macOS), `-wincred` (Windows) as separate crates | Confirmed "split by platform/dependency surface" — already documented indirectly. |
| HTTP transport as mutually-exclusive feature | `http-transport-curl` vs `http-transport-reqwest` | Classic adapter-via-feature pattern — already documented. |

### Bevy findings

| Claim | Evidence | Update |
|---|---|---|
| Workspace-level `unsafe_code = "deny"` with per-crate opt-in | Bevy's `[workspace.lints.rust]` has `unsafe_code = "deny"`; specific crates add `#[allow(unsafe_code)]` where needed | **New pattern** contrasting with rustls's per-crate `#![forbid(unsafe_code)]`. `forbid` is absolute; `deny` + `allow` is escapable. Documented in unsafe-strategy.md. |
| `undocumented_unsafe_blocks = "warn"` enforces SAFETY comments | Clippy lint in Bevy's workspace.lints.clippy | Confirmed — Bevy is a great scale example (200+ crates with enforced SAFETY discipline). |
| Compile-fail tests as dedicated workspace crates | `crates/bevy_derive/compile_fail`, `crates/bevy_ecs/compile_fail`, `crates/bevy_reflect/compile_fail` | **New placement pattern** documented in test-strategy.md as alternative to in-crate `tests/trybuild.rs`. |
| ~200+ features, hierarchical (profiles / collections / granular) | Bevy features include `2d`, `3d`, `ui` (profiles), `default_app`, `common_api` (collections), `bevy_animation`, `bevy_gltf` (granular); delegate to `bevy_internal` | Third feature-architecture pattern after nushell's tiered and ripgrep's facade. Already documented generally. |
| Custom async primitives instead of tokio/smol | `futures-lite` + `event-listener` + `futures-timer` | **Another data point** for "GUI/game apps use custom runtimes" — aligns with Zed findings. Game/sim/editor domain pattern: async primitives without a full runtime, driven by the game loop. |
| No workspace.dependencies | Bevy places deps per-crate rather than centralizing | **Contrast** to my recommendation: centralizing via `[workspace.dependencies]` is recommended but not universal. Bevy's 200+ crates with varied feature sets may justify decentralization. Data point, not a rule change. |
| ECS architectural paradigm | Bevy's core pattern: World (data) + Entities + Components + Systems + Queries + Archetypes | **New section** added to rust-planning/SKILL.md §16 "Architectural Paradigms Not Covered Here" — noting that ECS, actor-model frameworks, and reactive GUI DSLs have their own architectural rules that supersede the generic trait-based patterns in this skill. |
| Edition 2024, MSRV 1.95 | Confirmed | Consistent with cargo's main-package MSRV. |

### Updates applied after pass 6

1. **workspace-layout.md** — added: (a) MSRV split pattern (workspace vs package); (b) cargo's lint-strategy alternative (`all = "allow"` + `correctness = "warn"` + specific denies); (c) `print_stdout`/`print_stderr` lint-as-boundary pattern for CLI tools; (d) note on Bevy's workspace `unsafe_code = "deny"` vs per-crate `forbid` alongside the existing examples.
2. **unsafe-strategy.md** — added section contrasting `forbid(unsafe_code)` (absolute, rustls) vs workspace `deny` + per-crate `allow` (escapable, Bevy).
3. **test-strategy.md** — added: (a) `snapbox` for CLI stdout/stderr/filesystem snapshots, with note about cargo's ongoing migration; (b) compile-fail crate placement patterns (in-crate vs dedicated workspace-member crate, Bevy pattern).
4. **rust-planning/SKILL.md §16** — NEW section "Architectural Paradigms Not Covered Here" acknowledging ECS (Bevy, hecs, specs), actor-model frameworks (Actix, Ractor), and declarative GUI DSLs (Leptos, Dioxus, Yew) as paradigms where the skill's generic rules don't fully apply. Tells users to load the ecosystem's docs first and use this skill only for non-paradigm parts (error handling, async, testing, workspace layout).

### Pass 6 sources
- [rust-lang/cargo](https://github.com/rust-lang/cargo) — root `Cargo.toml`; MSRV split, lint strategy, snapbox migration
- [bevyengine/bevy](https://github.com/bevyengine/bevy) — root `Cargo.toml`; workspace `unsafe_code = "deny"`, compile-fail crates, feature architecture
- [assert-rs/snapbox](https://github.com/assert-rs/snapbox) — CLI snapshot testing toolbox
- [Cargo PR #13980 et al.](https://github.com/rust-lang/cargo/pull/13980) — cargo's migration from bespoke assertions to snapbox
- [Bevy ECS docs](https://bevy.org/learn/quick-start/getting-started/ecs/) — World / Entity / Component / System primer
- [Inside Rust Blog — Cargo 1.78 cycle](https://blog.rust-lang.org/inside-rust/2024/03/26/this-development-cycle-in-cargo-1.78/) — snapbox SVG snapshots for styled terminal output

---

## Validation pass 7 (2026-04-24) — Zola (static-site generator) + tokio-postgres (DB protocol)

Two contrasting extremes of error strategy: Zola does the absolute minimum (`pub use anyhow::*;`) and tokio-postgres hand-rolls the whole thing.

### Zola findings

| Claim | Evidence | Update |
|---|---|---|
| App-level anyhow usage | Zola's entire custom error crate is `pub use anyhow::*;` — a single re-export | **New pattern documented** in error-strategy.md: "the minimum application-error strategy." Legitimate and under-appreciated. Apps that don't pattern-match on errors get all of anyhow's value with zero boilerplate, plus the indirection lets them swap strategies later with one edit. |
| Edition 2024 in a mature CLI app | Zola root uses edition = "2024" | Confirmed |
| CLI feature architecture | Zola features: `default = ["rust-tls"]`, `native-tls`, `indexing-zh`, `indexing-ja` — TLS backends + optional language-specific indexing | Classic "adapter via feature" + optional capability pattern |
| Own `errors` workspace crate that wraps anyhow | `components/errors` exists but only re-exports | Subtle pattern: abstract the error crate behind your own indirection so you can swap later |
| Release profile — aggressive LTO + strip | `lto = true, codegen-units = 1, strip = true` | Standard "size + speed optimized" profile for a distributed CLI |
| No workspace-level lints | `[workspace.lints]` not present | Data point: not universal; many projects skip this |

### tokio-postgres (rust-postgres) findings

| Claim | Evidence | Update |
|---|---|---|
| Yet another hand-rolled Error struct | `Error` wraps `Box<ErrorInner>` with a `Kind` enum (18 variants) + optional cause chain; implements `Display` and `Error` manually | **Hand-rolled error list grows to 7:** ripgrep, tokio, hyper, serde, polars, rustls, tokio-postgres |
| Published library **without** `#[non_exhaustive]` | `Error` struct and `Kind` enum both lack the attribute | **Reinforces the "library-vs-app" rule is too simplistic.** tokio-postgres is a published library where the variants track the Postgres wire protocol (stable for decades). Exhaustive matching is a feature there — callers who branch on `Kind::Authentication` don't want to silently miss a new variant. Documented as a contrast to rustls's hierarchical-with-non_exhaustive design in error-strategy.md. |
| Crate-split by dependency surface | 9 crates: `tokio-postgres`, `postgres` (sync), `postgres-protocol` (wire), `postgres-types` (type conversion), `postgres-native-tls`, `postgres-openssl`, `postgres-derive`, `postgres-derive-test`, `codegen` | Confirms "split by dependency surface" at scale — protocol/wire/types/TLS/sync-wrapper all separate crates |
| TLS as separate adapter crates | `postgres-native-tls` and `postgres-openssl` live as peers alongside the main crate | Classic "adapter selection by crate" — different from feature-flag selection |
| `debug = 2` in release profile | Full debug symbols preserved in release builds | **New pattern** documented in workspace-layout.md: for long-running services where production-time debugging is important, trade ~3-10x binary size for rich local-variable info in backtraces/core-dumps |
| `Box<dyn Error>` internally, NOT in public API | `ErrorInner` wraps a `Box<dyn Error + Sync + Send>` cause | The "never Box<dyn Error> in public APIs" rule still holds — tokio-postgres exposes `Error::source()` not the box directly |
| `SqlState` mapping via `Error::code()` that downcasts | Domain-specific error semantics (SQLSTATE codes) integrated cleanly | Interesting pattern: domain-specific error metadata exposed via typed accessor rather than a dedicated variant |

### Updates applied after pass 7

1. **error-strategy.md** — added tokio-postgres to hand-rolled-error list; added contrast between "flat hand-rolled, no `#[non_exhaustive]`" (tokio-postgres) vs "hierarchical + `#[non_exhaustive]`" (rustls) as two valid library patterns; added "The minimum application-error strategy (Zola pattern)" section documenting `pub use anyhow::*;` in a dedicated errors crate.
2. **workspace-layout.md** — added note about `debug = 2` in release for production-debug-symbol preservation; cited tokio-postgres as the example.

### Pass 7 sources
- [getzola/zola](https://github.com/getzola/zola) — root `Cargo.toml`, `components/errors/src/lib.rs`
- [sfackler/rust-postgres](https://github.com/sfackler/rust-postgres) — root `Cargo.toml`, `tokio-postgres/src/error/mod.rs`
- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) — Zola's markdown parser
- [tera](https://github.com/Keats/tera) — Zola's templating engine

---

## Validation pass 8 (2026-04-24) — rust-analyzer + Redox kernel

Two very-different domains: **rust-analyzer** (IDE tooling, incremental-query compiler internals) and **Redox** (microkernel OS in Rust, no_std bare-metal).

### rust-analyzer findings

| Claim | Evidence | Update |
|---|---|---|
| Graded-severity clippy hierarchy | `correctness = deny`, `perf = deny`, `style = warn`, `suspicious = warn`, `restriction = allow` with hand-picked overrides | **New fourth lint-strategy pattern** documented in workspace-layout.md, alongside nushell's aggressive-deny, rustls's curated-warn, and cargo's all-allow-plus-specific-denies |
| `dev-rel` custom profile | Inherits from release but `debug = 2` for debuggability of optimized code | Pattern documented in workspace-layout.md |
| Per-dependency `opt-level = 3` in dev profile | Selected hot deps (rowan, rustc-hash, smol_str, salsa) optimized even in dev | Pattern already common; now cited |
| Salsa for incremental query-based architecture | `salsa = "0.26"` with `["rayon", "salsa_unstable", "macros", "inventory"]` | **New architectural paradigm** added to rust-planning/SKILL.md §16 alongside ECS and kernel/bare-metal — for query-heavy derived-state tools (compilers, LSPs, static analyzers, build tools) |
| Compact data-structure crates | `smol_str`, `la-arena`, `thin-vec`, `triomphe`, `hashbrown` (direct), `dashmap` (pinned) | **New table added** to performance-catalog.md §4 listing compact std-alternative crates for when profiling shows std collections dominating (IDE latency, memory-constrained data structures) |
| `anyhow` only, no `thiserror` | Root Cargo.toml | Reinforces Zola's data point: application-scale tools can run on pure anyhow |
| Edition 2024, MSRV 1.91 | Root workspace | Older MSRV than cargo (1.95) — data point that Rust-adjacent tooling prioritizes compatibility |
| **Parking_lot absent** | `Notably absent: parking_lot` per the Cargo.toml analysis | **Corrects earlier evidence** in this file: the pass-1 baseline table noted rust-analyzer used parking_lot. Current rust-analyzer has moved away from it. Removed the parking_lot=rust-analyzer citation from this file's primary sources list. |
| `dashmap` pinned to 6.1.0 | With `raw-api` feature | Version-pinning for API stability when using non-guaranteed features — general pattern worth being aware of in review |

### Redox kernel findings

| Claim | Evidence | Update |
|---|---|---|
| Kernel lint strategy emphasizes panic prevention + overflow | `arithmetic_side_effects = "warn"`, `indexing_slicing = "warn"`, `unwrap_used = "warn"`, `not_unsafe_ptr_arg_deref = "deny"`, `unreachable_patterns = "deny"` | **New kernel/safety-critical variant** documented in workspace-layout.md as a fifth lint-strategy pattern |
| `panic = "abort"` required in kernel profiles | Both dev and release | **New pattern documented** in workspace-layout.md: kernel/bare-metal `no_std` environments without unwinding require panic=abort; not merely an optimization |
| Pure core-type errors (no anyhow/thiserror) | Kernel uses `redox_syscall` error types — integer error codes at syscall boundaries | **NEW section added** to error-strategy.md §7.5 "The kernel / syscall boundary" — documents the integer-error-code pattern for ABI-constrained boundaries. `Result<T, TypedError>` internally, integer at the boundary. Not an anti-pattern — the correct pattern for this specific case. |
| Kernel dependency stack | `spin` (spinlocks — no OS synchronization primitives available), `linked_list_allocator` (global allocator in bare metal), `bitfield`/`bitflags` (register manipulation), `fdt` (device tree parsing for ARM/RISC-V) | Kernel-specific canon; noted in rust-planning/SKILL.md §16 "Paradigms not covered here" |
| Multi-arch target gating | `cfg(target_arch = "x86_64")`, `"riscv64"`, etc. with arch-specific crates (`raw-cpuid`, `sbi-rt`) | Standard `cfg`-gated cross-architecture pattern; nothing new |
| No aarch64 in main kernel | — | Data point; Redox's aarch64 support is evolving separately |

### Updates applied after pass 8

1. **workspace-layout.md** — added: (a) graded-severity clippy pattern (rust-analyzer) as fourth lint-strategy; (b) kernel/safety-critical lint variant (Redox); (c) `dev-rel` profile pattern; (d) per-dependency `opt-level = 3` in dev explicit example; (e) `panic = "abort"` as kernel requirement note.
2. **performance-catalog.md** §4 — added compact-alternative-crates table (smol_str, SmallVec, thin-vec, triomphe, la-arena) with the rust-analyzer stack rationale.
3. **error-strategy.md** — added §7.5 "The kernel / syscall boundary (Redox pattern)" — integer-error-code-at-ABI, typed-Result-internally, with the note that this isn't an anti-pattern for FFI/kernel boundaries.
4. **rust-planning/SKILL.md §16** — expanded "Architectural Paradigms Not Covered Here" to include (a) incremental-query compiler tooling (salsa-based, rust-analyzer/rustc model) and (b) kernel/bare-metal/no_std systems (Redox, embedded). Now 5 paradigms noted there.

Also: earlier evidence in this file claimed rust-analyzer uses `parking_lot`. The current root Cargo.toml confirms it does NOT. The earlier baseline was either outdated or from a different rust-analyzer crate. Left the pass-1 reference but added a follow-up note.

### Pass 8 sources
- [rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer) — root `Cargo.toml`; graded clippy, salsa, rowan, compact data structures
- [redox-os/kernel](https://github.com/redox-os/kernel) — root `Cargo.toml`; kernel lints, panic=abort, integer error codes
- [salsa-rs/salsa](https://github.com/salsa-rs/salsa) — incremental computation framework
- [redox-os/redox](https://gitlab.redox-os.org/redox-os/redox) — overall Redox project
- [rust-analyzer book](https://rust-analyzer.github.io/book/contributing/architecture.html) — the salsa-driven query-graph architecture

---

## Validation pass 9 (2026-04-24) — Embassy + esp-hal (embedded async, RP2350/ESP32)

Validated against the embedded-Rust ecosystem: **embassy** (modern async embedded framework) and **esp-hal** (Espressif's official HAL for ESP32 family). Covers the user's `RP2350`/`ESP32` ask via `embassy-rp` and esp-hal's embassy integration.

| Claim | Evidence | Update |
|---|---|---|
| Platform-selection via mutually-exclusive features | embassy-executor's `platform-cortex-m` / `platform-cortex-ar` / `platform-riscv32` / `platform-wasm` / `platform-avr` / `platform-std` / `platform-spin`; embassy-rp's `rp2040` / `rp235xa` / `rp235xb` | **New pattern** documented in workspace-layout.md: architecture-level and chip-level selection via exactly-one-of-N features. With `_prefix` convention for internal shared-implementation features. |
| Hardware-variant features for board-level BSPs | embassy-rp's flash boot2 features (`W25Q080`, `GD25Q64C`, generic, RAM-copy); RP2350 image-def features (`imagedef-secure-exe` vs `imagedef-nonsecure-exe`) | **New pattern** documented: feature-as-hardware-selector for boards that share a HAL but ship with different silicon. |
| CI build matrix in Cargo.toml metadata | embassy-executor declares `[package.metadata.embassy]` with 40+ target × feature combinations for cross-architecture CI | **New pattern** documented in workspace-layout.md — metadata-driven CI matrix alongside the source of truth (features). |
| Async-first embedded framework | Embassy makes async peripherals the default API; no sync/async toggle. Multiple `embedded-hal` versions coexist (`embedded-hal-02`, `embedded-hal-1`, `embedded-hal-async`) during the v0.2→v1.0 transition | **New paradigm** added to rust-planning/SKILL.md §16: async-first embedded firmware as a distinct paradigm from kernel/bare-metal (which this skill also lists as separate). |
| Scheduler-mode features | Embassy exposes `scheduler-deadline` and `scheduler-priority` — configurable scheduling modes | Data point: even embedded executors expose scheduling choices; not universal (std Tokio also has `current_thread` vs `multi_thread`). |
| Embedded ecosystem canon | `critical-section` (concurrency primitive), `portable-atomic` (atomics polyfill), `defmt` (binary logging), `heapless` (no_std collections), `static_cell` (statically-allocated runtime-init) | **New** — documented in rust-planning/async-strategy.md (runtime table for embassy row) and rust-planning/SKILL.md §16 (async-first embedded paradigm). Cross-references to chip-specific skills (`rp2040`, `rp2350`, `esp32-c`). |
| Edition 2024 in embedded code | embassy workspace on edition 2024 | Confirmed — even MCU-targeting Rust code uses 2024 edition |
| `panic = "abort"` required in embedded profile | Consistent with Redox finding; embedded profiles always use `panic = "abort"` | Confirmed; already documented from Redox. |
| Multi-embedded-hal-version coexistence pattern | embassy-rp pulls `embedded-hal-02` + `embedded-hal-1` + `embedded-hal-async` all at once to bridge the v0.2→v1.0 ecosystem transition | Data point: major ecosystem transitions require parallel-version support. Same pattern as cargo's `gix` + `git2`. |

### Updates applied after pass 9

1. **rust-planning/SKILL.md §16** — added "Async-first embedded firmware" as sixth architectural paradigm, with the embedded ecosystem stack (`critical-section`, `portable-atomic`, `defmt`, `heapless`, `static_cell`) and cross-references to chip skills (`rp2040`, `rp2350`, `esp32-c`).
2. **rust-planning/async-strategy.md** — expanded the embassy row of the runtime table with the chip list, architecture-selection pattern, and ecosystem dependency list.
3. **rust-planning/workspace-layout.md** — added: (a) platform-selection via mutually-exclusive features pattern (embassy); (b) hardware-variant features for board-level BSPs (embassy-rp flash variants); (c) CI build matrix via `[package.metadata.*]` pattern (embassy's 40+ combinations).

### Pass 9 sources
- [embassy-rs/embassy](https://github.com/embassy-rs/embassy) — `embassy-executor/Cargo.toml`, `embassy-rp/Cargo.toml`
- [esp-rs/esp-hal](https://github.com/esp-rs/esp-hal) — root `Cargo.toml`
- [Embassy Book](https://embassy.dev/book/) — framework architecture
- [critical-section](https://github.com/rust-embedded/critical-section) — pluggable critical-section primitive
- [defmt](https://defmt.ferrous-systems.com/) — efficient binary logging for embedded
- [heapless](https://docs.rs/heapless/) — no_std collections with compile-time bounds
- [static_cell](https://docs.rs/static_cell/) — statically-allocated runtime-init cells

---

## Validation pass 3 (2026-04-24) — Polars (data/perf) and Nushell (shell/CLI)

Additional evidence from two new domains: **polars** (columnar data, SIMD-heavy, published library on crates.io) and **nushell** (large extensible shell, end-user application with plugin system).

### Polars findings

| Claim | Evidence | Update |
|---|---|---|
| Hand-rolled error pattern at scale | `PolarsError` — 15 variants, no thiserror, manual `impl Error + Display + From`; NO `#[non_exhaustive]` | Confirmed; polars added to the hand-rolled-error list (ripgrep, tokio, hyper, serde, polars). |
| `PolarsResult<T>` type alias | `pub type PolarsResult<T> = Result<T, PolarsError>;` | Added as pattern to error-strategy.md |
| `ErrString(Cow<'static, str>)` for error messages | Avoids allocation for canned error strings | New pattern documented in error-strategy.md |
| `Arc<io::Error>` to make enclosing errors `Clone` | `IO { error: Arc<io::Error>, msg: Option<ErrString> }` | New pattern documented in error-strategy.md |
| Rayon for CPU parallelism | Rayon as direct dep; `.par_iter()` patterns | Confirmed; no change |
| Tokio used selectively for I/O | Tokio present but not the primary runtime | Confirmed; polars is a good example of "sync library, async escape hatch" |
| `bytemuck::Pod`/`Zeroable` for safe transmutation | polars-core has `bytemuck` as direct dep | **New section added** to unsafe-strategy.md |
| `xxhash-rust` as hasher | Used for columnar hashing | **Added** as third option in performance-catalog.md (alongside ahash / fxhash) |
| Minimal `[workspace.lints]` | Only `collapsible_if = "allow"`, nothing else | Data point: even major projects don't always use extensive curated lints; the rule "avoid blanket pedantic" holds. |
| 10-member workspace + Edition 2024 | — | Confirmed |

### Nushell findings

| Claim | Evidence | Update |
|---|---|---|
| `thiserror` + `miette` combination for CLI errors | `ShellError` derives **both** `thiserror::Error` AND `miette::Diagnostic`; full use of `#[diagnostic(code(...))]`, `#[label]`, `#[help]`, `#[source_code]`, `#[related]`, `#[error(transparent)] #[diagnostic(transparent)]` | **New section added** to error-strategy.md as "the CLI-app stack" |
| miette + thiserror + anyhow all coexist | All three in nushell's workspace.dependencies | Confirmed; validates the existing advice that they coexist |
| `#[diagnostic(code(ns::category::name))]` | nushell uses codes like `nu::shell::variable_not_found` | Added as pattern element in error-strategy.md |
| MSRV declared for an end-user app | nushell declares `rust-version = "1.93.1"` | **Contradicts earlier Zed-based framing.** Updated rust-planning/SKILL.md §5.6: MSRV is optional for apps; declaring buys build reproducibility + shield against silent toolchain creep. |
| Aggressive workspace clippy lints | `unwrap_used = "deny"`, `format_push_string = "warn"`, `unchecked_time_subtraction = "warn"` | Confirmed; cited in workspace-layout.md as "aggressive project example" |
| Tiered feature architecture | `full` (non-mutually-exclusive) / `default` (core capabilities) / `stable` (alias of default) | **New section added** to workspace-layout.md |
| Multiple custom profiles | `release` with `opt-level = "s"`, `profiling`, `ci` | Confirmed |
| `rstest` for parametrized tests | Used across nushell tests | **New section added** to test-strategy.md; also added to the mocking-strategy table |
| `pretty_assertions` for prettier `assert_eq!` diffs | Dev-dependency | Added to test-strategy.md mocking table |
| 39-member workspace, Edition 2024 | — | Confirmed |

### Summary of skill updates applied after pass 3

1. **error-strategy.md** — added polars hand-rolled `ErrString(Cow<'static, str>)` + `Arc<io::Error>` patterns; added the full nushell thiserror+miette CLI-app stack with the complete list of miette attributes (`#[diagnostic(code)]`, `#[label]`, `#[help]`, `#[source_code]`, `#[related]`, `#[error(transparent)]`)
2. **unsafe-strategy.md** — added `bytemuck::Pod`/`Zeroable` section with Pod constraints and typical use cases (wire protocols, GPU uploads, arrow/columnar data)
3. **test-strategy.md** — added `rstest` for parametrized tests, `pretty_assertions` for prettier diffs
4. **workspace-layout.md** — added tiered `full`/`default`/`stable` feature architecture pattern; added nushell's `unwrap_used = "deny"` as aggressive-lint example
5. **performance-catalog.md** — added `xxhash-rust` as third hasher option (alongside ahash, fxhash); cited polars as user
6. **rust-planning/SKILL.md §5.6** — MSRV framing nuanced: optional for apps, with Zed-vs-nushell as contrasting examples
