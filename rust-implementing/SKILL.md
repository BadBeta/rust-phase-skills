---
name: rust-implementing
globs: "*.rs"
description: >
  Rust for idiomatic implementation — decision tables, templates, and anti-patterns
  that make Claude write idiomatic Rust at the moment of writing. Covers ownership,
  pattern matching, `?` error propagation, iterators, trait design, generics vs
  `dyn Trait`, Pin/Unpin, Tokio, channels, `Arc<Mutex>` patterns, serde, macros,
  FFI, axum, sqlx/diesel, clap, WASM/GUI, GPU/wgpu, and TDD with mockall, insta,
  proptest, cargo-fuzz. Rust 2024.
  ALWAYS use when writing Rust code.
  ALWAYS use when deciding between match/if-let/let-else/`?`.
  ALWAYS use when deciding between `impl Trait`, `dyn Trait`, and generics.
  ALWAYS use when deciding between `Arc<Mutex<T>>` and channels.
  ALWAYS use when writing async Rust (Tokio, channels, spawn, select).
  ALWAYS use when writing tests, mocks, or practicing TDD.
  ALWAYS use when refactoring Rust toward idiomatic form.
  For architecture/design, load rust-planning. For reviewing/debugging/profiling,
  load rust-reviewing.
---

# Rust — Implementing Skill

This skill is optimized for the moment of writing Rust code. It is one of three Rust skills:

- **rust-implementing** (this) — what to type. Rules, decision tables, idiomatic templates, anti-patterns, daily-coding operations, TDD.
- **[rust-planning](../rust-planning/SKILL.md)** — what to build before typing. Architectural decisions: project layout, crate boundaries, trait placement, error strategy, async strategy, unsafe budget, test strategy, resilience.
- **[rust-reviewing](../rust-reviewing/SKILL.md)** — how to critique existing code. Review PRs, debug bugs (panics, deadlocks, OOM, UB), profile performance (flamegraph, perf, DHAT, tokio-console).

The three skills follow the skill-authoring three-modes framework: rules constrain (fire during review), decision tables guide (fire at moment of writing), BAD/GOOD pairs verify (fire during validation). Rust is implementation-heavy — decision tables carry the most weight in this skill.

## Subskills — deep implementation references

This SKILL.md carries the always-loaded decision tables, top anti-patterns, and core rules. For detail depth on a specific area, load the matching subskill:

| Subskill | Purpose | Load when writing... |
|---|---|---|
| [language-patterns.md](language-patterns.md) | Pattern matching extended (match ergonomics, let-else, if-let chains, or-patterns), ownership patterns (borrow splitting, `Cow<T>`, zero-copy, entry API), `?` operator chains with context, iterator composition (custom iterators, `IntoIterator`, lazy evaluation), closure capture semantics (Fn/FnMut/FnOnce), trait patterns (extension traits, blanket impls, orphan rule), `From`/`Into`/`AsRef` conversion hierarchy, RAII & `Drop`, module organization & visibility, conditional compilation, production patterns (config, graceful shutdown, retry, middleware), internal iteration (push-based callbacks, Producer/Consumer/Folder pattern from rayon) | Daily Rust code — idiomatic control flow, ownership, iteration |
| [error-handling.md](error-handling.md) | `thiserror` derive, `anyhow` context chaining, `?` propagation, `.map_err` conversions, `From` impls across layers, hand-rolled `impl Display + impl Error` (ripgrep/tokio), error-value recovery (`SendError<T>`), uninhabited error types (`NoError`), production error reporting, recovery strategies | Writing error types and `Result` flow |
| [type-system.md](type-system.md) | Trait patterns (extension traits, blanket impls, orphan rule, object safety, supertraits), type conversions (From/Into/TryFrom/AsRef/Deref hierarchy), type state pattern (builder, protocol state machine), GATs (lending iterator, generic collection, type-parameterized), const generics (matrix, fixed buffer, defaults), Pin/Unpin (futures internals, pin projection, pin-project-lite), async traits (native vs async-trait, RPITIT, Rust 2024 capture rules), sealed traits, lifetime patterns (variance, PhantomData, HRTBs), marker type parameters for coherence (axum pattern), diagnostic attributes (`do_not_recommend`, `on_unimplemented`) | Advanced trait/generic code, Pin/Unpin, GATs, const generics |
| [async-patterns.md](async-patterns.md) | Tokio runtime internals (work stealing, cooperative scheduling), Pin/Unpin explained, channels (mpsc, broadcast, watch, oneshot patterns), rayon for CPU-bound parallelism, Tower service pattern, actor patterns, structured concurrency, async closures (Rust 2024 edition), graceful shutdown | Async code, channels, tasks, Tower services |
| [serde-patterns.md](serde-patterns.md) | Derive patterns with all common attributes, custom `Serialize`/`Deserialize` (Visitor pattern), enum representations (tagged, untagged, adjacently tagged), format-specific (JSON, TOML, bincode, CSV), zero-copy deserialization with `Cow<'a, str>`, `DeserializeOwned` vs `Deserialize<'de>` (lifetime distinction, three-tier string hierarchy) | Any serde code — derives, custom serialization, zero-copy |
| [macros.md](macros.md) | Declarative macros (`macro_rules!`, repetition, TT muncher), procedural macros (`TokenStream`, `syn`, `quote`), derive macros (parsing struct/enum), attribute macros, when to use macros vs generics vs traits, real production macro examples | Writing macros (declarative or procedural) |
| [ffi-patterns.md](ffi-patterns.md) | Unsafe blocks, safety contracts, raw pointers, FFI with C (`bindgen`, `cbindgen`, `repr(C)`), `CString`/`CStr`, byte manipulation, endianness, network protocol parsing, `AbortIfPanic` guard (rayon pattern for critical unsafe sections), `catch_unwind` at FFI boundary | Unsafe code, FFI, C/C++ interop |
| [web-apis.md](web-apis.md) | **Axum** (primary): extractors, middleware, routing, WebSocket, Tower integration. Rejection pattern (extractor error handling as responses). Actix Web, Rocket comparison. Authentication (JWT, Argon2, sessions), sqlx integration, reqwest HTTP client, CORS, static assets | Web/HTTP server or client code |
| [database.md](database.md) | SQLx (compile-time queries, `query_as!`, connection pools, migrations), Diesel (schema DSL, associations), MongoDB, caching (in-memory with moka/dashmap, Redis with deadpool), query composition patterns | Database code — queries, connection pools, caching |
| [cli-tools.md](cli-tools.md) | clap derive and builder patterns, subcommands, value validation, shell completions, indicatif progress bars, crossterm terminal manipulation, prettytable output, file system operations, signal handling, non-fatal error accumulation (atomic flag pattern for parallel processing), lexopt alternative for complex CLIs (ripgrep pattern) | CLI tools, progress bars, terminal UI |
| [gui-wasm.md](gui-wasm.md) | egui (immediate mode), iced (Elm architecture), Leptos/Yew (web frontend), WASM (wasm-bindgen, wasm-pack, WASI), server-side Wasm, JS interop patterns | Desktop GUIs, web frontends, WASM |
| [gpu.md](gpu.md) | GPU graphics and compute via **wgpu** — Instance/Adapter/Device/Queue lifecycle, buffers/textures/bind groups/pipelines, WGSL shader authoring, async submission + `map_async` + `poll`, error scopes, `noop` backend for tests, RenderDoc/validation-layer debugging, cross-platform native + WebGPU | Writing GPU compute or graphics code (wgpu, Bevy, egui-wgpu, iced-wgpu) |
| [testing-patterns.md](testing-patterns.md) | **FIRST-CLASS**: unit/integration/E2E tests, mockall (mock traits, expectations), insta (snapshot testing), proptest (property-based), cargo-fuzz (fuzzing with arbitrary), async test patterns, database test fixtures, test organization, loom model checking (concurrency testing), compile-fail tests (trybuild), TDD workflow, test-first trait design | Any test file; when starting a feature test-first |
| [documentation.md](documentation.md) | Rustdoc conventions, doc comments (`///`, `//!`), doc test attributes (`no_run`, `compile_fail`, `should_panic`, hidden lines), intra-doc links (`[\`Type\`]`, `[\`Type::method\`]`), standard sections (`# Examples`, `# Errors`, `# Panics`, `# Safety`), feature-gated docs, `#[doc(hidden)]`/`#[doc(inline)]`, `include_str!` for external docs, docs.rs configuration, lints | Writing doc comments, publishing a crate |
| [data-structures.md](data-structures.md) | Rust-specific data structure patterns (Vec, HashMap, BTreeMap, VecDeque, BinaryHeap), algorithm implementations, optimization patterns (SmallVec, arrayvec, indexmap) | Data-structure-heavy code, algorithm implementation |
| [architecture-examples.md](architecture-examples.md) | Complete worked examples: DI containers (shaku), domain modeling, resilience patterns, nanoservice architecture, async structured logging, multi-layer error translation | When implementing an architecture designed in rust-planning |
| [quick-reference.md](quick-reference.md) | Extended std/crate function reference (~300 methods): String, Vec, HashMap, Iterator, Option, Result, File/Path, formatting, common trait implementations (Display, FromStr, From/Into, AsRef, Deref, Index, IntoIterator, Drop), macros (std library, cfg, derive, attributes), chrono, regex, reqwest, tracing, clap, uuid, base64, anyhow/thiserror | Looking up std/crate function signatures |

**For architecture-level decisions** (which project layout / which error strategy / when to go async / when to use `dyn Trait` / which data store / which test layer), load `rust-planning`. **For critique of existing code**, load `rust-reviewing`.

## How to navigate this skill while coding

1. **Starting a feature?** — Read §1 (Rules), §2 (Master "which construct?" decision table), §3 (TDD workflow). Write a failing test before implementation.
2. **Deciding which construct to use?** — §2 (Master decision table) + §3.X (Type System / Error Handling / Async decision sub-tables).
3. **Writing an error type?** — §8 (Error Handling) + `error-handling.md`.
4. **Writing a match / pattern?** — §6 (Pattern Matching) + `language-patterns.md`.
5. **Writing async code?** — §15 (Async/Await Core) + §16 (Concurrency Primitives) + `async-patterns.md`.
6. **Writing a trait?** — §18 (Traits & API Design) + `type-system.md`.
7. **Writing a test?** — §25 (Testing Essentials + TDD) + `testing-patterns.md`.
8. **Stuck on a "won't compile"?** — §4 (Thinking in Rust) + §5 (Ownership & Borrowing).

---

## Rules for Writing Rust Code (LLM)

0. **ALWAYS practice TDD — write the failing test BEFORE the production code.** The cycle is Red → Green → Refactor: (1) write a test that expresses the behavior you want and watch it fail for the right reason, (2) write the minimum code to make it pass, (3) refactor with the test as a safety net. For bug fixes, the reproduction IS the regression test — commit the failing test in the same PR as the fix. For new features, the test drives the trait / type shape into existence: the call site in the test tells you what the API must look like. This is not "write tests after you're done." This is not "consider adding tests." This is the default workflow. Narrow exceptions: throwaway exploratory spikes (marked as such, followed by a TDD pass for the keeper version), generated code, and pure composition-root glue whose behavior is already covered by downstream unit tests. Trait-first TDD in Rust: write the call site first → that shape becomes the trait → implement a fake satisfying the trait → write the failing test → implement the real adapter. See §25 (Testing Essentials + TDD) for the full workflow.

