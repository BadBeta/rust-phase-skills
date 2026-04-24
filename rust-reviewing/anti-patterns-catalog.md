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
