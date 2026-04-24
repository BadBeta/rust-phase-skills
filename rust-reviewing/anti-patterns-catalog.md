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