1. **ALWAYS use `Result` for recoverable errors.** Reserve `unwrap()` for tests, prototypes, and cases where the value is structurally guaranteed (e.g., `Option::take()` in a state machine). Use `expect("reason")` to document true invariants. In production, prefer `?` or explicit error handling over blind `unwrap()` on fallible operations.
2. **ALWAYS propagate errors with `?` operator** for straightforward propagation. Add context with `.map_err()` or `anyhow::Context` when crossing module boundaries. Use explicit `match` when you need different logic per branch, not just to re-wrap and propagate.
3. **PREFER borrowing over cloning.** Take `&str` for read-only string params, `&[T]` for read-only slices. Use `impl Into<String>` or `impl AsRef<str>` for flexible public APIs (as clap and axum do). Take ownership when the function needs to store or move the data (builders, async tasks, struct fields).
4. **ALWAYS use iterators over manual index loops.** Prefer `.iter()`, `.map()`, `.filter()`, `.collect()` over `for i in 0..len`. Iterator chains are zero-cost abstractions and prevent off-by-one errors. Exception: manual indexing is appropriate for unsafe pointer arithmetic, circular buffer manipulation, or simultaneous multi-array traversal with complex index relationships.
5. **ALWAYS derive `Debug` on public types.** Derive `Clone` unless the type owns a unique resource (file handle, connection, runtime). Derive `PartialEq` when meaningful — omit on types containing closures, trait objects, or I/O resources. Derive `serde::Serialize`/`Deserialize` for types crossing serialization boundaries. Use `#[non_exhaustive]` on **published-library public enums and error types** (like ripgrep, tokio, serde do) so adding variants isn't a breaking change. **Omit `#[non_exhaustive]` in application-internal workspace crates** where you want exhaustive-match compile errors to catch "did you handle the new variant?" bugs — Zed's `project::Event`, `CompletionSource`, etc. deliberately stay exhaustive for this reason.
6. **PREFER `thiserror` for library error types and `anyhow` for application error handling.** Many major libraries (tokio, axum, hyper, ripgrep, serde) hand-roll `impl Display + impl Error` for full control over formatting, `#[non_exhaustive]`, and patterns like `Error { kind: ErrorKind }` wrappers — both approaches are valid and production-proven. Never use `Box<dyn Error>` in public APIs. Define specific error variants, not catch-all strings.
7. **NEVER use `String` for error messages in `Result`.** Use typed errors (`Result<T, MyError>`) so callers can match on variants. String errors lose information and prevent programmatic handling.
8. **ALWAYS mark long-running or I/O operations as `async`** when in an async context. Never block the async runtime with `std::thread::sleep()` or synchronous I/O — use `tokio::time::sleep()` and async equivalents. Use `tokio::task::spawn_blocking()` for unavoidable blocking operations.
9. **Use appropriate synchronization for shared mutable state.** `Arc<Mutex<T>>` for simple cases, `Arc<RwLock<T>>` when reads vastly outnumber writes, `dashmap` for concurrent maps, `parking_lot::Mutex` for better performance. Use `std::sync::Mutex` (not `tokio::sync::Mutex`) unless you need to hold the lock across `.await` points. Never use `Rc<RefCell<T>>` across thread or `.await` boundaries (it is `!Send`).
10. **ALWAYS add a `// SAFETY:` comment on `unsafe` blocks** explaining why the invariants are upheld. Minimize unsafe surface area — wrap unsafe in safe abstractions with clear contracts. Note: the clippy lint `undocumented_unsafe_blocks` is in the restriction category (opt-in), but top-tier projects (tokio, rust-analyzer) follow this practice consistently.
11. **ALWAYS use `clippy` with a curated lint configuration.** Use `[workspace.lints.clippy]` in Cargo.toml (stable since 1.74) to define project-specific lints — this is the modern approach used by axum and other major projects. Avoid blanket `clippy::pedantic` (too noisy — even serde suppresses dozens of its lints). Curate specific warn-level lints relevant to your project. Never blanket `#[allow(clippy::all)]`.
12. **PREFER typed newtypes for domain values** where primitive confusion is a real risk. `struct UserId(u64)` prevents mixing up user IDs with order IDs. Use the newtype pattern for identifiers, validated values, and units. Raw primitives are fine for internal indices, sizes, and counters where confusion risk is low.
13. **ALWAYS use `tracing` over `log` for new projects** — structured, span-aware, async-compatible. Used by tokio, axum, and sqlx. Use `#[instrument]` on key entry points where span context is valuable.
14. **ALWAYS specify `edition = "2024"` in new Cargo.toml** — use the latest stable edition (stabilized in Rust 1.85.0) for improved RPIT lifetime captures, `unsafe extern` blocks, and `gen` keyword reservation. Note: async closures (also stabilized in 1.85.0) are edition-independent and work on all editions.
15. **PREFER `axum` for new web APIs** — Tower-native, maintained by tokio team (`tokio-rs/axum`), dominant ecosystem adoption (~4x actix-web downloads). Actix Web 4.0+ works under `#[tokio::main]`; `#[actix_web::main]` is only needed for actor support.
16. **NEVER use `Rc<RefCell<T>>` in async code** — it is `!Send`. Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` in any code that crosses `.await` points or thread boundaries. `Rc<RefCell<T>>` is only for single-threaded, synchronous code (or `tokio::task::spawn_local`).
17. **NEVER use `.clone()` to silence the borrow checker** without understanding why the borrow fails. Restructure ownership, use references, or scope borrows more tightly. Clone is appropriate for cheap-to-clone types (`Arc`, small structs) or when you genuinely need separate owned copies. Note: `.clone()` on `Arc` is idiomatic — the `Arc::clone(&x)` form is a style preference, not a requirement.
18. **ALWAYS handle `JoinHandle` results from `tokio::spawn`** — unwatched tasks that panic silently lose errors. Use `JoinSet`, store handles, or at minimum log spawn failures. Never fire-and-forget spawned tasks.

## Master "Which Construct?" Decision Table

**This is the primary decision surface for the skill.** When writing Rust code, consult this table to pick the right construct for the job. The "NOT this" column catches the common anti-pattern.

Rust has many ways to express the same idea. Picking the idiomatic construct at the moment of writing avoids rewrites and matches how the ecosystem's major libraries (tokio, axum, serde, ripgrep) are written.

### Control flow

| When you need to... | Use this | NOT this |
|---|---|---|
| Branch on enum variants exhaustively | `match value { Variant::A { .. } => ..., Variant::B => ..., }` | Chain of `if let` |
| Extract one variant, fall through otherwise | `let Variant::Ok(x) = value else { return Err(...) }` (let-else) | `let x = match value { Variant::Ok(x) => x, _ => return Err(...) }` |
| Handle 1-2 patterns, with else | `if let Some(x) = opt { use(x) } else { default() }` | `match opt { Some(x) => use(x), None => default() }` |
| Multiple let-else / match guards in sequence | `if let Chain` (Rust 2024 edition): `if let Some(x) = a && let Some(y) = b && x > 0` | Nested `if let` / nested `match` |
| Test a value matches a pattern (bool) | `matches!(value, Variant::A { .. })` | `if let Variant::A { .. } = value { true } else { false }` |
| Branch on multiple booleans | `match (a, b) { (true, true) => ..., }` | Nested `if`/`else if` |
| Early return on error | `let x = fallible()?;` | `let x = match fallible() { Ok(x) => x, Err(e) => return Err(e) }` |
| Early return with error context | `let x = fallible().map_err(\|e\| MyError::Context(e))?;` or `fallible().context("doing X")?` (anyhow) | Raw `?` without context at layer boundaries |

### Error handling

| Situation | Construct | NOT this |
|---|---|---|
| Library error type | `#[derive(thiserror::Error)]` enum with typed variants, `#[non_exhaustive]` | `Box<dyn Error>`; `Result<T, String>` |
| Application top-level error | `anyhow::Error` with `.context()` | Hand-built enum in main.rs |
| User-facing diagnostic (compiler-style) | `miette::Diagnostic` derive | `eprintln!("{}", e)` |
| Convert between error types | `impl From<SqlxError> for RepoError` (called by `?`) | `.unwrap()` / manual match |
| Fallback value on error | `.unwrap_or(default)`, `.unwrap_or_else(\|e\| ...)` | `match result { Ok(v) => v, Err(_) => default }` |
| Expected business failure | `Result<T, DomainError>` with typed variant | Panic / `unreachable!()` |
| Invariant violation (impossible state) | `panic!()` or `unreachable!()` with message | `Result` with never-matched error |
| Propagate + add context | `.context("...")?` (anyhow) or `.map_err(\|e\| Error::Context(e))?` | Bare `?` at crate boundary |
| Early exit from `main` | `-> anyhow::Result<()> { ... Ok(()) }` with `?` throughout | Manual process::exit |

### Type choice

| Situation | Use | NOT this |
|---|---|---|
| Function param: read-only string | `&str` | `&String`, `String` |
| Function param: read-only slice | `&[T]` | `&Vec<T>`, `Vec<T>` |
| Function param: flexible string (accept `&str`, `String`, `&String`) | `impl AsRef<str>` | Overload with multiple fns |
| Function param: flexible into-owned | `impl Into<String>` | Force caller to clone |
| Function param: take ownership | `String`, `Vec<T>`, `T` | Clone inside to pretend to borrow |
| Struct field: owned | `String`, `Vec<T>` | `&'a str` unless the lifetime is essential |
| Struct field: shared | `Arc<T>` | `Rc<T>` (unless single-threaded) |
| Return: may allocate or borrow | `Cow<'a, str>` | Two functions / sum type |
| Return: iterator | `impl Iterator<Item = T>` | `Vec<T>` (callers can collect) |
| Return: async | `impl Future<Output = T>` or `async fn` | Boxed future |
| Integer for indexing / length | `usize` | `u32`, `i32` (conversion bugs) |
| Integer for hardware register | `u32`, `u64` (documented width) | `usize` (varies per arch) |
| Money | `rust_decimal::Decimal` or custom newtype | `f64` |

### Static vs dynamic dispatch

| Situation | Use | NOT this |
|---|---|---|
| One concrete impl per binary | Generics (`impl Trait` or `<T: Trait>`) — monomorphizes, inlines | `Box<dyn Trait>` |
| Heterogeneous collection | `Vec<Box<dyn Trait>>` | Generics (can't) |
| Plugin / adapter list | `Box<dyn Trait>` or enum dispatch | Generic with phantom types |
| Public API of a library with one impl | `impl Trait` return type | Trait bound generic param (less ergonomic for users) |
| Callback / closure parameter | `impl Fn(X) -> Y` | `Box<dyn Fn(X) -> Y>` unless storing |
| Trait object safety concerns | Check object safety first; use `&dyn` / `Box<dyn>` if safe | Silent fallback to generics forces API change |

### Concurrency primitive

| Situation | Use | NOT this |
|---|---|---|
| Shared immutable data | `Arc<T>` | `&'static T` (requires `'static` everywhere) |
| Shared mutable, infrequent writes | `Arc<RwLock<T>>` (std for sync, `tokio::sync::RwLock` for async held across `.await`) | `Arc<Mutex<T>>` if reads dominate |
| Shared mutable, short critical sections NOT held across `.await` | `Arc<std::sync::Mutex<T>>` or `parking_lot::Mutex` for perf | `tokio::sync::Mutex` (unnecessary overhead when no `.await` inside the lock) |
| Shared mutable across `.await` | `tokio::sync::Mutex` / `RwLock` | `std::sync::Mutex` (would block the runtime) |
| Concurrent map | `dashmap::DashMap` | `Arc<Mutex<HashMap>>` |
| Atomic counter / flag | `AtomicU64` / `AtomicBool` (`Ordering::Relaxed` usually OK) | `Arc<Mutex<u64>>` |
| Single-threaded reference counting | `Rc<T>` | `Arc<T>` (overhead unneeded) |
| Producer → single consumer | `tokio::sync::mpsc::channel(capacity)` | Shared queue with mutex |
| Producer → many consumers (pub/sub) | `tokio::sync::broadcast::channel(capacity)` | `Vec<Sender>` |
| Many producers → observed latest value | `tokio::sync::watch::channel(initial)` | Poll `Arc<Mutex<T>>` |
| Request-response one-shot reply | `tokio::sync::oneshot::channel()` | mpsc of size 1 |
| CPU-bound parallelism | `rayon::par_iter()` | `tokio::spawn` in a hot loop (async runtime ≠ parallelism) |
| Fire-and-forget async task | `JoinSet::new()` + `set.spawn(fut)` | `tokio::spawn(fut); /* drop handle */` |
| Single long-lived background task | Store `JoinHandle` in your state, await on shutdown | Fire-and-forget |
| Blocking I/O inside async | `tokio::task::spawn_blocking(\|\| sync_work())` | Direct blocking call (blocks runtime thread) |

### Async patterns

| Situation | Use | NOT this |
|---|---|---|
| Run N futures concurrently, fail fast | `tokio::try_join!(a, b, c)` | Sequential `.await` |
| Run N futures concurrently, collect | `futures::future::join_all(...)` or `JoinSet` | Sequential `.await` |
| Race futures, take first | `tokio::select! { r = a => ..., r = b => ... }` | Spawn both, drop one |
| Timeout a future | `tokio::time::timeout(Duration::from_secs(30), fut).await?` | Check clock manually |
| Sleep (non-blocking) | `tokio::time::sleep(duration).await` | `std::thread::sleep` (blocks runtime) |
| Interval timer | `tokio::time::interval(duration)` + `.tick().await` | Sleep loop |
| Cancel a task | Pass `CancellationToken` (tokio-util); check with `select!` | `handle.abort()` (can corrupt state) |
| Shutdown many tasks | Parent `CancellationToken` → child tokens | Manual signal-per-task |
| Graceful drain | `select!` between shutdown + work, drain channel | Kill abruptly |

### Data-structure choice

| Situation | Use | NOT this |
|---|---|---|
| Unknown-size sequential | `Vec<T>` | Array |
| Fixed-size compile-time | `[T; N]` | `Vec<T>` |
| Small-stack-optimized | `smallvec::SmallVec<[T; N]>` | Always-heap `Vec` |
| Keyed lookup | `HashMap<K, V>` | `Vec<(K, V)>` + linear scan |
| Keyed lookup, insertion order | `indexmap::IndexMap<K, V>` | `HashMap` + parallel `Vec<K>` |
| Keyed lookup, sorted iteration | `BTreeMap<K, V>` | Sort `HashMap` on demand |
| Unique items | `HashSet<T>` | `Vec<T>` + dedup |
| Double-ended queue | `VecDeque<T>` | `Vec` with `remove(0)` (O(n)) |
| Priority queue | `BinaryHeap<T>` | Sort on every push |
| FIFO channel | `tokio::sync::mpsc` | Mutex-wrapped `VecDeque` |
| Cache with TTL/LRU | `moka::future::Cache` | Custom `Arc<Mutex<HashMap>>` |
| Small, fixed capacity | `arrayvec::ArrayVec` | `Vec` for stack-only data |

### Trait bound style

| Situation | Use | Why |
|---|---|---|
| Input position: one trait | `fn f(x: impl Trait)` | Most ergonomic for callers |
| Input position: multiple bounds | `fn f<T: Trait1 + Trait2>(x: T)` | Name the type for where-clause docs |
| Lifetime + `Send` for async | `fn f<T: Trait + Send + 'static>(x: T)` | Async spawning needs `'static` |
| Return position | `fn f() -> impl Trait` | Don't expose concrete type |
| Object safety needed | `Box<dyn Trait>` (trait must be object-safe) | Store heterogeneous impls |
| Many bounds, readable | `where T: Trait1, T: Trait2` below signature | Keep signature line short |

### Testing (first-class — see §25 and `testing-patterns.md`)

| Situation | Use | NOT this |
|---|---|---|
| Unit test inside module | `#[cfg(test)] mod tests { ... }` in same file | External test file |
| Integration test | `tests/foo.rs` (separate file, treats crate as external) | Inline in lib.rs |
| Async test | `#[tokio::test]` | `#[test]` with manual runtime |
| Mock a trait | `#[mockall::automock]` on trait + `MockTrait::new()` in test | Real impl + test DB |
| Snapshot assertion | `insta::assert_snapshot!(actual)` | `assert_eq!` for large strings |
| Property test | `proptest! { fn x(s in ".*") { ... } }` | Hand-crafted test cases only |
| Fuzz test | `cargo-fuzz` with `libfuzzer_sys::fuzz_target!` | Random inputs in unit test |
| Time control in async test | `tokio::time::pause()` + `advance()` | `std::thread::sleep` |
| DB-backed test, rolled back | `#[sqlx::test]` | Manual transaction |
| Compile-fail negative test | `trybuild` | Comment `// won't compile` |

### Macro choice

| Situation | Use | NOT this |
|---|---|---|
| Repeat syntactic pattern | `macro_rules!` (declarative) | Proc macro |
| Derive trait impl from struct | `#[derive(MyDerive)]` proc macro | `macro_rules!` (can't parse items) |
| Transform function/item | Attribute proc macro | `macro_rules!` + inline |
| Generate bindings from external format | `build.rs` + code generator | Hand-write |
| Format string | `format!`, `write!`, `writeln!` | String concat with `+` |

---

## Thinking in Rust

**Ownership as Resource Management** — Every value has exactly one owner. Design functions around who should own data. If a function doesn't need to keep the data, take a reference. If it does, take ownership. The ownership model replaces garbage collection AND prevents data races:

```rust
// Who needs to own this data?
fn process(data: &[u8]) -> Summary { /* borrows — caller keeps data */ }
fn consume(data: Vec<u8>) -> Summary { /* takes ownership — caller gives up data */ }
fn share(data: Arc<Vec<u8>>) { /* shared ownership — multiple owners */ }
```

**Make Invalid States Unrepresentable** — Use the type system to prevent bugs at compile time. Newtypes prevent mixing up IDs. Enums prevent impossible states. Type state prevents calling methods in wrong order:

```rust
// BAD: any string is a "valid" email
fn send_email(to: &str) { ... }

// GOOD: only validated emails can exist
struct Email(String);  // Can only be created through Email::new() which validates
fn send_email(to: &Email) { ... }  // Compiler guarantees validation happened
```

**Zero-Cost Abstractions** — Iterators, traits, generics, and closures compile to the same machine code as hand-written loops and switches. Don't avoid abstractions for "performance" — use them for correctness and clarity.

**Parse, Don't Validate** — Convert raw data into typed structures at system boundaries. Work with typed data internally. Validation returns `bool` (info discarded); parsing returns `Result<T>` (info preserved):

```rust
// BAD: validate then use raw data
fn process(input: &str) -> Result<()> {
    if !is_valid_email(input) { return Err(Error::Invalid); }
    send_email(input);  // Still a raw &str — could pass unvalidated string
    Ok(())
}

// GOOD: parse into typed structure at boundary
fn process(input: &str) -> Result<()> {
    let email = Email::parse(input)?;  // Parse once at boundary
    send_email(&email);  // Type guarantees validity
    Ok(())
}
```

**Compiler as Collaborator** — When the borrow checker rejects your code, it's usually revealing a real problem (data race, use-after-free, aliased mutation). Restructure the code rather than fighting it. If you can't make it work, the design likely has a concurrency or ownership bug.

### Imperative/OOP to Rust Translation

**Collection Operations:**

| C++/Java/Python | Idiomatic Rust |
|---|---|
| `for(i=0; i<len; i++) arr[i]` | `items.iter().enumerate()` or `.iter().map()` |
| `result = []; for x in list: result.append(f(x))` | `let result: Vec<_> = list.iter().map(f).collect();` |
| `list.filter(x => pred(x))` | `list.iter().filter(\|x\| pred(x)).collect()` |
| `acc = 0; for x in list: acc += x` | `list.iter().sum()` or `.fold(0, \|acc, x\| acc + x)` |
| `list.find(x => x.id == target)` | `list.iter().find(\|x\| x.id == target)` |
| `list.flatMap(x => x.children)` | `list.iter().flat_map(\|x\| &x.children).collect()` |
| `[...new Set(list)]` (deduplicate) | `list.into_iter().collect::<HashSet<_>>()` |
| `dict(zip(keys, values))` | `keys.iter().zip(values).collect::<HashMap<_,_>>()` |

**Control Flow & Error Handling:**

| C++/Java/Python | Idiomatic Rust |
|---|---|
| `try { risky() } catch(e) { handle(e) }` | `match risky() { Ok(v) => use(v), Err(e) => handle(e) }` |
| `if (x == null) throw ...` | `let x = opt.ok_or(Error::Missing)?;` |
| `x?.y?.z` (optional chaining) | `x.as_ref().and_then(\|x\| x.y.as_ref()).and_then(\|y\| y.z.as_ref())` |
| `switch(type) { case A: ... }` | `match value { Variant::A => ..., Variant::B => ... }` |
| `throw new Exception("msg")` | `return Err(MyError::Variant)` or `bail!("msg")` |
| `try { ... } finally { cleanup() }` | RAII: `Drop` impl runs automatically, or `scopeguard` crate |

**OOP Patterns to Rust:**

| OOP | Rust Equivalent |
|---|---|
| `class Foo extends Bar` | `trait Bar {}; impl Bar for Foo {}` — composition over inheritance |
| `interface IService` | `trait Service { fn method(&self); }` |
| `abstract class Base` | Trait with default methods + required methods |
| `obj.field = value` (mutate) | `let new = Struct { field: value, ..old };` or `&mut self` methods |
| `new Foo()` (constructor) | `Foo::new()` or `Foo::builder().build()` — no special syntax |
| `Singleton.getInstance()` | `static INSTANCE: LazyLock<Foo> = LazyLock::new(\|\| ...);` |
| `List<Animal> animals` (polymorphism) | `Vec<Box<dyn Animal>>` or `enum Animal { Dog(...), Cat(...) }` |
| `private` fields | All fields are private by default — add `pub` explicitly |
| `instanceof` check | `matches!(value, Variant::A { .. })` or `if let` |

## Ownership & Borrowing

### The Three Rules

1. **Each value has exactly one owner**
2. **When owner goes out of scope, value is dropped**
3. **Ownership can be transferred (moved) or borrowed**

```rust
let s1 = String::from("hello");
let s2 = s1;                      // Ownership moved to s2
// println!("{}", s1);           // ERROR: s1 no longer valid

let s3 = s2.clone();              // Deep copy, s2 still valid
println!("{} {}", s2, s3);
```

### Move vs Copy

```rust
// Copy types (stack-only, implement Copy trait):
// i8..i128, u8..u128, f32, f64, bool, char, (i32, bool), [i32; 5]
let x = 5;
let y = x;  // Copy — both valid

// Move types (heap data):
let s1 = String::from("hello");
let s2 = s1;  // Move — s1 invalid
```

### Borrowing Rules

```rust
let mut s = String::from("hello");

// Multiple immutable references: OK
let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2);

// Mutable reference after immutable refs are done: OK (NLL)
let r3 = &mut s;
r3.push_str(" world");

// NEVER: mutable + immutable at same time
// let r1 = &s; let r2 = &mut s; println!("{}", r1); // ERROR
```

### Lifetimes

```rust
// Explicit: returned reference lives as long as shortest input
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

// Elision rules (compiler infers):
// 1. Each ref param gets its own lifetime
// 2. If exactly one input lifetime, output gets it
// 3. If &self/&mut self, output gets self's lifetime

// Struct holding a reference
struct Excerpt<'a> {
    text: &'a str,
}

// 'static — lives for entire program (string literals, owned data)
let s: &'static str = "I live forever";
// T: 'static means T owns all its data (no borrowed refs)
fn spawn_task<T: Send + 'static>(data: T) { /* ... */ }
```

### Cow<T> — Clone on Write

```rust
use std::borrow::Cow;

// Avoids allocation when data doesn't need modification
fn maybe_uppercase(input: &str, shout: bool) -> Cow<str> {
    if shout {
        Cow::Owned(input.to_uppercase())  // Allocates only when needed
    } else {
        Cow::Borrowed(input)              // Zero-cost borrow
    }
}

// Common in parsers, config loaders, and string processing
fn normalize_path(path: &str) -> Cow<str> {
    if path.contains("//") {
        Cow::Owned(path.replace("//", "/"))
    } else {
        Cow::Borrowed(path)
    }
}
```

### Borrow Splitting

```rust
// The borrow checker tracks borrows to individual struct fields
struct GameState {
    player: Player,
    enemies: Vec<Enemy>,
    score: u32,
}

fn update(state: &mut GameState) {
    // This works — different fields are borrowed independently
    let player = &mut state.player;
    let enemies = &state.enemies;  // Immutable borrow of different field
    player.update(enemies);
    state.score += 1;  // Mutable borrow of yet another field
}

// But it doesn't work through methods — the whole &mut self is borrowed
impl GameState {
    fn update(&mut self) {
        // FAILS: self.player() borrows all of self, so self.enemies() can't borrow
        // let p = self.player();
        // let e = self.enemies();

        // WORKS: access fields directly
        let p = &mut self.player;
        let e = &self.enemies;
        p.update(e);
    }
}
```

### Temporary Borrow Scoping

```rust
// Limit borrow scope by using a block
let mut data = vec![1, 2, 3, 4, 5];

let first = {
    let slice = &data[..];
    slice.first().copied()  // Borrow of data ends here
};
data.push(6);  // Now we can mutate

// Same pattern with Mutex — don't hold the guard
let result = {
    let guard = mutex.lock().unwrap();
    guard.clone()  // Clone what you need, drop the guard
};
// Mutex is unlocked here
do_async_work(result).await;
```

### Deref Coercion

```rust
use std::ops::Deref;

// Deref enables transparent forwarding
struct Email(String);

impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

let email = Email("alice@example.com".into());
println!("{}", email.len());           // str::len() via Deref
println!("{}", email.contains('@'));   // str::contains() via Deref

// Deref chain: Box<String> → String → str
let boxed: Box<String> = Box::new("hello".into());
let s: &str = &boxed;  // Deref coercion through Box → String → str

// CAUTION: Don't abuse Deref for "inheritance"
// Deref is for smart pointer types, not for subtyping
// BAD: struct Admin(User); impl Deref for Admin { type Target = User; }
// GOOD: struct Admin { user: User, permissions: Permissions }
```

> **Deep dive:** [type-system.md](type-system.md) — Pin/Unpin internals (self-referential types,
> why futures must be pinned), lifetime variance, subtyping, advanced Cow patterns, zero-copy parsing
> with borrowed data, PhantomData uses beyond type state.
> [language-patterns.md](language-patterns.md) — Cow\<T\> in structs, zero-copy patterns (serde borrow,
> string interning), entry API for complex map updates, From/Into/AsRef conversion hierarchy.

## Pattern Matching

```rust
enum Command {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    Color(u8, u8, u8),
}

fn handle(cmd: Command) {
    match cmd {
        Command::Quit => std::process::exit(0),
        Command::Move { x, y } => println!("Move to ({x}, {y})"),
        Command::Write(text) => println!("{text}"),
        Command::Color(r, g, b) => println!("rgb({r}, {g}, {b})"),
    }
}

// Guards, bindings, or-patterns
match value {
    n if n < 0 => println!("negative"),
    id @ 1..=5 => println!("small: {id}"),
    1 | 2 | 3 => println!("one two three"),
    _ => println!("other"),
}

// if let / while let
if let Some(v) = optional { use_value(v); }
while let Some(top) = stack.pop() { process(top); }

// Nested destructuring
let ((x, y), Point { z, .. }) = (coords, point);
```

### let-else — Bind or Diverge

```rust
// let-else keeps the happy path flat — bind or return/break/continue
fn process(input: &str) -> Result<Output, Error> {
    let Some(header) = input.lines().next() else {
        return Err(Error::EmptyInput);
    };

    let Ok(config) = parse_header(header) else {
        return Err(Error::InvalidHeader);
    };

    let Some(value) = config.get("key") else {
        return Err(Error::MissingKey("key"));
    };

    // header, config, value all available here — no nesting
    Ok(Output::new(value))
}
```

### if-let Chains (Rust 2024 Edition)

```rust
// Chain multiple patterns with &&
if let Some(user) = get_user(id)
    && let Some(email) = user.email.as_ref()
    && email.ends_with("@company.com")
{
    send_internal_notification(email);
}

// Replaces deeply nested if-let blocks
```

### matches! Macro

```rust
// Returns bool — useful in filter/any/all contexts
let is_digit = matches!(ch, '0'..='9');
let is_keyword = matches!(word, "if" | "else" | "for" | "while" | "loop" | "match");

// With guards
let is_small_positive = matches!(n, x if x > 0 && x < 100);

// In iterators
let has_errors = results.iter().any(|r| matches!(r, Err(_)));
let errors: Vec<_> = results.iter()
    .filter(|r| matches!(r, Err(_)))
    .collect();

// Matching nested patterns
let is_ok_and_even = matches!(result, Ok(n) if n % 2 == 0);
```

> **Deep dive:** [language-patterns.md](language-patterns.md) — or-patterns with bindings,
> match ergonomics (auto-ref), match on references without moving, exhaustive matching strategies,
> destructuring complex types (nested structs, slice patterns, tuple structs).
> [type-system.md](type-system.md) — type state pattern with exhaustive enum matching.

## Type System

### Generics

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    list.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
}

struct Pair<T, U> { first: T, second: U }

// Specialized implementation
impl Pair<f64, f64> {
    fn distance(&self) -> f64 {
        (self.first.powi(2) + self.second.powi(2)).sqrt()
    }
}
```

### Traits

```rust
trait Summary {
    fn summarize(&self) -> String;
    fn preview(&self) -> String { format!("Read more: {}", self.summarize()) }
}

// Trait bounds — three equivalent syntaxes
fn notify(item: &impl Summary) { /* sugar for generic */ }
fn notify<T: Summary>(item: &T) { /* explicit generic */ }
fn notify<T>(item: &T) where T: Summary + Display { /* where clause */ }

// impl Trait in return position (opaque type)
fn make_iter() -> impl Iterator<Item = i32> {
    (0..10).filter(|n| n % 2 == 0)
}
```

### Associated Types vs Generics

```rust
// Associated type: ONE implementation per type (Iterator::Item)
trait Container {
    type Item;
    fn get(&self, idx: usize) -> Option<&Self::Item>;
}

// Generic: MANY implementations per type (From<T>)
trait From<T> {
    fn from(value: T) -> Self;
}
```

### Trait Objects (Dynamic Dispatch)

```rust
// dyn Trait = vtable-based dynamic dispatch
fn render(components: &[Box<dyn Draw>]) {
    for c in components { c.draw(); }
}

// Object safety: no Self return, no generics, methods take &self/&mut self
// Static dispatch (monomorphized): faster, code bloat
// Dynamic dispatch (vtable): smaller code, slight overhead
```

### Extension Traits & Sealed Traits

```rust
// Extension trait: add methods to types you don't own
trait StrExt {
    fn shout(&self) -> String;
}
impl StrExt for str {
    fn shout(&self) -> String { self.to_uppercase() + "!" }
}

// Sealed trait: prevent external implementations
mod private { pub trait Sealed {} }
pub trait MyTrait: private::Sealed {
    fn method(&self);
}
// Only types in this crate can impl Sealed, so only they can impl MyTrait

// Orphan rule: you can impl a trait only if you own the trait OR the type
// Foreign trait on foreign type? Use a newtype wrapper:
// impl Display for Vec<u8> { }       // FAILS: orphan rule
struct Bytes(Vec<u8>);
// impl Display for Bytes { }         // WORKS: you own Bytes

// TryFrom — fallible conversions (impl From for infallible)
impl TryFrom<u32> for Port {
    type Error = PortError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 65535 { Err(PortError::OutOfRange) } else { Ok(Port(value as u16)) }
    }
}
// let port = Port::try_from(8080)?;
```

### RPITIT — Return Position Impl Trait in Traits (stable 1.75+)

```rust
// Before: required async-trait crate or Box<dyn Future>
// After: native async fn in traits and impl Trait returns

trait Repository {
    // Native async fn in trait — no #[async_trait] needed
    async fn find(&self, id: u64) -> Option<Record>;

    // impl Trait in trait return position
    fn iter(&self) -> impl Iterator<Item = &Record>;
}
```

### GATs — Generic Associated Types (stable 1.65+)

```rust
// GATs let associated types have their own generic parameters
trait LendingIterator {
    type Item<'a> where Self: 'a;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}

// Enables borrowing from self in associated types
struct WindowsMut<'a, T> {
    data: &'a mut [T],
    pos: usize,
}

impl<'a, T> LendingIterator for WindowsMut<'a, T> {
    type Item<'b> = &'b mut [T] where Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        if self.pos + 2 <= self.data.len() {
            let window = &mut self.data[self.pos..self.pos + 2];
            self.pos += 1;
            Some(window)
        } else {
            None
        }
    }
}
```

### Const Generics (stable 1.51+)

```rust
// Parameterize types and functions by compile-time constants
struct ArrayVec<T, const N: usize> {
    data: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> ArrayVec<T, N> {
    fn new() -> Self {
        Self { data: std::array::from_fn(|_| None), len: 0 }
    }

    fn push(&mut self, value: T) -> Result<(), T> {
        if self.len >= N {
            return Err(value);
        }
        self.data[self.len] = Some(value);
        self.len += 1;
        Ok(())
    }
}

// Zero-cost matrix with compile-time dimensions
fn dot_product<const N: usize>(a: &[f64; N], b: &[f64; N]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
```

### Type State Pattern (Compile-Time State Machines)

```rust
use std::marker::PhantomData;

// States are zero-sized types — no runtime cost
struct Draft;
struct Review;
struct Published;

struct Document<State> {
    title: String,
    content: String,
    _state: PhantomData<State>,
}

impl Document<Draft> {
    fn new(title: String) -> Self {
        Self { title, content: String::new(), _state: PhantomData }
    }
    fn edit(&mut self, content: String) { self.content = content; }
    fn submit(self) -> Document<Review> {
        Document { title: self.title, content: self.content, _state: PhantomData }
    }
}

impl Document<Review> {
    fn approve(self) -> Document<Published> {
        Document { title: self.title, content: self.content, _state: PhantomData }
    }
    fn reject(self) -> Document<Draft> {
        Document { title: self.title, content: self.content, _state: PhantomData }
    }
}

impl Document<Published> {
    fn view(&self) -> &str { &self.content }
}

// Compiler prevents: doc.view() on Draft, doc.edit() on Published
let doc = Document::<Draft>::new("RFC".into());
// doc.view(); // ERROR: no method `view` on Document<Draft>
```

### Higher-Rank Trait Bounds (HRTBs)

```rust
// for<'a> means "for ALL possible lifetimes"
fn apply<F>(f: F) where F: for<'a> Fn(&'a str) -> &'a str {
    let owned = String::from("hello");
    println!("{}", f(&owned));      // Works with temporary
    println!("{}", f("static"));    // Works with 'static
}

// Common in closure-accepting APIs and trait objects
fn get_processor() -> Box<dyn for<'a> Fn(&'a str) -> usize> {
    Box::new(|s| s.len())
}
```

### Static vs Dynamic Dispatch Decision

| Criteria | `impl Trait` / Generics (Static) | `dyn Trait` (Dynamic) | Enum Dispatch |
|----------|----------------------------------|----------------------|---------------|
| Set of types known at compile time? | Yes or no | Usually no | **Yes** (closed set) |
| Need heterogeneous collection? | No | **Yes** (`Vec<Box<dyn T>>`) | **Yes** (`Vec<MyEnum>`) |
| Performance critical? | **Best** (monomorphized, inlined) | Vtable overhead | **Good** (no vtable, match) |
| Binary size concern? | Larger (code per type) | **Smaller** | **Compact** (no monomorphization or vtable) |
| Adding new types? | Easy | Easy | Requires code changes |
| Object safety required? | No | **Yes** (no Self return, no generics) | No |

**Rule of thumb:** Start with generics/`impl Trait`. Use `dyn Trait` when you need heterogeneous collections or plugin-style extensibility. Use enum dispatch when you have a closed, known set of variants and want maximum performance.

### String Type Decision

| Type | Use When |
|------|----------|
| `&str` | Function parameters, read-only string access, string literals |
| `String` | Owned, growable strings — struct fields, return values, building strings |
| `Cow<'a, str>` | May or may not need to allocate — parsers, config loaders, normalization |
| `&[u8]` / `Vec<u8>` | Binary data, non-UTF-8 content, byte-level manipulation |
| `OsString` / `&OsStr` | File paths, environment variables (may not be valid UTF-8) |
| `CString` / `&CStr` | FFI with C (null-terminated, no interior nulls) |
| `Box<str>` | Immutable owned string with exact allocation (no capacity overhead) |

> **Deep dive:** [type-system.md](type-system.md) — **trait patterns** (extension traits, blanket impls,
> orphan rule, object safety rules, supertraits, trait composition, sealed traits deep dive),
> **type conversions** (From/Into/TryFrom/AsRef/Deref hierarchy, conversion decision guide),
> **type state** (builder with required fields, protocol state machine, when to use),
> **GATs** (lending iterator, generic collection trait, type-parameterized associated types),
> **const generics** (matrix, fixed buffer, defaults, nightly expressions),
> **Pin/Unpin** (why futures are self-referential, pin projection, pin-project-lite),
> **async traits** (native vs async-trait, RPITIT, Rust 2024 capture rules, trait_variant),
> **lifetime patterns** (elision rules, variance, PhantomData uses, 'static misconceptions, HRTBs).

## Error Handling

### Rules for Error Handling (LLM)

1. **ALWAYS use `thiserror` for library crates, `anyhow` for application crates.** Libraries expose typed errors callers can match; applications just need context and propagation.
2. **NEVER use `String` as an error type** — use typed error enums with variants. String errors lose all programmatic handling capability.
3. **ALWAYS add `.context()` or `.with_context()` when propagating errors across module boundaries** — bare `?` loses the "where" information.
4. **ALWAYS use `#[from]` for automatic error conversion** between layers. Define `From` impls or use `#[from]` in thiserror enums.
5. **PREFER matching specific error variants** over catch-all handlers — `Err(DbError::NotFound { .. })` not `Err(e) => log(e)`.
6. **NEVER use `unwrap()` in library code.** Use `expect("invariant reason")` only for true invariants. In applications, prefer `?` with context.
7. **ALWAYS use `ensure!()` / `bail!()` from anyhow** for early validation — cleaner than manual if-return-Err.
8. **ALWAYS document error conditions** in `///` doc comments for public functions — which error variants can be returned and why.

### Error Crate Decision

| Crate | Use When | Key Feature |
|-------|----------|-------------|
| `thiserror` | Library crates, typed API errors | Derive `Error` with `#[error]`, `#[from]`, `#[source]` |
| `anyhow` | Application crates, CLI tools | `context()`, `bail!()`, `ensure!()`, any error type |
| `color-eyre` | Applications needing rich error reports | Colorized backtraces, span traces, custom sections |
| `miette` | User-facing tools, diagnostics | Source code snippets, labels, help text in errors |

**Rule:** Use `thiserror` + `anyhow` together — thiserror in your library crates, anyhow in your binary crate. They interoperate seamlessly.

```rust
// Library: use thiserror for typed errors callers can match
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("record not found: {entity} id={id}")]
    NotFound { entity: &'static str, id: String },

    #[error("query failed: {query}")]
    QueryFailed { query: String, #[source] cause: sqlx::Error },

    #[error(transparent)]
    Io(#[from] std::io::Error),  // auto From impl
}

// Application: use anyhow for convenience + context
use anyhow::{Context, Result, bail, ensure};

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading config from {path}"))?;
    let config: Config = toml::from_str(&content)
        .context("parsing config TOML")?;
    ensure!(config.port > 0, "port must be positive, got {}", config.port);
    Ok(config)
}

// Error conversion across layers
fn fetch_user(id: u64) -> Result<User, AppError> {
    let config = load_config()?;       // ConfigError -> AppError
    let user = db.find_user(id)?;      // DbError -> AppError
    Ok(user)
}

// ? with Option
fn nested_lookup(data: &HashMap<String, HashMap<String, i32>>) -> Option<i32> {
    let inner = data.get("outer")?;
    let value = inner.get("inner")?;
    Some(*value)
}

// .context() vs .with_context() — use with_context for expensive formatting
fn load_user(id: u64) -> Result<User> {
    db.find(id)
        .with_context(|| format!("loading user {id} from database"))?
        .ok_or_else(|| anyhow::anyhow!("user {id} not found"))
}
// Error output with context chain:
// Error: loading user 42 from database
//   Caused by:
//     connection refused
```

### Error Conversion Across Layers

```rust
// Layer 1: Repository errors (thiserror)
#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("not found: {entity} {id}")]
    NotFound { entity: &'static str, id: String },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

// Layer 2: Service errors — wraps repo errors
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Repo(#[from] RepoError),         // Auto From impl
    #[error("validation: {0}")]
    Validation(String),
    #[error("unauthorized")]
    Unauthorized,
}

// Layer 3: Handler errors — wraps service errors
#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error(transparent)]
    Service(#[from] ServiceError),   // Auto From impl
    #[error("bad request: {0}")]
    BadRequest(String),
}

// Now ? converts automatically up the chain:
// sqlx::Error → RepoError → ServiceError → HandlerError
async fn handle_request(id: u64) -> Result<Response, HandlerError> {
    let user = user_service.find(id)?;   // ServiceError → HandlerError
    Ok(Response::ok(user))
}
```

> **Deep dive:** [error-handling.md](error-handling.md) — color-eyre setup and customization,
> miette diagnostic reports with source snippets, multi-layer error translation (handler → service → repo),
> error conversion chains across crate boundaries, recovery strategies (retry, fallback, circuit breaker),
> custom error context types, anyhow downcast patterns, collecting multiple errors (partition, validation).

## Iterators & Closures

Iterators are Rust's primary abstraction for collection processing. They are **zero-cost** — iterator chains compile to the same machine code as hand-written loops. Prefer iterators over manual indexing in all cases.

### The Three Iterator Methods

```rust
let v = vec![1, 2, 3];
v.iter()        // yields &T — borrows the collection (most common)
v.iter_mut()    // yields &mut T — mutably borrows
v.into_iter()   // yields T — consumes the collection (moves ownership)

// for loops use IntoIterator automatically:
for x in &v { }       // equivalent to v.iter()
for x in &mut v { }   // equivalent to v.iter_mut()
for x in v { }        // equivalent to v.into_iter() — v consumed!
```

### Iterator Adapters (Lazy)

Nothing happens until a consuming adapter is called — adapters just build a pipeline:

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Transform
let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

// Filter
let evens: Vec<&i32> = numbers.iter().filter(|x| *x % 2 == 0).collect();

// Filter + Transform (skip None)
let parsed: Vec<i32> = ["1", "two", "3"].iter()
    .filter_map(|s| s.parse().ok())
    .collect();  // [1, 3]

// Flatten nested iterators
let words: Vec<&str> = ["hello world", "foo"].iter()
    .flat_map(|s| s.split_whitespace())
    .collect();  // ["hello", "world", "foo"]

// Chain two iterators
let all: Vec<i32> = (1..4).chain(10..13).collect();  // [1,2,3,10,11,12]

// Take / Skip / Window
let first_3: Vec<_> = numbers.iter().take(3).collect();
let skip_2: Vec<_> = numbers.iter().skip(2).collect();
let windows: Vec<_> = numbers.windows(2).collect();  // [[1,2],[2,3],[3,4],[4,5]]
let chunks: Vec<_> = numbers.chunks(2).collect();     // [[1,2],[3,4],[5]]

// Inspect without consuming (debugging)
let result: Vec<_> = numbers.iter()
    .inspect(|x| println!("before filter: {x}"))
    .filter(|x| **x > 2)
    .inspect(|x| println!("after filter: {x}"))
    .collect();

// Peekable — look ahead without consuming
let mut iter = numbers.iter().peekable();
if iter.peek() == Some(&&1) { iter.next(); }

// scan — stateful map (like fold but yields intermediate values)
let running_sum: Vec<i32> = numbers.iter()
    .scan(0, |state, &x| { *state += x; Some(*state) })
    .collect();  // [1, 3, 6, 10, 15]
```

### Consuming Adapters

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Aggregation
let sum: i32 = numbers.iter().sum();
let product: i32 = numbers.iter().product();
let max = numbers.iter().max();              // Option<&i32>
let min = numbers.iter().min();
let count = numbers.iter().filter(|x| **x > 2).count();

// Searching
let has_even = numbers.iter().any(|x| x % 2 == 0);        // bool
let all_pos = numbers.iter().all(|x| *x > 0);             // bool
let first_big = numbers.iter().find(|x| **x > 3);         // Option<&&i32>
let position = numbers.iter().position(|x| *x == 3);      // Option<usize>

// fold — the most general consuming adapter
let csv = numbers.iter().fold(String::new(), |mut acc, x| {
    if !acc.is_empty() { acc.push(','); }
    acc.push_str(&x.to_string());
    acc
});

// try_fold — fold that short-circuits on error
let result = numbers.iter().try_fold(0i32, |acc, &x| {
    acc.checked_add(x).ok_or("overflow")
});

// for_each — side effects (prefer for loop for readability)
numbers.iter().for_each(|x| println!("{x}"));

// Collecting Results — short-circuits on first Err
let results: Result<Vec<i32>, _> = ["1", "2", "3"].iter()
    .map(|s| s.parse::<i32>())
    .collect();

// Collect into HashMap
use std::collections::HashMap;
let map: HashMap<&str, usize> = ["hello", "world"].iter()
    .map(|s| (*s, s.len()))
    .collect();

// Unzip into two collections
let (names, ages): (Vec<&str>, Vec<u32>) = [("Alice", 30), ("Bob", 25)]
    .iter().copied().unzip();

// partition by predicate
let (evens, odds): (Vec<i32>, Vec<i32>) = numbers.into_iter()
    .partition(|x| x % 2 == 0);
```

### zip, enumerate, and Combining

```rust
let names = vec!["Alice", "Bob", "Charlie"];
let scores = vec![95, 87, 92];

// enumerate — index + value
for (i, name) in names.iter().enumerate() {
    println!("{i}: {name}");
}

// zip — pair elements from two iterators
let ranked: Vec<_> = names.iter().zip(scores.iter()).collect();
// [("Alice", 95), ("Bob", 87), ("Charlie", 92)]

// zip stops at shortest — safe by design
let short = vec![1, 2];
let long = vec!["a", "b", "c", "d"];
let pairs: Vec<_> = short.iter().zip(long.iter()).collect(); // [(1,"a"), (2,"b")]
```

### Custom Iterator

```rust
struct Counter { count: u32, max: u32 }

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count < self.max { self.count += 1; Some(self.count) } else { None }
    }
}

let sum: u32 = Counter { count: 0, max: 5 }.sum(); // 15

// Once you implement Iterator, you get 70+ methods free:
// map, filter, fold, sum, take, skip, chain, zip, enumerate, ...
```

### Iterator Performance Patterns

```rust
// Pre-allocate when collecting into Vec
let result: Vec<_> = Vec::with_capacity(items.len());
// Or use size_hint via collect (automatic for ExactSizeIterator)

// Avoid: repeated allocation
let mut result = Vec::new();
for item in items { result.push(transform(item)); }  // May reallocate many times
// Prefer: single allocation via collect
let result: Vec<_> = items.iter().map(transform).collect();

// Use Iterator::by_ref() to partially consume
let mut iter = numbers.iter();
let first_two: Vec<_> = iter.by_ref().take(2).collect();  // [1, 2]
let rest: Vec<_> = iter.collect();  // [3, 4, 5] — continues where left off
```

### Closures

```rust
// Fn traits: Fn (borrow) ⊂ FnMut (mut borrow) ⊂ FnOnce (consume)
let add = |a, b| a + b;                    // Fn
let mut count = 0;
let mut inc = || { count += 1; };          // FnMut
let consume = move || println!("{count}"); // FnOnce (owns count)

// Return closures
fn make_adder(x: i32) -> impl Fn(i32) -> i32 { move |y| x + y }
fn make_op(add: bool) -> Box<dyn Fn(i32, i32) -> i32> {
    if add { Box::new(|a, b| a + b) } else { Box::new(|a, b| a - b) }
}

// move closures — take ownership of captured variables
let name = String::from("Alice");
let greet = move || println!("Hello, {name}!");  // name moved into closure
// Can't use `name` here anymore

// Closure as function parameter
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(f(x)) }
let result = apply_twice(|x| x + 3, 7);  // 13
```

> **Deep dive:** [language-patterns.md](language-patterns.md) — custom iterators, IntoIterator implementations,
> iterator composition patterns, closure capture semantics (Fn/FnMut/FnOnce hierarchy).
> [data-structures.md](data-structures.md) — iterator fusion, SIMD-friendly iteration with rayon.
> [async-patterns.md](async-patterns.md) — async iterators, Stream trait, tokio StreamExt.

## Collections & Data Structures

| Type | Use Case | Lookup | Insert |
|------|----------|--------|--------|
| `Vec<T>` | Ordered sequence, stack | O(n) | O(1) amortized push |
| `HashMap<K, V>` | Key-value lookup | O(1) avg | O(1) avg |
| `BTreeMap<K, V>` | Sorted key-value, range queries | O(log n) | O(log n) |
| `HashSet<T>` | Membership testing | O(1) avg | O(1) avg |
| `VecDeque<T>` | Double-ended queue | O(1) ends | O(1) ends |
| `BinaryHeap<T>` | Priority queue | O(1) peek | O(log n) |

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key", 42);
let val = map.get("key");           // Option<&V>
map.entry("key").or_insert(0);      // Insert if absent
*map.entry("counter").or_insert(0) += 1; // Increment pattern

// Entry API — and_modify + or_insert
let mut word_count: HashMap<&str, usize> = HashMap::new();
for word in text.split_whitespace() {
    word_count.entry(word)
        .and_modify(|count| *count += 1)
        .or_insert(1);
}

// Entry with lazy initialization
let mut connections: HashMap<String, Connection> = HashMap::new();
let conn = connections.entry(host.to_string())
    .or_insert_with(|| {
        Connection::new(&host)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("connection failed")
    });
conn.send(message)?;
```

> **Deep dive:** [data-structures.md](data-structures.md) — Vec internals (capacity, reallocation),
> HashMap implementation (hashing, load factor), BTreeMap for range queries, VecDeque ring buffer,
> BinaryHeap patterns, custom Hash implementations, IndexMap for insertion-order preservation,
> SmallVec/ArrayVec for stack-allocated small collections, performance comparison benchmarks.

## Strings & Slices

```rust
// &str — borrowed string slice (view into UTF-8 bytes)
// String — owned, heap-allocated, growable
let s: &str = "hello";                    // String literal ('static)
let owned: String = s.to_string();        // or String::from(s)
let borrowed: &str = &owned;              // Deref coercion

// Common operations
let mut s = String::with_capacity(100);   // Pre-allocate
s.push_str("hello");
s.push('!');
let combined = format!("{} {}", s, "world"); // Neither moved

// OsString/Path for file system
use std::path::{Path, PathBuf};
let path = Path::new("/tmp/file.txt");
let ext = path.extension();               // Option<&OsStr>
let mut buf = PathBuf::from("/tmp");
buf.push("file.txt");
```

> **Deep dive:** [quick-reference.md](quick-reference.md) — comprehensive String methods (split, trim, replace,
> case, pad, find, chars, bytes), Path/PathBuf operations, OsString conversion patterns.

## Serde Essentials

### Rules for Serde (LLM)

1. **ALWAYS use `#[serde(rename_all = "camelCase")]`** on structs sent to/from JavaScript/JSON APIs — Rust uses snake_case, JS uses camelCase.
2. **ALWAYS use `#[serde(skip_serializing_if = "Option::is_none")]`** on `Option` fields — omit absent fields rather than serializing `null`.
3. **ALWAYS use `#[serde(default)]`** on fields that may be absent in input — provides Default::default() rather than failing deserialization.
4. **PREFER internally-tagged enums** (`#[serde(tag = "type")]`) for most API enums — produces `{"type": "variant", ...}` which is cleaner than externally-tagged `{"variant": {...}}`.
5. **ALWAYS derive both `Serialize` and `Deserialize`** unless there's a specific reason not to — asymmetric serde is confusing and error-prone.
6. **PREFER `Cow<'a, str>` over `String`** in deserialization-heavy types — enables zero-copy deserialization when possible.

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    user_name: String,                          // serializes as "userName"
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,                 // omitted if None
    #[serde(default)]
    is_active: bool,                            // defaults to false if missing
    #[serde(rename = "type")]
    kind: String,                               // field named "type" in JSON
    #[serde(flatten)]
    metadata: HashMap<String, serde_json::Value>, // inline remaining fields
}

// Enum representations
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]                          // internally tagged
enum Event {
    #[serde(rename = "click")]
    Click { x: i32, y: i32 },
    #[serde(rename = "key")]
    KeyPress { key: String },
}
// Produces: {"type": "click", "x": 10, "y": 20}

// Custom deserialization for sensitive data
#[derive(Serialize, Deserialize)]
struct Config {
    #[serde(deserialize_with = "deserialize_duration_secs")]
    timeout: Duration,
}
```

> **Deep dive:** [serde-patterns.md](serde-patterns.md) — custom Serialize/Deserialize (Visitor pattern),
> all enum representations (externally tagged, internally tagged, adjacently tagged, untagged),
> format-specific patterns (JSON, TOML, bincode, CSV), zero-copy deserialization with `Cow<'a, str>`,
> `#[serde(flatten)]` for catch-all fields, `#[serde(with)]` for custom field serialization,
> deserialize_with helpers, `serde_json::Value` for dynamic JSON.

## Async/Await Core

### Rules for Async Rust (LLM)

1. **NEVER block the async runtime** — no `std::thread::sleep()`, no synchronous file I/O, no CPU-heavy computation in async functions. Use `tokio::time::sleep()`, `tokio::fs`, and `tokio::task::spawn_blocking()`.
2. **ALWAYS handle JoinHandle results** — `tokio::spawn` returns a JoinHandle. Store it, await it, or use `JoinSet`. Dropping it silently detaches the task.
3. **ALWAYS use bounded channels** (`mpsc::channel(N)`) — unbounded channels can OOM under load. Size the buffer based on expected backpressure.
4. **NEVER hold a `MutexGuard` across an `.await` point** — this blocks the runtime thread. Lock, extract data, drop the guard, then await.
5. **PREFER `tokio::select!` with cancel safety** — understand which futures are cancel-safe. `mpsc::Receiver::recv()` is cancel-safe; `read_to_string()` is not.
6. **PREFER structured concurrency** — use `JoinSet` or `tokio::join!` over raw `tokio::spawn`. Track and await all spawned tasks.
7. **ALWAYS use `spawn_blocking` for CPU-bound work** — keeps the async executor free for I/O tasks. Threshold: >1ms of CPU work.
8. **PREFER channels over shared state** — channels provide natural backpressure and don't risk deadlocks. Use `Arc<Mutex<T>>` only when you need synchronous shared state.

```rust
// async fn returns impl Future<Output = T>
async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

// tokio runtime — async web services
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let result = fetch_data("https://api.example.com").await?;
    Ok(())
}

// ExitCode from main — CLI tools needing specific exit codes (ripgrep pattern)
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,       // Found matches
        Ok(false) => ExitCode::from(1),      // No matches
        Err(err) => {
            // Handle broken pipe gracefully (common when piping to head/less)
            if is_broken_pipe(&err) { return ExitCode::SUCCESS; }
            eprintln!("Error: {err:#}");
            ExitCode::from(2)                // Runtime error
        }
    }
}

// Spawn concurrent tasks
let handle = tokio::spawn(async { expensive_work().await });
let result = handle.await?;

// Run multiple futures concurrently
let (a, b) = tokio::join!(fetch("url1"), fetch("url2"));

// Race — first to complete wins
tokio::select! {
    result = fetch("url1") => handle(result),
    _ = tokio::time::sleep(Duration::from_secs(5)) => timeout(),
}

// JoinSet for dynamic task management
let mut set = tokio::task::JoinSet::new();
for url in urls { set.spawn(async move { fetch(&url).await }); }
while let Some(result) = set.join_next().await { process(result?)?; }
```

### Async Closures (Rust 2024 Edition)

```rust
// edition = "2024" enables async closures natively
let fetch = async |url: &str| -> Result<String, Error> {
    reqwest::get(url).await?.text().await.map_err(Into::into)
};

// Useful with higher-order async functions
async fn retry<F, Fut, T, E>(f: F, attempts: u32) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    for _ in 0..attempts {
        if let Ok(v) = f().await { return Ok(v); }
    }
    f().await
}
```

### NEVER Block the Runtime

```rust
// BAD: blocks the async runtime thread
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));    // BLOCKS!
    std::fs::read_to_string("file.txt").unwrap();  // BLOCKS!
}

// GOOD: use async equivalents
async fn good() -> Result<()> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    tokio::fs::read_to_string("file.txt").await?;
    Ok(())
}

// For CPU-bound work, use spawn_blocking
let result = tokio::task::spawn_blocking(|| {
    heavy_computation()
}).await?;
```

> **Deep dive:** [async-patterns.md](async-patterns.md) — tokio runtime internals (work stealing,
> cooperative scheduling), **Pin/Unpin** explained (why futures must be pinned, self-referential types),
> **channels** (mpsc, broadcast, watch, oneshot — patterns, sizing, backpressure), **rayon** for CPU-bound
> parallelism (par_iter, join, scope), **Tower** service pattern (Service trait, Layer, middleware),
> actor patterns (manual and with actix), structured concurrency, graceful shutdown patterns,
> async closures (2024 edition), Stream trait and StreamExt.

## Concurrency Primitives

### When to Use Which Concurrency Primitive

| Need | Use | Why |
|------|-----|-----|
| Shared counter/flag | `AtomicUsize` / `AtomicBool` | Lock-free, cheapest option |
| Shared state, mostly reads | `Arc<RwLock<T>>` | Many concurrent readers, one writer |
| Shared state, frequent writes | `Arc<Mutex<T>>` or `parking_lot::Mutex` | Simpler than RwLock, less overhead |
| Concurrent HashMap | `dashmap::DashMap` | Sharded, no global lock |
| Producer-consumer | `tokio::sync::mpsc` | Bounded channel with backpressure |
| Broadcast to many consumers | `tokio::sync::broadcast` | Each subscriber gets every message |
| Latest-value config | `tokio::sync::watch` | Receivers see most recent value |
| One-shot response | `tokio::sync::oneshot` | Single value, single use |
| CPU-bound parallelism | `rayon::par_iter()` | Work stealing, automatic thread pool |
| Borrow stack data in threads | `std::thread::scope` | No Arc needed, threads must finish |

```rust
use std::sync::{Arc, Mutex, RwLock};

// Arc<Mutex<T>> — shared mutable state across threads
let counter = Arc::new(Mutex::new(0));
let c = Arc::clone(&counter);
tokio::spawn(async move { *c.lock().unwrap() += 1; });

// RwLock — many readers OR one writer
let cache = Arc::new(RwLock::new(HashMap::new()));
let data = cache.read().unwrap();     // Multiple concurrent readers OK
let mut data = cache.write().unwrap(); // Exclusive writer

// Atomic types — lock-free counters
use std::sync::atomic::{AtomicUsize, Ordering};
static REQUESTS: AtomicUsize = AtomicUsize::new(0);
REQUESTS.fetch_add(1, Ordering::Relaxed);

// Send: safe to transfer between threads (most types)
// Sync: safe to share references between threads (&T is Send)
// NOT Send: Rc<T>. NOT Sync: Cell<T>, RefCell<T>

// Scoped threads — borrow stack data without Arc
std::thread::scope(|s| {
    let data = vec![1, 2, 3];
    s.spawn(|| println!("{:?}", &data[..2]));
    s.spawn(|| println!("{:?}", &data[2..]));
});

// parking_lot::Mutex — faster, no poisoning, deadlock detection in debug
use parking_lot::Mutex;
let data = Mutex::new(Vec::new());
data.lock().push(42); // No .unwrap() needed
```

> **Deep dive:** [async-patterns.md](async-patterns.md) — channel patterns with complete examples,
> actor pattern (message loop with mpsc), Mutex anti-patterns (holding across await, poisoning),
> rayon parallel iterators and scopes, tokio task management (JoinSet, CancellationToken),
> structured concurrency patterns, deadlock prevention.

## Traits & API Design

### Standard Trait Implementations

```rust
// Accept generic inputs, return concrete types
pub fn process(input: impl AsRef<str>) -> String { /* ... */ }
pub fn read_file(path: impl AsRef<Path>) -> io::Result<Vec<u8>> { /* ... */ }
pub fn set_name(&mut self, name: impl Into<String>) { self.name = name.into(); }

// Supertraits
trait Pet: Animal + Display { fn cuddle(&self); }

// From/Into for conversions
impl From<ConfigFile> for AppConfig {
    fn from(file: ConfigFile) -> Self { /* ... */ }
}
// Now: let config: AppConfig = file.into();
```

### Blanket Implementations

```rust
// Implement trait for all types satisfying bounds
impl<T: Display> ToString for T {
    fn to_string(&self) -> String { format!("{self}") }
}
// Now every Display type gets to_string() free
```

### Marker Traits: Send, Sync, Sized

```rust
// Send: safe to transfer between threads
// Sync: safe to share &T between threads
// Sized: known size at compile time (default bound on generics)
// ?Sized: may be unsized (DST) — enables &dyn Trait, &str, &[T]

fn print_ref<T: ?Sized + Display>(value: &T) { println!("{value}"); }
print_ref("hello");  // &str is unsized — works
```

### Library Authoring Patterns

Production libraries universally use these patterns (anyhow, serde_json, reqwest, axum, dashmap):

```rust
// ── Crate-level Result alias (every library does this) ──
pub type Result<T, E = Error> = core::result::Result<T, E>;
// Users write: fn load() -> mylib::Result<Config>

// ── Compile-time trait assertions (reqwest pattern) ──
// Catch Send/Sync/Clone regressions in CI — zero runtime cost
#[cfg(test)]
fn _assert_traits() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_clone<T: Clone>() {}

    assert_send::<Client>();
    assert_sync::<Client>();
    assert_clone::<Client>();
}

// ── Re-export doc control (see documentation.md for full guide) ──
#[doc(inline)]                  // Show in this crate's docs
pub use self::extract::Json;

#[doc(no_inline)]               // Link to original crate's docs
pub use http::StatusCode;

#[doc(hidden)]                  // Hide from docs (macro internals)
pub mod __private { /* used by proc macros */ }

// ── docs.rs feature badges ──
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub fn from_json<T: DeserializeOwned>(s: &str) -> Result<T> { /* ... */ }

// ── Feature validation at compile time ──
#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Enable either 'postgres' or 'sqlite' feature");

// ── Different lints for test vs production (reqwest/axum pattern) ──
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]
#![cfg_attr(test, allow(clippy::print_stdout))]

// ── deny(missing_docs) on public libraries ──
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

// ── #[cold] on error paths (anyhow pattern) ──
#[cold]  // Hint: this path is rarely taken — optimize for the happy path
fn handle_error(err: Error) -> Response { /* ... */ }

// ── #[must_use] on important return values ──
#[must_use = "this builder does nothing unless .build() is called"]
pub struct ClientBuilder { /* ... */ }
```

> **Deep dive:** [architecture-patterns.md](../rust-planning/architecture-patterns.md) — trait-based dependency injection (generics vs trait objects),
> SOLID principles in Rust (single responsibility, open-closed, Liskov, interface segregation, dependency inversion),
> blanket implementations, extension traits for foreign types, API design guidelines.
> [type-system.md](type-system.md) — sealed traits, object safety rules, marker traits.

## Modules & Cargo

### Module System

```rust
// src/lib.rs
mod config;           // loads src/config.rs or src/config/mod.rs
pub mod api;          // public module
pub(crate) mod internal; // visible within crate only

// Re-exports for clean public API
pub use config::Config;
pub use api::{Client, Response};

// Prelude module
pub mod prelude {
    pub use crate::{Config, Client, Error};
}

// Feature-gated modules (axum pattern) — entire modules conditionally compiled
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "json")]
pub use json::Json;

// Facade crate pattern (ripgrep pattern) — re-export subcrates through one entry point
// grep/src/lib.rs re-exports grep-cli, grep-matcher, grep-printer, grep-regex, grep-searcher
pub use grep_cli as cli;
pub use grep_matcher as matcher;
#[cfg(feature = "pcre2")]
pub use grep_pcre2 as pcre2;
pub use grep_printer as printer;
// Users depend on `grep` and get all subcrates — version management in one place

// #![forbid(unsafe_code)] — use in pure-safe crates (axum enforces this workspace-wide)
// Put in lib.rs to guarantee no unsafe anywhere in the crate
#![forbid(unsafe_code)]
```

### Cargo Workspaces

```toml
# Cargo.toml (workspace root)
[workspace]
members = ["crates/*"]
resolver = "2"  # REQUIRED for workspaces — enables v2 feature resolution

[workspace.dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
thiserror = "2"
anyhow = "1"

# Workspace-wide lints (axum pattern) — centralize lint config
[workspace.lints.rust]
missing_docs = "warn"
missing_debug_implementations = "warn"
unreachable_pub = "warn"

[workspace.lints.clippy]
dbg_macro = "warn"
print_stdout = "warn"
needless_pass_by_value = "warn"
# Allow type_complexity for generic-heavy code (axum/tower pattern)
type_complexity = "allow"

# crates/core/Cargo.toml
[dependencies]
serde.workspace = true    # Inherits version from workspace
thiserror.workspace = true

[lints]
workspace = true          # Inherit workspace lint configuration
```

### Feature Flags

```toml
[features]
default = ["json"]
json = ["dep:serde_json"]
postgres = ["dep:sqlx"]
full = ["json", "postgres"]

[dependencies]
serde_json = { version = "1.0", optional = true }
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"], optional = true }
```

```rust
#[cfg(feature = "json")]
pub fn from_json(s: &str) -> Result<Config> { /* ... */ }

#[cfg(feature = "postgres")]
pub mod postgres;
```

### Build Scripts & Profiles

```toml
# Cargo.toml
[profile.release]
lto = true            # Link-time optimization (thin LTO — fast builds)
debug = 1             # Basic debug symbols (useful for profiling)

[profile.dev]
opt-level = 0         # Fast compile
debug = true          # Full debug info

[profile.dev.package."*"]
opt-level = 2         # Optimize dependencies even in dev

# Custom profile with inheritance (ripgrep pattern)
# Use: cargo build --profile release-lto
[profile.release-lto]
inherits = "release"
lto = "fat"           # Full LTO — maximum optimization, slow compile
codegen-units = 1     # Better optimization (default is 16)
strip = true          # Strip debug symbols
panic = "abort"       # Smaller binary, no unwinding
debug-assertions = false
overflow-checks = false

# Debian packaging profile inheriting from release-lto
[profile.deb]
inherits = "release-lto"
```

### Project Layout Essentials

```
# Single crate (start here — split only when you have a reason)
my_app/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root (optional — add when you need tests/benches)
│   ├── config.rs         # Configuration
│   ├── models.rs         # Domain types
│   ├── handlers.rs       # HTTP/CLI handlers
│   ├── db.rs             # Database access
│   └── errors.rs         # Error types
└── tests/
    └── integration.rs    # Integration tests

# When to split into workspace: >10K lines, separate deploy targets,
# or team boundaries. See rust-planning/workspace-layout.md for workspace layouts.
```

### Trait-Based Dependency Injection (Key Pattern)

```rust
// Define trait in domain layer
pub trait UserRepo: Send + Sync {
    async fn find(&self, id: u64) -> Result<User, RepoError>;
    async fn save(&self, user: &User) -> Result<(), RepoError>;
}

// Implement in infrastructure layer
pub struct PgUserRepo { pool: PgPool }
impl UserRepo for PgUserRepo {
    async fn find(&self, id: u64) -> Result<User, RepoError> { /* sqlx query */ }
    async fn save(&self, user: &User) -> Result<(), RepoError> { /* sqlx insert */ }
}

// Inject via generics (zero-cost) or trait objects (flexible)
pub struct UserService<R: UserRepo> { repo: R }
// OR
pub struct UserService { repo: Arc<dyn UserRepo> }
```

> **Deep dive — ALWAYS load rust-planning for architecture planning and refactoring:**
> [rust-planning/SKILL.md](../rust-planning/SKILL.md) — **10 architectural principles**, **44 planning rules**,
> workspace & crate organization, `[workspace.dependencies]` inheritance, feature flag architecture,
> hexagonal/clean architecture in Rust, growing architecture (3 stages: single crate → lib+bin → workspace),
> inter-component communication (channels, shared state), refactoring signals, anti-patterns catalog.
> [architecture-patterns.md](../rust-planning/architecture-patterns.md) — pattern walkthroughs.
> [architecture-examples.md](architecture-examples.md) — complete worked examples with full directory layouts (stays in rust-implementing).

## Struct & Enum Patterns

### Builder Pattern

Two styles: **consuming** (method chains, ergonomic) and **borrow-based** (reusable builder, ripgrep pattern):

```rust
// Consuming builder — methods take `self` and return `Self` (common for config)
#[derive(Default)]
struct ServerConfigBuilder {
    host: Option<String>,
    port: Option<u16>,
    max_conn: Option<usize>,
}

impl ServerConfigBuilder {
    fn host(mut self, host: impl Into<String>) -> Self { self.host = Some(host.into()); self }
    fn port(mut self, port: u16) -> Self { self.port = Some(port); self }
    fn max_conn(mut self, n: usize) -> Self { self.max_conn = Some(n); self }

    fn build(self) -> Result<ServerConfig, &'static str> {
        Ok(ServerConfig {
            host: self.host.ok_or("host required")?,
            port: self.port.unwrap_or(8080),
            max_conn: self.max_conn.unwrap_or(100),
        })
    }
}
// Usage: ServerConfigBuilder::default().host("0.0.0.0").port(3000).build()?

// Borrow-based builder — methods take `&mut self` and return `&mut Self`
// (ripgrep SearchWorkerBuilder pattern — builder is reusable, methods can be fallible)
struct SearchWorkerBuilder {
    search_zip: bool,
    binary_detection: BinaryDetection,
}

impl SearchWorkerBuilder {
    fn new() -> Self { Self { search_zip: false, binary_detection: BinaryDetection::default() } }
    fn search_zip(&mut self, yes: bool) -> &mut Self { self.search_zip = yes; self }
    fn preprocessor(&mut self, cmd: &Path) -> anyhow::Result<&mut Self> {
        // Fallible — resolves binary path, can fail
        let _resolved = resolve_binary(cmd)?;
        Ok(self)
    }
    fn build(&self) -> SearchWorker { /* construct from &self */ }
}
// Usage: let mut builder = SearchWorkerBuilder::new();
// builder.search_zip(true); builder.preprocessor(cmd)?; let worker = builder.build();
```

### Composable Query/Filter Pattern

Common in search engines (tantivy), query builders, and filter systems — compose trait objects into boolean/logical trees:

```rust
// Collect heterogeneous query types via trait objects, combine with boolean logic
use std::fmt::Debug;

trait Query: Debug { fn matches(&self, doc: &Document) -> bool; }

// Build query clauses dynamically from user input
let mut clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

for field in &search_fields {
    let query = FuzzyTermQuery::new(field, &text, distance);
    clauses.push((Occur::Should, Box::new(query)));  // OR semantics
}
if let Some(filter) = category_filter {
    clauses.push((Occur::Must, Box::new(ExactQuery::new("category", filter))));
}

let combined = BooleanQuery::new(clauses);
// This is Strategy + Composite via trait objects — each clause is independent,
// BooleanQuery composes them with AND/OR/NOT semantics (Occur::Must/Should/MustNot)
```

### Newtype Pattern

```rust
// Type safety — prevents mixing up IDs
struct UserId(u64);
struct OrderId(u64);

fn process(user: UserId, order: OrderId) { /* can't swap them */ }

// Validated newtype — parse, don't validate
struct Email(String);
impl Email {
    fn new(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        if !value.contains('@') { return Err(ValidationError::InvalidEmail); }
        Ok(Email(value))
    }
    fn as_str(&self) -> &str { &self.0 }
}

// #[repr(transparent)] — same memory layout as inner type (used by anyhow, serde)
// Enables safe transmutes and FFI compatibility for single-field wrappers
#[repr(transparent)]
struct Wrapper<T>(T);
```

### Enum Dispatch

```rust
// Enum dispatch is faster than dyn Trait (no vtable indirection)
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => std::f64::consts::PI * r * r,
            Shape::Rectangle(w, h) => w * h,
        }
    }
}

// #[non_exhaustive] — allows adding variants without breaking downstream
#[non_exhaustive]
pub enum Error {
    NotFound,
    PermissionDenied,
}
```

> **Deep dive:** [domain-patterns.md](../rust-planning/domain-patterns.md) — DDD entities and value objects (newtype pattern),
> aggregates with invariant enforcement, enum-based state machines, command pattern, repository trait design.
> [architecture-examples.md](architecture-examples.md) — builder pattern with validation, configuration structs.

## Smart Pointers

| Type | Use Case | Thread-Safe? |
|------|----------|-------------|
| `Box<T>` | Heap allocation, recursive types, trait objects | Send if T: Send |
| `Rc<T>` | Multiple owners, single-threaded | No |
| `Arc<T>` | Multiple owners, thread-safe | Yes |
| `RefCell<T>` | Interior mutability, single-threaded | No |
| `Mutex<T>` | Interior mutability, thread-safe | Yes |
| `RwLock<T>` | Many readers OR one writer | Yes |
| `Cow<T>` | Clone-on-write, avoid allocation | Depends on T |
| `Pin<P>` | Prevent moves (required for self-referential futures) | Depends on P |

```rust
// Box — recursive types
enum List { Cons(i32, Box<List>), Nil }

// Pin — prevents the value from being moved in memory
// Required by async/await runtime for self-referential futures
use std::pin::Pin;
let pinned: Pin<Box<dyn Future<Output = i32>>> = Box::pin(async { 42 });
```

> **Deep dive:** [type-system.md](type-system.md) — Pin/Unpin internals (why async futures
> are self-referential, pin projection, Unpin auto-trait), when to use Box vs Rc vs Arc,
> interior mutability patterns (Cell, RefCell, OnceCell), custom smart pointer implementations.

## Common Mistakes (BAD/GOOD)

```rust
// BAD: unwrap in production         | GOOD: propagate with context
let user = db.find(id).unwrap();     | let user = db.find(id).context("finding user")?;

// BAD: String errors                | GOOD: typed errors with thiserror
fn parse(s: &str) -> Result<C, String> | #[derive(thiserror::Error)] enum ParseError { ... }

// BAD: clone to satisfy borrow checker | GOOD: borrow the data
fn f(data: &Vec<String>) -> String { | fn f(data: &[String]) -> String {
    data.clone().join(", ")          |     data.join(", ")
}                                    | }

// BAD: manual index loop            | GOOD: iterator chain
for i in 0..items.len() {           | let names: Vec<_> = items.iter()
    if items[i].active {             |     .filter(|i| i.active)
        results.push(items[i].name); |     .map(|i| &i.name)
    }                                |     .collect();
}                                    |

// BAD: blocking in async            | GOOD: use async equivalents
std::thread::sleep(dur);            | tokio::time::sleep(dur).await;
std::fs::read_to_string(p).unwrap()  | tokio::fs::read_to_string(p).await?

// BAD: primitive obsession          | GOOD: newtype wrappers
fn create(user_id: u64, order: u64)  | fn create(user: UserId, order: OrderId)

// BAD: unsafe without comment       | GOOD: document safety invariant
unsafe { ptr::read(addr) }          | // SAFETY: addr valid, aligned, initialized
                                     | unsafe { ptr::read(addr) }

// BAD: Vec for lookups              | GOOD: HashMap for O(1) lookup
items.iter().find(|u| u.id == id)    | items.get(&id)

// BAD: Rc across threads            | GOOD: Arc for thread-safe ref counting
let data = Rc::new(vec![1, 2, 3]);   | let data = Arc::new(vec![1, 2, 3]);

// BAD: String concat with +         | GOOD: push_str or format!
result = result + &part;             | result.push_str(&part);

// BAD: global mutable state         | GOOD: LazyLock or dependency injection
static mut CONFIG: Option<C> = None; | static CONFIG: LazyLock<C> = LazyLock::new(|| { ... });

// BAD: match when if-let suffices   | GOOD: if let
match opt { Some(v) => f(v), None => {} } | if let Some(v) = opt { f(v); }

// BAD: overengineered traits        | GOOD: start with a function
trait DataProcessor<I,O,E> { ... }   | fn process(input: &str) -> String { ... }

// BAD: large unsafe blocks          | GOOD: minimal unsafe, safe wrapper
unsafe { /* 50 lines */ }           | fn safe_wrapper(p: *const u8, n: usize) -> &[u8] {
                                     |     // SAFETY: caller guarantees validity
                                     |     unsafe { std::slice::from_raw_parts(p, n) }
                                     | }
```

### Extended BAD/GOOD Examples

**Excessive boolean parameters → config struct:**
```rust
// BAD: unclear call sites
fn process(data: &str, verbose: bool, validate: bool, cache: bool) { ... }
process(data, true, false, true);  // What do these bools mean?

// GOOD: configuration struct with defaults
struct ProcessOptions { verbose: bool, validate: bool, cache: bool }
impl Default for ProcessOptions {
    fn default() -> Self { Self { verbose: false, validate: true, cache: true } }
}
fn process(data: &str, opts: ProcessOptions) { ... }
process(data, ProcessOptions { verbose: true, ..Default::default() });
```

**Stringly-typed APIs → type-safe keys:**
```rust
// BAD: magic strings, no compile-time check
fn get_config(key: &str) -> Option<String> {
    match key { "database.host" => Some("localhost".into()), _ => None }
}
let host = get_config("databse.host");  // Typo compiles fine!

// GOOD: enum keys — typos caught at compile time
enum ConfigKey { DatabaseHost, DatabasePort }
fn get_config(key: ConfigKey) -> String {
    match key {
        ConfigKey::DatabaseHost => "localhost".into(),
        ConfigKey::DatabasePort => "5432".into(),
    }
}
```

**Deadlock from inconsistent lock ordering:**
```rust
// BAD: threads acquire locks in different order → deadlock
fn transfer(from: &Mutex<Account>, to: &Mutex<Account>, amount: f64) {
    let mut from_lock = from.lock().unwrap();  // Thread 1: locks A
    let mut to_lock = to.lock().unwrap();      // Thread 1: waits for B
    // Thread 2 might lock B then wait for A → DEADLOCK
    from_lock.balance -= amount;
    to_lock.balance += amount;
}

// GOOD: consistent lock ordering by ID
fn transfer(from: &Arc<Account>, to: &Arc<Account>, amount: f64) {
    let (first, second) = if from.id < to.id { (from, to) } else { (to, from) };
    let mut first_lock = first.balance.lock().unwrap();
    let mut second_lock = second.balance.lock().unwrap();
    if from.id < to.id {
        *first_lock -= amount; *second_lock += amount;
    } else {
        *second_lock -= amount; *first_lock += amount;
    }
}
```

**Public error enum without `#[non_exhaustive]`:**
```rust
// BAD: adding a variant is a breaking change for downstream match arms
#[derive(Debug)]
pub enum ErrorKind {
    Io(std::io::Error),
    Parse(String),
}

// GOOD: #[non_exhaustive] allows adding variants without breaking callers (ripgrep pattern)
#[derive(Debug)]
#[non_exhaustive]
pub enum ErrorKind {
    Io(std::io::Error),
    Parse(String),
}
// Callers MUST use `_ =>` wildcard arm — new variants won't break them
```

**Channel send losing the unsent value:**
```rust
// BAD: value is dropped on send failure — caller loses the data
fn send(tx: &Sender<Message>, msg: Message) -> Result<(), Error> {
    tx.send(msg).map_err(|_| Error::Closed)?;
    Ok(())
}

// GOOD: return the value in the error so caller can retry or save (tokio pattern)
pub struct SendError<T>(pub T);
impl<T> SendError<T> {
    pub fn into_inner(self) -> T { self.0 }
}
```

**Silently ignoring Result:**
```rust
// BAD: silently drops errors
fn cleanup() {
    let _ = std::fs::remove_file("temp.txt");  // Error swallowed
    std::fs::read_to_string("config.txt");      // Unused Result warning
}

// GOOD: handle or explicitly log
fn cleanup() {
    if let Err(e) = std::fs::remove_file("temp.txt") {
        tracing::warn!("Could not remove temp file: {e}");
    }
}
```

**Use after move:**
```rust
// BAD: using value after move
let s = String::from("hello");
let s2 = s;           // s moved to s2
// println!("{s}");   // ERROR: value used after move

// GOOD: borrow instead of move
let s = String::from("hello");
let s2 = &s;          // borrow, not move
println!("{s} {s2}"); // Both valid

// GOOD: clone when you genuinely need two owned copies
let s = String::from("hello");
let s2 = s.clone();
println!("{s} {s2}");
```

**Long parameter lists → builder or parameter object:**
```rust
// BAD: 7+ parameters — impossible to read at call sites
fn create_user(name: &str, email: &str, age: u32, city: &str,
               country: &str, phone: &str, admin: bool) -> User { ... }

// GOOD: builder pattern
let user = UserBuilder::new("John", "john@example.com")
    .age(30)
    .address(Address { city: "NYC".into(), country: "USA".into() })
    .build()?;

// GOOD: parameter object
struct CreateUserRequest<'a> {
    name: &'a str, email: &'a str, age: u32,
    address: Address, phone: Option<&'a str>,
}
fn create_user(req: CreateUserRequest) -> Result<User, Error> { ... }
```

## Tracing & Observability

```rust
use tracing::{info, warn, error, instrument, span, Level};

// Setup subscriber
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter("my_app=debug,tower_http=trace")
        .with_target(true)
        .json() // Structured JSON output for production
        .init();
}

// #[instrument] adds spans automatically
#[instrument(skip(pool), fields(user_id = %id))]
async fn get_user(id: u64, pool: &PgPool) -> Result<User> {
    info!("fetching user");
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id as i64)
        .fetch_optional(pool).await?
        .ok_or(AppError::NotFound)?;
    info!(username = %user.name, "user found");
    Ok(user)
}

// Manual spans for custom scoping
let span = span!(Level::INFO, "batch_process", batch_size = items.len());
let _guard = span.enter();

// Structured fields
warn!(retry_count = 3, endpoint = "api.example.com", "request failed, retrying");
error!(error = %e, "database connection lost");
```

> **Deep dive:** [observability.md](observability.md) — tracing subscriber setup (EnvFilter, JSON formatting,
> file appenders, rolling logs), OpenTelemetry integration, custom tracing layers, span lifecycle,
> metrics with prometheus/opentelemetry, structured logging best practices. For build profiles and
> deployment pipeline planning, see [rust-planning/SKILL.md](../rust-planning/SKILL.md).

## Testing Essentials

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(add(2, 3), 5);
        assert!(is_valid("test@example.com"));
        assert_ne!(hash("a"), hash("b"));
    }

    #[test]
    fn test_error_variant() {
        let result = parse_config("");
        assert!(matches!(result, Err(ConfigError::Empty)));
    }

    #[test]
    #[should_panic(expected = "must be non-zero")]
    fn test_panic() { divide(1, 0); }

    // Async tests
    #[tokio::test]
    async fn test_async() {
        let result = fetch_data("url").await;
        assert!(result.is_ok());
    }
}

// Integration tests in tests/ directory
// tests/integration_test.rs — each file is a separate binary

// Doc tests — verified by cargo test (see documentation.md for full guide)
/// Adds two numbers.
///
/// # Examples
///
/// ```
/// assert_eq!(mylib::add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 { a + b }
```

### Test-Driven Development (TDD) in Rust

Rust's compiler acts as a **continuous verification layer** alongside TDD. The cycle is Red → Green → Refactor, but Rust adds a preliminary step: **define the types and traits first** — the compiler enforces contracts that other languages rely on tests to catch.

**TDD workflow — trait-first design:**

```rust
// STEP 1: Define the trait (contract) — this is the "design" step
pub trait OrderValidator {
    fn validate(&self, order: &Order) -> Result<(), ValidationError>;
}

// STEP 2: RED — Write a failing test against a type that doesn't exist yet
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_order_with_zero_quantity() {
        let validator = QuantityValidator;  // Doesn't exist yet — won't compile
        let order = Order { item: "bolt".into(), quantity: 0, price_cents: 500 };
        let result = validator.validate(&order);
        assert!(matches!(result, Err(ValidationError::InvalidQuantity(_))));
    }
}

// STEP 3: GREEN — Write the minimum implementation to pass
pub struct QuantityValidator;

impl OrderValidator for QuantityValidator {
    fn validate(&self, order: &Order) -> Result<(), ValidationError> {
        if order.quantity == 0 {
            return Err(ValidationError::InvalidQuantity(
                "quantity must be > 0".into()
            ));
        }
        Ok(())
    }
}

// STEP 4: REFACTOR — now add more tests (max quantity, composite validators)
// and refactor while tests stay green
```

**Key Rust TDD principles:**
- **Traits are your test seams** — define behavior as traits, inject dependencies via `impl Trait` or `Box<dyn Trait>`, swap real implementations for mocks in tests
- **The compiler is your first test** — type errors, lifetime violations, and exhaustive match checks catch entire bug classes before tests even run
- **Test error variants specifically** — `assert!(matches!(result, Err(MyError::Specific(_))))` not just `is_err()`
- **Use `cargo watch -x test`** — automatic re-run on save for tight feedback loops

> **Deep dive:** [testing-patterns.md](testing-patterns.md) — **TDD workflow** (complete red-green-refactor examples,
> trait-first design, growing a module test-first, async TDD, when TDD doesn't fit Rust),
> **mockall** (mock traits, expectations, sequences), **insta** (snapshot testing for JSON, YAML,
> debug output), **proptest** (property-based testing, arbitrary generators, shrinking),
> **cargo-fuzz** (fuzzing with libfuzzer, arbitrary crate), async test patterns (tokio::test,
> test timeouts), database test fixtures (sqlx test databases), E2E testing patterns,
> test organization (unit vs integration vs doc tests).

## Performance — writing performant code

When writing performance-sensitive code, these are the implementation-side levers:

- Pre-allocate with `Vec::with_capacity()` / `HashMap::with_capacity()` when you know the size
- Take `&str` / `&[T]` instead of `String` / `Vec<T>` in function params unless you need ownership
- Avoid `.clone()` to "silence" the borrow checker — restructure ownership instead (see §Rules rule 17)
- `SmallVec` for small-N collections that rarely grow
- `Box<dyn Trait>` has vtable overhead in hot loops → monomorphize with generics or use enum dispatch
- Hold `MutexGuard` for the shortest possible scope; never across `.await`
- For parallelism on CPU-bound work, use `rayon::par_iter()`; for concurrent I/O, use Tokio

**Measuring / benchmarking / profiling is a reviewing-phase activity.** Don't optimize blind:

- [rust-reviewing/profiling-playbook.md](../rust-reviewing/profiling-playbook.md) — criterion/iai/divan micro-bench setup; flamegraph/samply/perf for CPU; DHAT/heaptrack for heap; tokio-console for async; compile-time profiling
- [rust-reviewing/performance-catalog.md](../rust-reviewing/performance-catalog.md) — 25+ common pitfalls with symptom → root cause → fix
- [data-structures.md](data-structures.md) — Rust-specific data structure performance patterns (SmallVec, indexmap, capacity pre-allocation)

## Rust 2024 Edition

New features in `edition = "2024"`:

| Feature | Description |
|---------|-------------|
| **Async closures** | `async \|\| { ... }` — native async closures without workarounds |
| **RPIT captures** | `-> impl Trait` captures all in-scope lifetimes by default |
| **`unsafe extern` blocks** | `unsafe extern "C" { ... }` — extern blocks must be marked unsafe |
| **`unsafe_op_in_unsafe_fn`** | Unsafe ops in unsafe fn bodies must be in explicit `unsafe { }` blocks |
| **Reserved syntax** | `gen` keyword reserved for future generators |

```rust
// edition = "2024"

// Async closures — first-class support
let fetch = async |url: &str| {
    reqwest::get(url).await?.text().await
};

// RPIT captures all lifetimes by default
fn process(data: &[u8]) -> impl Iterator<Item = u8> + '_ {
    // In 2024, the '_ is implicit — captures &[u8]'s lifetime automatically
    data.iter().copied().filter(|b| *b != 0)
}

// Unsafe extern blocks
unsafe extern "C" {
    fn external_function(ptr: *const u8, len: usize) -> i32;
}
```

## Quick Reference — Top ~100 Daily Patterns

The absolute most-used snippets. Anything deeper lives in **[quick-reference.md](quick-reference.md)** — String, Vec, HashMap, Option, Result, io, fs, env, Path, Time, Tokio, serde, regex, Display/FromStr/From/Into/AsRef/Deref, macros, cfg attrs, derives, reqwest, chrono, anyhow/thiserror, plus type conversions and OnceLock patterns.

### Type conversions (most-needed)

```rust
let s: String = "hello".to_string();   // &str -> String
let s: &str  = &my_string;              // String -> &str (Deref)
let n: i32   = "42".parse()?;           // &str -> number
let s: String = 42.to_string();         // number -> String
let opt = result.ok();                  // Result -> Option
let res = opt.ok_or(Error::Missing)?;   // Option -> Result
let bytes: &[u8] = s.as_bytes();        // &str -> &[u8]
let s = std::str::from_utf8(bytes)?;    // &[u8] -> &str
```

### Common derives

```rust
#[derive(Debug)]                         // {:?} formatting
#[derive(Clone, Copy)]                   // Value semantics (Copy requires Clone)
#[derive(PartialEq, Eq, Hash)]          // HashMap/HashSet keys
#[derive(PartialOrd, Ord)]               // Sorting
#[derive(Default)]                        // Default::default()
#[derive(Serialize, Deserialize)]        // serde
#[derive(thiserror::Error)]              // Library error types
```

### LazyLock / OnceLock one-liners

```rust
use std::sync::{LazyLock, OnceLock};
static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+$").unwrap());
static POOL: OnceLock<PgPool> = OnceLock::new();  // .set() once, .get() thereafter
```
(Full patterns — DB pool init, Tokio runtime for NIF/FFI, `get_or_init` — in [quick-reference.md](quick-reference.md).)

### Option essentials

```rust
opt.map(|x| x.len())                    // Option<T> -> Option<U>
opt.and_then(|x| x.parse().ok())       // flat_map — avoids Option<Option<_>>
opt.unwrap_or_default()                 // T or Default
opt.unwrap_or_else(|| compute())        // T or lazy default
opt.ok_or(Error::Missing)?             // convert to Result and propagate
opt.as_deref()                          // Option<String> -> Option<&str>
opt.is_some_and(|x| x > 0)             // bool (stable 1.70+)
opt?                                    // early-return None
```

### Result essentials

```rust
result.map(|v| v.len())                 // transform Ok
result.map_err(Into::into)              // translate error type at boundary
result.and_then(|v| parse(v))          // flat_map for Result
result.unwrap_or_else(|_| fallback())   // T or recovery
result?                                  // early-return Err
result.ok()                              // Result<T, E> -> Option<T>
let all: Result<Vec<_>, _> = items.iter().map(try_parse).collect();  // short-circuits on first Err
```

### String essentials

```rust
let s = "Hello, World!";
s.contains("World"); s.starts_with("Hello"); s.ends_with('!');
s.split(',').collect::<Vec<_>>();       // splits (also split_whitespace, lines, rsplit)
s.trim(); s.trim_start_matches('#');
s.replace("World", "Rust");
s.to_lowercase();                        // locale-aware
let mut s = String::with_capacity(256);
s.push_str("prefix:"); write!(s, " {}", 42).unwrap();  // needs std::fmt::Write
let joined = ["a", "b", "c"].join(", "); // "a, b, c"
```

### Vec essentials

```rust
let mut v = Vec::with_capacity(1000);   // pre-allocate when size known
v.push(x); v.extend([1, 2, 3]);
v.pop();                                 // Option<T>
v.swap_remove(i);                        // O(1) remove (swap with last)
v.retain(|x| *x > 0);                   // in-place filter
v.sort_by_key(|item| item.priority);   // sort by derived key
v.dedup();                               // sorted-dedup
v.binary_search(&target);                // sorted-only; Result<idx, idx>
let squared: Vec<_> = v.iter().map(|x| x * x).collect();
let (a, b) = v.split_at(mid);
```

### HashMap essentials — master the Entry API

```rust
use std::collections::HashMap;
let mut m = HashMap::with_capacity(100);
m.insert(key, val);                      // returns Option<V> (old value)
*m.entry(key).or_insert(0) += 1;        // increment counter — the #1 idiom
m.entry(key).or_insert_with(|| expensive_default());
m.entry(key).and_modify(|v| *v += 1).or_insert(1);  // modify-or-insert
m.get(&key); m.get_mut(&key); m.contains_key(&key); m.remove(&key);
let counts: HashMap<char, usize> = text.chars().fold(HashMap::new(),
    |mut m, c| { *m.entry(c).or_insert(0) += 1; m });
```

### Formatting cheatsheet

```rust
format!("{:?}", v);                      // Debug
format!("{:#?}", v);                     // pretty Debug (multi-line)
format!("{} {}", a, b);                  // Display, positional
format!("{name}: {val}", name = "x", val = 42);
format!("{:.2}", 3.14159);              // "3.14"
format!("{:>10}", 42); format!("{:0>5}", 42);   // " 42"; "00042"
format!("{:x}"); format!("{:#x}"); format!("{:08b}", 42);  // hex/bin
format!("{:?}", start.elapsed());       // Instant::elapsed() as Duration
```

> **Deeper reference:** [quick-reference.md](quick-reference.md) covers Path/PathBuf, `fs::` read/write/buffered/directory, `env::var`/`Command`, Duration/Instant/SystemTime, regex, chrono, reqwest, serde_json, Tokio sync primitives, std trait impls (Display, FromStr, From/Into, AsRef, Deref, Index, IntoIterator, Drop), cfg attributes, and the full LazyLock/OnceLock patterns including Tokio-runtime-in-NIF. [async-patterns.md](async-patterns.md) covers channels (mpsc/oneshot/broadcast/watch) and CSP pipelines.

## Related Skills

- **[rust-planning](../rust-planning/SKILL.md)** — Architectural planning: project layout (single crate → lib+bin → workspace), crate boundaries, trait placement, error strategy, async strategy, unsafe budget, test strategy, resilience. Load when starting a project, a major refactor, or when you need to decide *what to build*.
- **[rust-reviewing](../rust-reviewing/SKILL.md)** — Reviewing PRs, debugging bugs (panics, deadlocks, OOM, UB), profiling (flamegraph, perf, DHAT, tokio-console). Load when critiquing existing code or investigating a bug/perf issue.
- **[rust-nif](../rust-nif/SKILL.md)** — Rust NIFs with Rustler for Elixir. Key: use `#[rustler::nif]`, return NIF-compatible types, never block the BEAM scheduler, use `send` for long operations.
- **[rust-wasm](../rust-wasm/SKILL.md)** — Rust WebAssembly with Phoenix LiveView integration. Key: `wasm-bindgen` for JS interop, `wasm-pack` for bundling, keep WASM modules focused.
- **[c-programming](../c-programming/SKILL.md)** — C for embedded/FFI. Key: when writing FFI bindings between Rust and C, see `ffi-patterns.md` for `repr(C)`, `bindgen`, and `CString` patterns.

## References

- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- The Rust Programming Language: https://doc.rust-lang.org/book/
- Rust Edition Guide: https://doc.rust-lang.org/edition-guide/
- Rust Reference: https://doc.rust-lang.org/reference/
- Crate docs: https://docs.rs/ (serde, tokio, axum, clap, tracing, sqlx, reqwest, rayon)
- Clippy lints: https://rust-lang.github.io/rust-clippy/
- Rust Design Patterns: https://rust-unofficial.github.io/patterns/
