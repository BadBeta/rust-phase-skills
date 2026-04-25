---
name: rust-planning
description: >
  Rust architectural planning — decisions made BEFORE writing code. Covers project
  layout (single crate, lib+bin, Cargo workspace), crate boundaries, dependency
  direction, trait-based DI, hexagonal/DDD architecture, error strategy, async
  strategy (runtime choice, actor vs channels), resilience, unsafe/FFI strategy,
  test strategy (pyramid, mocking, property/fuzz), data strategy, feature flags,
  and architectural anti-patterns. Rust 2024.
  ALWAYS use when designing, architecting, structuring, or planning a Rust project.
  ALWAYS use when choosing between single-crate, lib+bin, or workspace layouts.
  ALWAYS use when deciding crate boundaries, trait placement, or dep direction.
  ALWAYS use when starting a new Rust project or major refactor.
  ALWAYS use when choosing async runtime or supervision/shutdown shape.
  For writing code, load rust-implementing. For reviewing/debugging/profiling,
  load rust-reviewing.
---

# Rust — Planning Skill

Architectural decisions made **before** writing implementation code. This skill sits upstream of `rust-implementing`: planning answers *what to build and how to structure it*; implementing answers *how to type it idiomatically*.

## About this skill family

- **rust-planning** (this) — upfront architecture: project layout, crate boundaries, dependency direction, trait placement, error strategy, async strategy, unsafe budget, resilience, testing strategy, architectural style.
- **[rust-implementing](../rust-implementing/SKILL.md)** — the moment of writing: decision tables for constructs (`?` vs `match`, `Arc<Mutex>` vs channels, `impl Trait` vs `dyn Trait`), idiomatic templates, BAD/GOOD anti-patterns Claude commonly produces, TDD, testing essentials, macro patterns.
- **[rust-reviewing](../rust-reviewing/SKILL.md)** — code inspection: review PRs, debug bugs (panics, deadlocks, OOM, UB), profile performance (flamegraph, perf, DHAT, tokio-console).

The three skills follow the skill-authoring three-modes framework. This skill leans heavily on **decision tables** (fire at the moment of designing) and **process-style rules** (constraints that fire during design review). Code templates appear mainly as project-layout shapes and composition-root patterns — the bulk of code templates lives in `rust-implementing`.

## Subskills — deep references within rust-planning

This SKILL.md covers the always-loaded rules, planning workflow, master decision tables, and quick-reference material. For depth on any topic, load the matching subskill:

| Subskill | Scope | Load when |
|---|---|---|
| [architecture-patterns.md](architecture-patterns.md) | SOLID in Rust, hexagonal, onion, clean architecture, layered design, facade crate pattern, enum-based polymorphism vs `dyn Trait`, Tower Layer/Service composition (axum), high-throughput ingestion, nanoservices | Deciding architectural style; composing layers; splitting a monolith |
| [workspace-layout.md](workspace-layout.md) | Cargo workspace organization, `[workspace.dependencies]` + `[workspace.lints]` inheritance, member crate layout, when to split crates, feature flag architecture, feature-gated server roles, visibility boundaries | Designing Cargo.toml structure; splitting a single crate into workspace |
| [domain-patterns.md](domain-patterns.md) | DDD in Rust, entities, value objects (newtype), aggregates, bounded contexts as Cargo workspaces, anti-corruption layers, domain events, event sourcing (state replay, snapshots, versioning), CQRS (read models, projections), inter-context communication (gRPC, Kafka, outbox) | Modeling a domain; deciding aggregate boundaries; event sourcing / CQRS |
| [async-strategy.md](async-strategy.md) | Sync vs async, Tokio vs async-std vs smol, single vs multi-threaded runtime, task budget, actor vs channels, structured concurrency, graceful shutdown design, when NOT to add channels | Deciding to go async; picking a runtime; designing task/channel topology |
| [error-strategy.md](error-strategy.md) | thiserror vs anyhow vs color-eyre vs miette comparison, library vs application error handling, multi-layer error translation, hand-rolled Error+ErrorKind pattern (ripgrep/tokio), error-value recovery (`SendError<T>`), uninhabited error types (`NoError`), when NOT to use `Box<dyn Error>` | Designing error types; error boundaries between layers/crates |
| [unsafe-strategy.md](unsafe-strategy.md) | When unsafe is justified, minimal unsafe surface, safety contracts, FFI strategy (bindgen vs cbindgen vs cxx), `catch_unwind` at FFI boundaries, `AbortIfPanic` guard (rayon pattern), Miri in CI, unsafe review checklist | Deciding whether to use unsafe; designing an FFI boundary |
| [services-architecture.md](services-architecture.md) | Monolith vs nanoservices vs microservices, kernel pattern, feature-gated adapters, resilience (circuit breakers, retries, backoff, idempotency), service discovery (Kubernetes, kube-rs), CAP theorem, TCP server/client architecture, TLS placement | Designing a service, multi-service architecture, resilience strategy |
| [data-strategy.md](data-strategy.md) | Store choice (PostgreSQL / MySQL / SQLite / Mongo / sled / embedded / Redis), ORM vs query builder vs raw SQL (SQLx / Diesel trade-offs), migration strategy, connection pooling, caching strategy (moka, Redis), data ownership across contexts | Choosing a data store; designing migrations; caching strategy |
| [test-strategy.md](test-strategy.md) | **FIRST-CLASS**: test pyramid for Rust, mocking strategy (trait-first, mockall), property-based testing scope (proptest / quickcheck), fuzzing scope (cargo-fuzz / afl), snapshot testing (insta), loom model checking, compile-fail tests (trybuild), CI strategy, coverage goals, E2E strategy (cargo-nextest, assert_cmd, `axum-test` crate's `TestServer`), test as design driver | Planning test infrastructure; deciding testing investment per layer |
| [distributed-rust.md](distributed-rust.md) | Multi-node patterns, gRPC (tonic) service contracts, HTTP service-to-service, message bus (Kafka/NATS), distributed consensus considerations, partition handling, idempotency across services | Designing multi-node / clustered / multi-region deployments |
| [long-running-projects.md](long-running-projects.md) | Meta-workflow for projects spanning multiple sessions and milestones: the three-document model (`PLAN.md` / `continue.md` / commit messages), milestone-boundary checklist, SSOT invariant verification, pending-items pruning, cross-session handoff quality, hibernation preparation. | Starting/resuming a long-running project; writing `continue.md`; milestone commit discipline |

**Cross-references:** subskills link to each other and to the other main skills' subskills (when they exist) via relative paths.

## How to use this skill

0. **Resuming a long-running project (M-prefix commits, `continue.md`, multi-session work)?** — **Load `long-running-projects.md` first.** It's the meta-workflow for everything below: which document records what (PLAN.md vs continue.md vs commit messages), milestone-boundary checklist, SSOT invariant verification, cross-session handoff quality. If the project has 5+ commits with `M\d+:` prefixes, this subskill is the primary reference and the rest of this SKILL.md becomes lookup material for specific design questions that come up *within* a milestone.
1. **Starting a new project?** — Read §1 (Rules), §2 (Planning Workflow), §3 (Master Decision Table), §5 (Project Layout). Walk through the decisions in sequence.
2. **Adding a new feature to an existing project?** — §2 (Planning Workflow), §3 (Master Decision Table). Check §6 (do I need a new crate?) and §8 (do I need async / channels?).
3. **Refactoring?** — §13 (Growing Architecture) to identify where you are, §14 (Refactoring Signals), §15 (Anti-patterns) to find what to fix.
4. **Choosing how two components should talk?** — §8 (Inter-Component Communication) — the mechanisms and escalation path.
5. **Designing for failure?** — §11 (Resilience) — where circuit breakers, retries, timeouts, and graceful shutdown live.
6. **Designing error types?** — §7 (Error Strategy) + `error-strategy.md`.
7. **Planning tests before writing?** — §12 (Test Strategy) + `test-strategy.md`. TDD is first-class — test the interface into existence.
8. **About to write code?** — Load `rust-implementing` alongside this skill.

---

## 1. Rules for Architecting Rust Applications (LLM)

These rules consolidate 10 architectural principles + 17 decision rules + planning-level rules from async/error/unsafe/testing. They fire during design review; the master decision table in §3 fires at the moment of deciding.

### Foundational principles

1. **ALWAYS have dependencies point inward.** Infrastructure depends on Application. Application depends on Domain. Domain depends on nothing external. A domain module must NEVER import `sqlx`, `axum`, `reqwest`, `redis`, or any infrastructure crate. `Cargo.toml` is the proof — if `domain/Cargo.toml` lists framework crates, the architecture is broken.
2. **ALWAYS treat traits as ports and implementations as adapters.** Every external dependency (database, API, email, file system, message queue) is behind a `trait` defined by the domain. Infrastructure implements the trait. Config or the composition root selects which implementation runs. This IS hexagonal architecture — Rust's trait system has it built in.
3. **ALWAYS encode layer direction in `Cargo.toml`.** `domain/Cargo.toml` has zero infrastructure deps. `infra/Cargo.toml` depends on `domain`. Dependency direction is auditable from `Cargo.toml` alone — use it as the primary enforcement mechanism, not convention.
4. **ALWAYS design for replaceability.** Can you swap a component's implementation without changing business logic? If not, introduce a trait at the boundary. Can you test a business rule without a database, HTTP server, or external service? If not, the architecture has a boundary problem.
5. **ALWAYS keep the composition root single.** `main()` (or a builder called from `main()`) creates concrete implementations, injects them into use cases, and starts the server. This is the only place that knows about all concrete types. No service discovers its own dependencies at runtime.
6. **ALWAYS start without frameworks and add them at the edges.** Domain logic is plain Rust — no `#[derive(FromRow)]`, no `#[actix_web::get]`, no framework annotations. Framework-specific code lives in the outermost layer only. Litmus test: can you delete `infrastructure/` and `api/` and still compile `domain/` and `application/`?
7. **ALWAYS translate errors at layer boundaries.** Each layer has its own error type. Domain errors are business-meaningful (`OrderNotModifiable`, `InsufficientFunds`). Infrastructure errors are technical (`ConnectionTimeout`, `RowNotFound`). `From` conversions translate between them at layer boundaries. Never surface infrastructure errors to callers.

### Project-layout and crate boundaries

8. **ALWAYS start with the simplest layout.** Single crate + modules for small apps. Promote to `src/lib.rs` + `src/main.rs` when a second binary needs the library. Promote to Cargo workspace only when crate-level boundaries are needed (multi-team, multi-binary, independent publishing, compile-time dependency enforcement). "Feels like it's getting big" is not a reason to split into a workspace.
9. **ALWAYS use `[workspace.dependencies]` and `[workspace.lints]` for multi-crate workspaces.** Centralize version pins and clippy/rustc lint configuration at the workspace level with `[lints] workspace = true` in each member. Override per-crate only when necessary.
10. **ALWAYS name modules after the domain**, not the framework (`orders`, `catalog`, `billing` — not `controllers`, `models`, `services`). Scream the domain.
11. **NEVER organize code into `models/`, `services/`, `helpers/` directories.** These are anti-patterns from other ecosystems. Rust uses crates (or modules in smaller apps) as boundary walls with `pub(crate)` for internal visibility.
12. **NEVER expose domain entities directly as API responses.** Use separate DTOs (`CreateOrderRequest`, `OrderResponse`). Domain entities carry invariants and internal state that callers should not see or depend on.

### Dependency inversion and DI

13. **ALWAYS define repository and gateway traits in the domain layer.** The domain owns the contract; infrastructure implements it. Never define a trait next to its implementation in infra.
14. **ALWAYS use constructor injection** — pass dependencies into `new()` or `build()`. Never use global mutable state, `lazy_static!` service locators, or hidden singletons for dependencies. `LazyLock` is fine for truly immutable configuration derived from environment or files, not for mutable services.
15. **PREFER generics (`impl OrderRepository`) over trait objects (`Box<dyn OrderRepository>`)** when there is only one implementation per compilation target. Generics enable monomorphization and inlining. Use `dyn Trait` when you need heterogeneous collections or plugin architectures.
16. **PREFER manual DI (constructor injection in `main()`) over DI containers** for applications with fewer than ~20 services. Containers add indirection without proportional benefit. Consider `shaku` or similar only when service graph complexity justifies it.
17. **PREFER `Arc<T>` for sharing services across async tasks over `&'static T` globals.** `Arc` makes ownership explicit and enables testing with different instances.

### Error strategy

18. **PREFER `thiserror` for published-library error types and `anyhow` (or `color-eyre`/`miette`) for application error handling.** `thiserror` gives typed errors downstream callers can pattern-match on; use it for crates you publish to crates.io or that cross organizational boundaries. `anyhow` is ergonomic for **application code broadly — not only `main.rs` but throughout internal workspace crates** (Zed uses `anyhow::Result` throughout `editor`, `language`, `project` — all internal but not meant to be reusable libraries). The two coexist: use `anyhow` for the app, `thiserror` for any embedded sub-libraries you may spin off. `miette` for user-facing diagnostic quality.
19. **NEVER use `Box<dyn Error>` in public library APIs.** Define typed error enums so callers can match on variants. `Box<dyn Error>` loses information and breaks pattern matching.
20. **NEVER use `String` for error messages in `Result`.** Typed errors (`Result<T, MyError>`) let callers match on variants and recover programmatically. String errors lose information and prevent programmatic handling.
21. **CONSIDER hand-rolled `impl Display + impl Error` with `Error { kind: ErrorKind }` wrappers** for top-tier libraries — ripgrep, tokio, hyper, and serde all take this approach. It gives full control over formatting, `#[non_exhaustive]`, and display patterns beyond what `thiserror` enables.

### Async strategy

22. **NEVER introduce async before the sync version is too slow or too blocking.** Async costs ergonomic complexity (lifetimes, `Send` bounds, pin projection, runtime selection). Use async for concurrent I/O at scale, not for "making things faster."
23. **ALWAYS pick one async runtime per binary and commit to it.** Mixing Tokio and async-std in the same binary causes subtle bugs. Tokio is the default for web/service code; `smol` for embedded and minimal deployments. **`async-std` was officially discontinued in March 2025 (v1.13.1 final); migrate off it to Tokio or smol.**
24. **ALWAYS design task topology upfront.** How many top-level tasks? Are they supervised (`JoinSet`)? How do they coordinate (channels, shared state, `Notify`)? Sketch the task graph before spawning anything.
25. **NEVER spawn fire-and-forget tasks.** Every `tokio::spawn` must have its `JoinHandle` tracked (in `JoinSet`, stored in state, or awaited). Untracked tasks that panic silently swallow errors and leak forever.
26. **ALWAYS set timeouts at every external boundary** (HTTP clients, database queries, gRPC calls, socket operations). Cascade correctly: outer > middle > inner. Otherwise outer timeouts fire before inner ones with meaningless errors.
27. **ALWAYS design graceful shutdown into the supervision structure from day one.** Parent `CancellationToken` → child tokens → drain outstanding work → close connections → exit. Retrofitting shutdown is painful.

### Unsafe and FFI strategy

28. **NEVER use unsafe without a documented reason.** Acceptable reasons: FFI boundary, performance-critical primitive (documented with benchmarks), direct hardware access (embedded), interop with a safe abstraction that requires it. Unacceptable reasons: "the borrow checker is annoying", "it's faster" (without measurement).
29. **ALWAYS isolate unsafe in a safe wrapper with a clear contract.** The public API is safe; unsafe lives inside one module with `// SAFETY:` comments on every `unsafe` block. The safe wrapper's contract is what callers see.
30. **ALWAYS run Miri in CI for crates with unsafe.** Miri catches UB that the compiler can't. Budget the CI time — Miri is slow but catches bugs no other tool finds.
31. **ALWAYS add `catch_unwind` at the FFI boundary when Rust panics would cross into non-Rust code.** Unwinding into C is UB. Convert panics to error returns at the boundary.

### Resilience

32. **ALWAYS make retryable operations idempotent** (webhook handlers, queue consumers, distributed calls, background jobs). Use idempotency keys or unique constraints. Without idempotency, retries multiply effects.
33. **NEVER place circuit breakers or retry logic in domain modules.** They belong in infrastructure adapters, wrapping external calls. Domain logic should not know that retries exist.
34. **ALWAYS plan graceful degradation** for external dependencies that may be unavailable. What happens when payment processor is down? When Redis is unreachable? When the metric exporter fails? Design the fallback path upfront.

### Testing strategy (first-class)

34a. **ALWAYS use test-driven development (TDD) as the default workflow.** Every new feature, every bug fix, every behavioral change starts with a failing test. The cycle is Red → Green → Refactor: write a failing test that expresses the desired behavior, write the minimum code to pass, then refactor with the test as a safety net. For bug fixes, the reproduction IS the regression test — commit the failing test in the same PR as the fix. TDD is not optional practice; it is how a well-designed Rust codebase is built. Planning a feature means planning the tests that will drive it into existence FIRST, then the trait/type shape the tests will need, then the production code. Exceptions are narrow: throwaway exploratory spikes (marked as such and followed by a real TDD pass for the keeper version), generated code, and pure glue at the composition root where behavior is already covered by downstream unit tests.

35. **ALWAYS design for testability BEFORE writing the first line of production code.** If you can't test the business rule without a database, introduce a trait at the boundary. If you can't mock the HTTP client, the dependency isn't inverted. Testability is a planning concern, not an afterthought.
36. **ALWAYS sketch the test pyramid** for a project: how many unit tests (inside each crate, fast, no I/O), integration tests (across module boundaries, possibly real dependencies behind Docker), E2E tests (real HTTP server, real DB). Ratio should be many unit → some integration → few E2E.
37. **ALWAYS consider property-based testing for pure functions with invariants** — parsers, serializers, state machines, arithmetic on newtypes. proptest or quickcheck. The cost is modest; the bug-finding value is high.
38. **ALWAYS plan fuzzing scope** for parsers, deserializers, protocol implementations, and anything processing untrusted input. cargo-fuzz with libFuzzer for most cases; afl for more thorough campaigns. Fuzzing is a design choice at project start, not a retrofit.
39. **ALWAYS plan snapshot-test usage** for anything with complex stable output: CLI output, error messages, serialized structures. `insta` is the standard. Decide yes/no at planning time; retrofitting snapshots is easy but deciding test style up front avoids churn.
40. **PREFER trait-first design for units that cross system boundaries** — even when there's only one implementation today. The trait is the test surface; the implementation is the production adapter.

### Growth and refactoring

41. **ALWAYS prefer additive growth.** Stage 1 (single crate) → Stage 2 (lib + bin with modules for layers) → Stage 3 (Cargo workspace with crate-level layering). Never restructure fundamentals between stages — add, don't rewrite.
42. **NEVER split into microservices for "loose coupling" or "fault isolation."** Rust crates already give compile-time loose coupling. Process-level isolation is a separate concern. Split only for: different languages, compliance isolation, wildly different scaling needs, genuinely separate teams/release cycles.

### Delegation

43. **ALWAYS hand off to `rust-implementing` for the actual code.** This skill decides *what to build*; the implementing skill covers *how to type it idiomatically* (which construct, which derive, how to write the `?` chain, how to derive `IntoResponse`).
44. **ALWAYS hand off to `rust-reviewing` for critique.** This skill designs the architecture; the reviewing skill audits whether existing code follows the plan, finds bugs, and measures performance.

---

## 2. The Planning Workflow

Walk through this sequence before starting any Rust project or significant feature. Answer each question; defer to the named section for detail.

### 2.1 Opening questions for a new project

| Question | Defer to |
|---|---|
| What IS the domain? What are the business concepts (bounded contexts)? | §6 Domain Boundaries |
| What are the inputs (interfaces) and outputs (side effects / external systems)? | §4 Principles (Hexagonal), `architecture-patterns.md` |
| Is this a library, a binary, or both? Will it be published to crates.io? | §5 Project Layout |
| Single crate, lib+bin, or Cargo workspace? | §5 Project Layout |
| What's the async story? Any async at all? Which runtime? | §10 Async Strategy + `async-strategy.md` |
| What state needs to survive a crash? What's volatile? | §7.5 Persistence; `data-strategy.md` |
| What external services are involved? What happens when they fail? | §11 Resilience + `services-architecture.md` |
| Which data store(s)? Any caching? | `data-strategy.md` |
| What are the failure modes the system must tolerate gracefully? | §11 Graceful Degradation |
| What's the testing strategy before writing the first line? | §12 Test Strategy + `test-strategy.md` |
| Any unsafe? Any FFI? Any non-Rust interop? | §9 Unsafe Strategy + `unsafe-strategy.md` |
| What's the MSRV commitment? Edition 2021 or 2024? | §5.6 Edition & MSRV |

### 2.2 Opening questions for a new feature in an existing project

| Question | Defer to |
|---|---|
| Does this feature belong in an existing crate/module, or does it warrant a new one? | §6.2 When to create a new crate |
| Which module/crate OWNS the data this feature operates on? | §7 Data Ownership |
| Does the feature cross crate/module boundaries? If yes, how? | §8 Inter-Component Communication |
| Does it need async? Does it need a new task? | §10 Async Strategy |
| Is there a retry / failure path? Is the operation idempotent? | §11.2 Idempotency |
| Does the feature need to degrade gracefully when a dependency is down? | §11.4 Graceful Degradation |
| What changes at the public trait surface? Any `#[non_exhaustive]` concerns? | `error-strategy.md`, `architecture-patterns.md` |

### 2.3 Opening questions when refactoring

| Question | Defer to |
|---|---|
| Which growth stage is this app at? | §13 Growing Architecture |
| Are there crates/modules doing more than one job (mixed responsibilities)? | §6.2 When to split |
| Are there domain modules importing infra crates? | §1 Rule 1 (Dependency direction) |
| Are there `sqlx::Error` leaking through application code? | §7 Error Strategy (layer translation) |
| Are there `Arc<Mutex<T>>` where a channel would be cleaner? | §8 Inter-Component Communication |
| Is there global mutable state (`lazy_static!`, `LazyLock<Mutex<T>>`) instead of injection? | §1 Rule 14 |
| Are there panics (`unwrap`, `expect`) in production paths? | §7 Error Strategy |
| Is the supervision/shutdown tree flat or structured? | §10.4 Graceful Shutdown |

### 2.4 The "what's needed now vs later" test

Rust architecture is **additive**. The progression is:

```
Stage 0 (script):     Single .rs file (cargo-script or trivial main.rs)
Stage 1 (small app):  Single crate + modules (domain logic in plain Rust, traits only at external boundaries)
Stage 2 (medium app): src/lib.rs + src/main.rs, module-level layering (domain/app/infra/api), traits at every external boundary
Stage 3 (large app):  Cargo workspace, crate-level layering, [workspace.dependencies], feature-gated optional adapters
Stage 4 (distributed): Multiple workspaces or separate repos, service contracts (gRPC/tonic, HTTP, message bus)
```

**Never adopt a stage before its triggering problem appears.** Each stage adds complexity; unjustified complexity compounds. Triggers:

- **Stage 1 → Stage 2**: need a second binary (CLI + server), or tests require mocking an external dependency, or module file > 800 lines.
- **Stage 2 → Stage 3**: compile times > 30s incremental, multiple teams owning distinct subsystems, need feature flags to ship different binaries, need to publish a crate independently.
- **Stage 3 → Stage 4**: different languages required, compliance isolation, genuinely different scaling profiles per subsystem, separate deployment lifecycle.

### 2.5 Plan document hygiene

The plan (typically `PLAN.md` or `ARCHITECTURE.md`) is a working artifact during design. Once implementation begins, everything in the plan stays in the plan — never port citations, section numbers, or TDD step numbers into source comments.

- **Section numbers renumber.** A `//! TDD'd by PLAN.md §8 tests #5-7` comment rots the moment §8 becomes §9.
- **Skill citations leak scaffolding.** `//! Per rust-planning §16 BAD/GOOD #2` confuses readers who don't have the skill loaded, and revises when the skill revises.
- **Process notes are not invariants.** "TDD'd by ..." describes how the code was written, not what it does. Invariants belong in doc comments; process belongs in the commit message or the plan.

Rule of thumb: `grep -rn "PLAN\.md\|TDD'd\|rust-planning §\|rust-implementing §" src/` in a finished project should return zero hits. The plan document is the place for the citations; keep the source free of them. (See rust-reviewing §7b #17 and rust-implementing's BAD/GOOD for "Planning-artifact citations in source comments".)

---

## 3. Master "Planning Decision" Table

This is the spine of the skill. Every major architectural question maps to a row. Find your question in the left column; the right columns show the decision and the defer-to section.

### 3.1 Project layout

| Question | Answer | Details |
|---|---|---|
| New project, one person, one binary | Single crate, modules for layering as it grows | §5.1 |
| New project, small team, one deployable | Single crate with `src/lib.rs` + `src/main.rs`, module-level layering | §5.2 |
| Need a second binary (CLI + server) | Single crate, add `src/bin/cli.rs` | §5.2 |
| Publishing a reusable library to crates.io | Single crate; no supervision tree; behaviour-based extension points via traits | §5.3 (Library vs App) |
| Multiple teams with hard boundaries | Cargo workspace with crate-level layering | §5.4 (Workspace) |
| Need different Cargo features per deployable | Workspace + feature-gated binaries | `workspace-layout.md` |
| "Should I split this monolith?" | **Almost certainly no.** Add crates inside a workspace first. | §13 (Growing Architecture) |
| Need code in multiple languages (Rust NIFs, Python FFI) | Rust stays one crate, use FFI (`rustler`, `pyo3`, `cxx`) | `unsafe-strategy.md`; also `rust-nif` skill |
| Feels like it's getting big | Add modules, then crates, then workspace — do NOT split into microservices | §13 |

### 3.2 Crate/module boundaries

| Question | Answer | Details |
|---|---|---|
| Does this feature need a new crate? | Yes if: different dep surface (infra vs domain), multi-team ownership, independent publishing | §6.2 |
| Where does this function live? | In the crate/module that OWNS the primary data being manipulated | §7.1 |
| How big is too big for one crate? | Multiple unrelated aggregates = too big. Unrelated infra adapters (db + http client + kafka) = split | §6.3 |
| Two crates need the same type? | Put the type in the domain crate; both depend on domain | §6.4 |
| Cross-crate data sharing? | Via owning crate's public API, never by reaching into internal modules | §7.1 |
| Integrating with an external system / legacy | Anti-corruption layer at the adapter | §6.5 + `domain-patterns.md` |

### 3.3 Dependency direction and DI

| Question | Answer | Details |
|---|---|---|
| Where does the trait live? | In the crate/module that USES the behavior (domain/app) — not where it's implemented | §4 Rule 13 |
| `Box<dyn Trait>` or `impl Trait` / generics? | Generics if one impl per target. `dyn` for heterogeneous collections, plugin architectures. | §4 Rule 15 |
| Manual DI or container? | Manual until > 20 services, then evaluate shaku / similar | §4 Rule 16 |
| Global state or injection? | Injection via constructor. `LazyLock` only for immutable config. Never `lazy_static!` for mutable services. | §4 Rule 14 |
| Cross-cutting concerns (logging, metrics, auth) | Tower Layers for axum / service composition. Middleware, not mixed into handlers. | `architecture-patterns.md` (Tower) |

### 3.4 Error strategy

| Question | Answer | Details |
|---|---|---|
| Library error type | `thiserror` derive; or hand-rolled `impl Display + impl Error` with `Error { kind: ErrorKind }` for top-tier libs | `error-strategy.md` |
| Application error type (main-level) | `anyhow::Error` with `.context()` chaining | `error-strategy.md` |
| User-facing diagnostics (compiler-style) | `miette` | `error-strategy.md` |
| Multi-layer translation | `From` conversions at each boundary; domain never sees `sqlx::Error` | §7 + `error-strategy.md` |
| Error variants | Specific variants, not catch-all `String`. `#[non_exhaustive]` on public error enums. | §1 Rule 20 |
| Expected business failure | `Result<T, DomainError>` with a typed variant | `error-strategy.md` |
| Unexpected bug (invariant violation) | Panic / `unreachable!()` — the type system says this can't happen | `error-strategy.md` |

### 3.5 Async strategy

| Question | Answer | Details |
|---|---|---|
| Should this be async at all? | Async if: concurrent I/O at scale, web server, streaming, many tasks. Sync if: CLI, library with sync consumers, computational | §10.1 |
| Which runtime? | Tokio (default for web/services). `smol` for minimal/embedded. `async-std` discontinued (March 2025) — migrate off | §10.2 + `async-strategy.md` |
| Single-threaded or multi-threaded runtime? | Multi-threaded (default) for web servers. Single-threaded (`current_thread`) for low-overhead services, deterministic testing, resource-constrained environments | §10.2 |
| How many tokio tasks? | As few as possible — often one per top-level concern (server, background worker, metrics flusher). Avoid per-request tasks unless needed. | §10.3 |
| Supervise tasks? | `JoinSet` for a group; store `JoinHandle` in state; never fire-and-forget | §10.3 |
| Communication between tasks | See §8 (Inter-Component Communication) decision table | §8 |

### 3.6 Inter-component communication (see §8 for full treatment)

| Need | Mechanism | When |
|---|---|---|
| One component calls another's public API | Direct function call | **Default.** No channel overhead |
| Shared immutable config / pool | `Arc<T>` | Connection pools, Tokio `Client`, config snapshot |
| Shared mutable state, infrequent writes | `Arc<RwLock<T>>` or `dashmap` | Caches, registries |
| One-to-one async message passing | `tokio::sync::mpsc` | Producer-consumer, work queue |
| One-to-many broadcast | `tokio::sync::broadcast` | Event notification, pub/sub within process |
| Latest-value watch | `tokio::sync::watch` | Config reload, status, health state |
| One-shot reply | `tokio::sync::oneshot` | Request-response to a task |
| CPU-bound parallelism | `rayon::par_iter()` | Data parallelism, embarrassingly parallel |
| Cross-service | HTTP or gRPC (tonic) | Separate deployments, different languages |
| Events must survive restart | Database-backed queue / Kafka / NATS | Durable messaging |

### 3.7 Data strategy (see `data-strategy.md` for full treatment)

| Question | Answer | Details |
|---|---|---|
| Which store? | PostgreSQL for relational; SQLite for embedded; sled/redb for pure-Rust embedded; Redis for cache/queue; Kafka/NATS for durable messaging | `data-strategy.md` |
| SQL access style | `sqlx` for compile-checked SQL + async. `diesel` for type-safe ORM. Raw `tokio-postgres` for max control. | `data-strategy.md` |
| Migrations | `sqlx-cli` or `refinery` — run from a dedicated binary, not from app startup in production | `data-strategy.md` |
| Connection pool | `sqlx::PgPool` / `deadpool-postgres` — one per process, passed via `Arc` | `data-strategy.md` |
| Caching | In-memory: `moka` (async, TinyLFU). Distributed: Redis via `deadpool-redis` / `fred` | `data-strategy.md` |
| Data ownership | One crate/module owns each entity; others read via that crate's public API | §7.1 |

### 3.8 Testing strategy (see `test-strategy.md` for full treatment)

| Question | Answer | Details |
|---|---|---|
| Test framework | Built-in `cargo test`. `cargo-nextest` for faster parallel execution | `test-strategy.md` |
| Mocking | `mockall` for trait-based mocks (most common). Manual mock structs for simple cases. | `test-strategy.md` |
| Property-based | `proptest` — for parsers, serializers, state machines, arithmetic | `test-strategy.md` |
| Fuzzing | `cargo-fuzz` (libFuzzer) — for parsers, deserializers, protocol handling, untrusted input | `test-strategy.md` |
| Snapshot | `insta` — for CLI output, error messages, serialized structures | `test-strategy.md` |
| E2E HTTP | `reqwest` + test server (`axum-test` crate's `TestServer`, `actix-web::test::TestServer`), or `assert_cmd` for CLI | `test-strategy.md` |
| Concurrency | `loom` — model-checking for lock-free or unsafe concurrency | `test-strategy.md` |
| Compile-fail | `trybuild` — verify that certain patterns fail to compile (type safety) | `test-strategy.md` |
| Coverage | `cargo-llvm-cov` or `cargo-tarpaulin` — set target; don't chase 100% | `test-strategy.md` |

### 3.9 Unsafe strategy (see `unsafe-strategy.md` for full treatment)

| Question | Answer | Details |
|---|---|---|
| Is unsafe justified? | FFI: yes. Performance with measurement: yes. Borrow-checker frustration: NO. | §9 |
| How much unsafe? | Minimal. Isolate in one module; public API is safe. | §9 |
| Safety documentation | Every `unsafe` block has `// SAFETY:` explaining why invariants hold | §9 |
| FFI tooling | `bindgen` (C → Rust), `cbindgen` (Rust → C), `cxx` (C++ interop) | `unsafe-strategy.md` |
| Panics across FFI | `catch_unwind` at the Rust-to-C boundary; unwinding into C is UB | §9 |
| CI hardening | Miri (UB detection), AddressSanitizer for C interop | `unsafe-strategy.md` |

### 3.10 Resilience

| Question | Answer | Details |
|---|---|---|
| Any external dependency that can fail? | Yes → timeout + retry-with-backoff + circuit breaker in the adapter | §11 |
| Where do timeouts live? | Every external boundary. Cascade outer > middle > inner | §11.1 |
| Where do retries live? | Infrastructure adapter, never domain | §11.2 |
| Which operations need idempotency? | Every retryable operation (webhooks, queue consumers, distributed calls, background jobs) | §11.2 |
| Graceful shutdown? | Designed upfront — CancellationToken tree, drain, close connections, exit | §10.4 + §11.5 |
| Health endpoints? | `/health` (liveness) + `/ready` (readiness). Readiness checks downstream deps with fast timeouts | §11.3 |
| Fallback behavior | Planned per dependency: cached-last-good, degraded-mode, user-visible error | §11.4 |

---

## 4. Architectural Principles (the "why" behind the rules)

Ten foundational principles. When rules conflict or requirements are ambiguous, refer back here. These hold at every scale (Stage 1 → Stage 4).

1. **Dependencies point inward.** Infrastructure depends on Application. Application depends on Domain. Domain depends on nothing external. A domain module must NEVER import `sqlx`, `axum`, `reqwest`, `redis`, or any infrastructure crate. The `Cargo.toml` of the domain crate is the proof — if it lists framework crates, the architecture is broken.

2. **Traits are ports. Implementations are adapters.** Every external dependency (database, API, email, file system, message queue) is behind a `trait` defined by the domain. Infrastructure implements the trait. Config or the composition root selects which implementation runs. This IS hexagonal architecture — Rust's trait system has it built in.

3. **The ownership system IS the architecture boundary.** `pub(crate)` enforces aggregate roots — inner entities are invisible outside the crate. Moving an entity into an aggregate transfers ownership — no accidental sharing. Rust's type system encodes architectural decisions that other languages leave to convention.

4. **Cargo.toml encodes layer direction.** `domain/Cargo.toml` has zero infrastructure deps. `infra/Cargo.toml` depends on `domain`. If you need to add `sqlx` to your domain crate, your architecture has a boundary problem. Dependency direction is auditable from `Cargo.toml` alone.

5. **Feature flags are compile-time architecture decisions.** Cargo features let you swap adapters, enable optional subsystems, and gate infrastructure at compile time. Feature-gated code is dead-code-eliminated — zero runtime cost for disabled features. See `workspace-layout.md` for feature-flag architecture patterns.

6. **Start without frameworks, add them at the edges.** Domain logic is plain Rust — no `#[derive(FromRow)]`, no `#[actix_web::get]`, no framework annotations. Framework-specific code lives in the outermost layer only. The litmus test: can you delete the `infrastructure/` and `api/` crates and still compile `domain/` and `application/`?

7. **Design for replaceability.** Can you swap a component's implementation without changing business logic? If not, introduce a trait at the boundary. Can you test a business rule without a database, HTTP server, or external service? If not, your architecture has a boundary problem.

8. **Errors translate at layer boundaries.** Each layer has its own error type. Domain errors are business-meaningful (`OrderNotModifiable`, `InsufficientFunds`). Infrastructure errors are technical (`ConnectionTimeout`, `RowNotFound`). `From` conversions translate between them at layer boundaries. Never surface infrastructure errors to callers.

9. **The composition root wires everything.** `main()` (or a builder in `main()`) creates concrete implementations, injects them into use cases, and starts the server. This is the only place that knows about all concrete types. No service discovers its own dependencies at runtime.

10. **Keep traits small and focused.** No client should depend on methods it doesn't use. If a function only needs `find()`, don't force it to depend on a trait that also defines `save()`, `delete()`, and `export_csv()`. Split into focused traits (`Find<T>`, `Save<T>`). Compose with trait bounds: `impl Find<Order> + Save<Order>`.

11. **Single source of truth (SSOT).** Every fact about the system lives in exactly one authoritative place. A trait's signature is defined once (in the crate that uses it). A table's schema is owned by one module. An error variant set is declared once. Dependency versions are pinned once via `[workspace.dependencies]`. Magic values are `const` / `static`, not literals repeated across files. Conversions between types use `From`/`Into`/`TryFrom`, defined once per direction — not rewritten at each call site.

    The litmus test: *"if this fact changes, how many files do I have to update?"* The answer should be 1 — or 1-plus-compiler-enforced-callers. If updating a fact means grep-and-replace, the fact isn't single-sourced. Rust makes SSOT easier than most languages: generics state an algorithm once for many types; traits state an interface once; `derive` macros derive `Debug`/`Clone`/`Serialize` from the struct; `build.rs` generates types from a spec file; workspace inheritance shares versions and lint configuration. Reach for these before duplicating.

    SSOT is what most "DRY" guidance is really aiming at — but framed around ownership of a fact rather than mechanical deduplication of code. Two different `From` impls that happen to look similar aren't a violation; two places computing the same tax rate with independent `const` values *are*.

---

## 5. Project Layout Decisions

### 5.1 Stage 1 — Small App (single crate, modules)

A single crate with well-organized modules. No workspace, no traits for internal boundaries.

```
my-app/
├── Cargo.toml          (single [package], edition = "2024")
└── src/
    ├── main.rs          (composition: config → state → router → serve)
    ├── config.rs        (figment/envy config loading)
    ├── models.rs        (domain structs, newtypes, validation)
    ├── handlers.rs      (HTTP handlers — thin, delegate to models/db)
    ├── db.rs            (database queries, connection pool setup)
    └── errors.rs        (unified error type with IntoResponse)
```

**What matters at this stage:**
- Domain logic in pure functions on structs (no framework annotations on domain types)
- One error type with `From` conversions
- Handlers delegate to domain functions — no business logic in route handlers
- Config loaded once in `main()`, passed as `State`
- No traits for internal boundaries — direct function calls are fine
- No workspace overhead

**When to grow:** When `models.rs` exceeds ~500 lines, or you need to share types with a second binary (CLI, worker), or tests require mocking an external dependency.

### 5.2 Stage 2 — Medium App (lib + bin, module layering)

Split into library + binary(ies). Introduce traits for external dependencies.

```
my-app/
├── Cargo.toml          (single [package] with [[bin]] + [lib])
├── src/
│   ├── main.rs          (composition root — wires everything)
│   ├── lib.rs           (re-exports domain, app, infra modules)
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── order.rs     (entities, value objects, domain errors)
│   │   ├── customer.rs
│   │   └── ports.rs     (trait OrderRepository, trait PaymentGateway)
│   ├── app/
│   │   ├── mod.rs
│   │   └── use_cases.rs (PlaceOrderUseCase, orchestration logic)
│   ├── infra/
│   │   ├── mod.rs
│   │   ├── postgres.rs  (PgOrderRepository implements OrderRepository)
│   │   └── stripe.rs    (StripeGateway implements PaymentGateway)
│   └── api/
│       ├── mod.rs
│       ├── routes.rs    (axum Router setup)
│       └── handlers.rs  (extract → use case → respond)
```

**What changes from Stage 1:**
- Traits appear for external dependencies — enables mocking in tests
- Domain module has zero I/O imports
- Error types split: `DomainError`, `InfraError`, `ApiError` with `From` conversions
- Use case structs take trait-bounded dependencies via constructor injection
- `main()` is the composition root — only place that knows concrete types
- Still one crate — modules provide boundaries, `pub(crate)` hides internals

**When to grow:** When compile times exceed 30s, when multiple teams work on distinct subsystems, when you need different Cargo features for different deployment targets, or when a library crate should be published independently.

### 5.3 Library vs Application

Libraries and applications have different architectural constraints:

| Concern | Library | Application |
|---|---|---|
| Dependencies | Minimize; prefer features | Pin aggressively via lockfile |
| Global state | **NEVER** (breaks composability) | OK in composition root |
| Error type | `thiserror` enum or hand-rolled `impl Error` | `anyhow::Error` acceptable at main level |
| Logging | `tracing` facade only, no subscribers | Initializes subscribers |
| Runtime | Don't force one (generic over `Runtime`, or be sync) | Picks one (Tokio usually) |
| Panics | Very rare — prefer `Result` | Acceptable for impossible-state checks |
| `Cargo.toml` features | Expose optional functionality via features | Consume features |
| MSRV | Stated clearly, tested in CI | Choose based on dep constraints |

**Library-crate checklist:** no panics in public API, no `unwrap`/`expect` outside initialization, every public function has `# Errors` section when it returns `Result`, no runtime lock-in, no global state.

### 5.4 Stage 3 — Large App (Cargo workspace)

Full Cargo workspace with crate-level boundaries.

```
my-app/
├── Cargo.toml           (workspace, [workspace.dependencies], [workspace.lints])
├── crates/
│   ├── domain/          (zero infra deps — entities, traits, errors)
│   │   ├── Cargo.toml   (only: uuid, chrono, thiserror, serde)
│   │   └── src/
│   ├── app/             (use cases — depends only on domain)
│   │   ├── Cargo.toml   (only: domain, tracing)
│   │   └── src/
│   ├── infra/           (adapters — depends on domain, app, sqlx, redis, etc.)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── api/             (HTTP layer — axum, routing, auth middleware)
│   │   ├── Cargo.toml
│   │   └── src/
│   └── cli/             (CLI binary — clap, different entry point)
│       ├── Cargo.toml
│       └── src/
```

**What changes from Stage 2:**
- Crate boundaries enforce dependency direction at `Cargo.toml` level — impossible to accidentally import `sqlx` in domain
- `[workspace.dependencies]` ensures version consistency across crates
- Each crate compiles independently — better incremental builds
- Feature flags gate optional adapters (`redis-cache`, `metrics`)
- CI can test crates independently (`cargo test -p domain`)
- Nested supervision: domain crate might have sub-crates for distinct bounded contexts

See `workspace-layout.md` for the full treatment of `[workspace.dependencies]`, `[workspace.lints]`, feature architecture, and member Cargo.toml patterns.

### 5.5 What DOESN'T Change Between Stages

These principles hold at every scale:
- Domain logic lives in pure functions on structs — this never changes
- External dependencies are behind traits — this never changes (Stage 1 uses direct calls, but refactors to traits when testing demands it)
- `Result<T, E>` for fallible operations, `?` for propagation — this never changes
- Composition happens in `main()` — this never changes
- Error types translate at boundaries — this never changes

**The progression is additive.** Add modules, then traits, then crates as needed. Never restructure the fundamentals.

### 5.6 Edition and MSRV

- **Edition 2024** (stabilized in Rust 1.85.0) is the default for new projects — RPIT lifetime capture improvements, `unsafe extern` blocks, `gen` keyword reservation. Previous editions remain fully supported; picking 2024 just unlocks the newest ergonomics.
- **MSRV (Minimum Supported Rust Version)** is a commitment. For libraries: state in README and CI. For applications: tie to the oldest rustc you support in your deployment environment. `rust-version = "1.85"` in `Cargo.toml`. **Data from validation:** Zed (end-user app) does NOT declare MSRV; nushell (end-user app) DOES, at `1.93.1`. Declaring it in an app buys build-reproducibility and a shield against accidental dep bumps that silently raise your toolchain floor — but is optional. Declaring it in a library is expected.
- **Async closures** (stabilized in 1.85.0) are edition-independent — work on all editions.

### 5.7 Rust as Embedded Library (NIF/FFI Architecture)

When Rust is used as an embedded library (Rustler NIFs, PyO3, C FFI), the standard web architecture doesn't apply:

- **No HTTP layer** — the host language handles networking
- **No domain layer** inside the Rust crate — domain logic may live in the host language, with Rust owning only the computationally-expensive or correctness-critical pieces
- **Composition root is `rustler::init!`** (or `#[pymodule]`, `extern "C"` exports)
- **State lives in `OnceLock<T>` or `ResourceArc<T>`**, not in application state
- **Threading is controlled by the host** — dirty schedulers (BEAM), GIL release (Python)

**Architecture for NIF crates:**

```
src/
├── lib.rs          # rustler::init!, NIF function thin wrappers
├── types.rs        # NifStruct/NifMap/NifTaggedEnum definitions
├── runtime.rs      # OnceLock<Runtime>, init/shutdown
├── commands.rs     # Command enum, command handler
└── core/           # Pure Rust logic (testable without NIF)
    ├── mod.rs
    └── ...
```

**Key principle:** NIF functions are thin wrappers. Keep business logic in `core/` — testable with `cargo test`, no Rustler dependency. See the `rust-nif` skill for the Rust-side NIF patterns and the `elixir-planning` skill for the Elixir-side integration.

---

## 6. Domain Boundaries

See `domain-patterns.md` for the full DDD treatment (aggregates, value objects, bounded contexts, event sourcing, CQRS). The distilled planning decisions:

### 6.1 Do I need a new crate?

A new crate is warranted when ANY of:
- Different dependency surface (domain has zero I/O deps; infra has sqlx + axum + redis + ...). Keeping them in the same crate forces the smaller surface to pay for the larger.
- Multi-team ownership — each crate has clear owners, reducing merge conflicts.
- Independent publishing — this crate will be published to crates.io or an internal registry separately.
- Independent compilation — compile times warrant the split, and the crates are genuinely independent.
- Plugin architecture — core + multiple optional adapters where each adapter is its own crate.

A new crate is NOT warranted for:
- "Feels big" — use modules instead.
- "Separation of concerns" inside a single team's code — use modules.
- Wanting visibility boundaries — `pub(crate)` gives you that in a single crate.

### 6.2 When to create a new context (bounded context)

A bounded context is a domain-language boundary: within one context, terms mean specific things (in `Ordering`, "Customer" means "the person placing this order" — in `Billing`, "Customer" may mean "the legal entity receiving invoices"). When you have genuinely different languages inside your system, each gets its own crate (in Stage 3+) or module group (in Stage 2).

Signals you need a new context:
- Same word used for different things
- Two feature areas have essentially no shared vocabulary
- Different team owns the area
- Different deployment/release cycle

See `domain-patterns.md` §Bounded Contexts for the full pattern.

### 6.3 Aggregate boundaries

An **aggregate** is the consistency boundary — the unit of change that must be atomic. One aggregate = one transaction.

Rules:
- Each aggregate has one **root** entity; external code can only hold references to the root.
- Modifications to an aggregate go through the root's methods.
- Cross-aggregate operations are sagas or eventual consistency, never multi-aggregate `BEGIN TRANSACTION ... COMMIT` across unrelated roots.
- Ownership (`pub(crate)` on inner entities) enforces this at the type level.

### 6.4 Shared types

When two contexts need the same concept:
- If it's a **value object** (immutable, no identity): put in a shared `kernel` or `shared` crate, both depend on it.
- If it's an **entity** (has identity, mutable): it belongs to ONE context. The other context references it by ID (`OrderId`) or by a context-local representation.

### 6.5 Anti-corruption layer

When integrating with a legacy or external system, never let its data model leak into your domain. Translate at the adapter:

- External system speaks in `LegacyCustomerRecord` → adapter converts to domain `Customer`.
- Domain speaks in `Order` → adapter converts to external `PurchaseOrderV2`.
- Domain never imports the external API's types.

---

## 7. Data Strategy

See `data-strategy.md` for the full treatment. Planning-level decisions:

### 7.1 Data ownership

- **One crate/module owns each entity's persistence.** The `Orders` context owns `orders` table. Nobody else writes to it.
- **Reads by other contexts go through the owner's public API** — `OrdersQueries::get_by_id(id)` — not via direct SQL.
- **This is a planning decision**, not an implementation one. If `Billing` is reading from `orders` directly, move the read into `Orders`.

### 7.2 Store choice

| Store | When |
|---|---|
| PostgreSQL | Default for relational. Strong consistency, mature tooling, powerful query language |
| SQLite | Embedded, single-writer, edge/CLI/desktop apps |
| MySQL | Legacy or team preference; PostgreSQL is usually better |
| Redis | Cache, job queue, pub/sub within a single cluster |
| MongoDB | Document-shaped data with rich queries; consider Postgres JSONB first |
| sled / redb | Pure-Rust embedded key-value, no FFI; good for CLI state, simple caches |
| Kafka / NATS | Durable messaging, event streaming, cross-service |
| DynamoDB / Bigtable / Spanner | Cloud-scale needs, vendor commitment |

### 7.3 Access pattern

| Need | Choice |
|---|---|
| Compile-checked SQL + async | `sqlx` (most common for new code) |
| Type-safe ORM with migrations | `diesel` (sync by default; async via `diesel_async`) |
| Raw async PG client | `tokio-postgres` |
| Query builder on top of raw | `sea-query`, `refinery` for migrations |
| Embedded KV | `sled`, `redb` |

### 7.4 Migration strategy

- **Migrations run from a dedicated binary or CLI**, not from app startup in production. App startup should error if migrations are out of date, not silently migrate.
- **Migrations are forward-only** in production. Create a new migration to undo a bad one.
- **Compatibility between migration steps and deployments**: a rolling deploy means old and new code briefly coexist — migrations must be compatible with both.

### 7.5 Persistence vs in-memory

| State | Where |
|---|---|
| Must survive restart | Database |
| Can be re-derived from DB | In-memory, cache |
| Cross-process consistency | Database or distributed cache |
| Single-process, small | `Arc<RwLock<T>>` or `dashmap` |
| Hot, read-heavy | `moka` async cache in front of DB |

### 7.6 Caching strategy

See `data-strategy.md` §Caching for the full treatment. Planning rule: **cache invalidation is a design decision, not an afterthought.** Pick one:
- TTL-based (simplest, tolerates staleness)
- Write-through (write to DB, then update cache)
- Write-behind (write to cache, async flush to DB — only if loss is acceptable)
- Invalidation by event (PubSub or DB triggers)

---

## 8. Inter-Component Communication

How components talk to each other is a critical architectural decision in Rust. Unlike Elixir where processes and message passing are built-in, Rust requires explicit choices about communication mechanisms.

### 8.1 Decision Guide

| Need | Mechanism | When to Use |
|------|-----------|-------------|
| Simple sync call | Direct function call | **Default** — one component calls another's public API |
| Shared data across async tasks | `Arc<T>` with interior mutability | Connection pools, config, shared caches |
| One-to-one async message passing | `tokio::sync::mpsc` | Producer-consumer, work queues, log shipping |
| One-to-many broadcast | `tokio::sync::broadcast` | Event notification, price feeds, state changes |
| Latest-value watch | `tokio::sync::watch` | Config reload, status updates, health state |
| One-shot response | `tokio::sync::oneshot` | Request-response within async tasks |
| CPU-bound parallel work | `rayon::par_iter()` | Data parallelism, batch processing |
| Cross-service communication | HTTP/gRPC (reqwest/tonic) | Separate deployments, different languages |
| Persistent async jobs | Database-backed queue (custom or crate) | Must survive restarts, need retries |

### 8.2 Escalation Path

Start with the simplest mechanism. Escalate only when you have the specific problem the next level solves.

```
1. Direct function calls (default — no channel overhead)
   │
   ├── Need async decoupling? → tokio::sync::mpsc (bounded channel)
   │
   ├── Need multiple listeners? → tokio::sync::broadcast
   │
   ├── Need latest value only? → tokio::sync::watch
   │
   ├── Need backpressure? → Bounded mpsc (blocks sender when full)
   │
   ├── Events must survive restarts? → Database-backed queue
   │
   └── Cross-service? → HTTP/gRPC with retry + circuit breaker
```

### 8.3 Shared State vs Channels

| Use shared state (`Arc<Mutex<T>>`) when... | Use channels when... |
|---|---|
| Multiple readers, infrequent writes | Clear producer-consumer relationship |
| Need atomic read-modify-write | Tasks should be decoupled (different lifetimes) |
| State is a single value (counter, cache) | Processing involves I/O or blocking work |
| All accessors are in the same task group | Need backpressure or buffering |

**Shared state anti-patterns to avoid at design time:**
- Holding `MutexGuard` across `.await` — use `tokio::sync::Mutex` or restructure the code so the lock is released before await
- Using `RwLock` for write-heavy workloads — contention defeats the purpose
- Global `lazy_static!` / `LazyLock<Mutex<T>>` for service instances — use `Arc` + injection instead

### 8.4 When NOT to Add Channels

Most Rust applications don't need channels for internal component communication. Direct function calls through trait-bounded dependencies are the right default. Signals that you ACTUALLY need a channel:

- Work can be deferred (logging, metrics, notifications) — a channel lets callers not wait
- Producer is faster than consumer and you need backpressure — bounded mpsc provides this
- Multiple consumers process different aspects of the same events — broadcast channel
- Cross-task decoupling is a genuine requirement (e.g., background worker independent of request handlers)

If none of these apply, use a direct method call. A bounded mpsc channel between two objects that always work together is just an indirection.

---

## 9. Unsafe Strategy

See `unsafe-strategy.md` for full treatment. Planning-level decisions:

### 9.1 Unsafe Budget

Decide up front: **is unsafe allowed in this crate, and for what reasons?**

Acceptable reasons:
- **FFI boundary** — calling into or being called from C/C++. Unavoidable.
- **Hardware access** — embedded, memory-mapped I/O. Wrap in typed safe abstractions.
- **Measured performance** — benchmarks show a safe version is insufficient AND the unsafe version is correct.
- **Interop with a safe abstraction that requires it** — e.g., implementing a trait that has unsafe methods.

Unacceptable reasons:
- "The borrow checker is annoying" — restructure the code.
- "It's faster" without benchmarks — prove it.
- "Other languages do this" — this is Rust; use Rust patterns.

### 9.2 Unsafe Isolation

If unsafe is allowed:
- **Concentrate it in one module** with a safe public API.
- **Every `unsafe` block has `// SAFETY:` comment** explaining why invariants are upheld.
- **Public API is safe** — callers should never see `unsafe fn`. If they must, it's a separate sub-crate or a feature-gated "unsafe API" module.

### 9.3 FFI Architecture

| Direction | Tooling |
|---|---|
| Call C from Rust | `bindgen` (generates Rust bindings from C headers) |
| Call Rust from C | `cbindgen` (generates C headers from Rust) |
| Call C++ from Rust | `cxx` (safe C++ interop) |
| Embed in another runtime | `rustler` (BEAM), `pyo3` (Python), `jni` (Java), `neon` (Node) |

### 9.4 Panic discipline at FFI boundary

**Unwinding into non-Rust code is Undefined Behavior.** Always wrap Rust-side FFI functions with `catch_unwind`:

```rust
#[no_mangle]
pub extern "C" fn do_work(input: *const u8, len: usize) -> i32 {
    let result = std::panic::catch_unwind(|| {
        // ... actual work ...
        0
    });
    result.unwrap_or(-1) // Never panic across the boundary
}
```

For critical unsafe sections inside a rayon closure or similar, use the `AbortIfPanic` guard pattern (see `unsafe-strategy.md`) to ensure the process aborts rather than continuing in a corrupted state.

### 9.5 CI hardening for unsafe crates

- **Miri** — catches UB (out-of-bounds, use-after-free, data races in single-threaded code). Slow but invaluable.
- **AddressSanitizer** — when linking with C code, catches memory errors.
- **Loom** — for lock-free concurrent code, model-check the happens-before relationships.
- **Fuzzing** — for parsers and format handlers, cargo-fuzz catches crashes and UB.

---

## 10. Async Strategy

See `async-strategy.md` for full treatment. Planning-level decisions:

### 10.1 Should this be async at all?

**Prefer sync** for:
- CLI tools (unless doing many concurrent HTTP/disk operations)
- Libraries that don't need to run many concurrent tasks
- Pure computation
- Code that will be called from a sync context (mixing requires `tokio::runtime::Runtime::block_on` which is awkward)

**Prefer async** for:
- Web servers handling many concurrent connections
- Clients doing concurrent I/O (pipelined HTTP, fan-out to many APIs)
- Streaming / pipelining (download while processing)
- Actor-like systems with many long-lived independent tasks
- NIFs/embedded runtimes where the host language is async

### 10.2 Runtime choice

| Runtime | When |
|---|---|
| **Tokio** (default) | Web services, databases, most async code. Huge ecosystem; `axum`, `sqlx`, `reqwest`, `tonic` all integrate. |
| `smol` | Small/embedded, minimal dependencies, want `async-std`-style API with less baggage |
| `async-std` | **Discontinued** as of March 2025 (v1.13.1 final). Do not start new code with it; migrate existing code to Tokio or smol. |
| `monoio` / `glommio` | io_uring-based, single-threaded-per-core, Linux-only high-performance servers |
| `embassy` | Embedded (no_std), bare-metal microcontrollers |

**Rule:** one runtime per binary. Never mix Tokio + async-std in the same process — their reactors don't share. GUI apps with custom runtimes (Zed/GPUI wrapping `async-task` + platform primitives, egui, iced) count as "one runtime" — the UI executor IS the runtime; don't try to spawn Tokio tasks on it.

### 10.3 Task topology

- Sketch the task graph before spawning.
- Top-level tasks: usually 1-5 (HTTP server, background workers, metrics flusher, health checker).
- Per-connection tasks: spawned by the server automatically (axum does this); don't manually spawn per-request unless doing something unusual.
- Supervise tasks: `JoinSet` for groups; store handles in state; never fire-and-forget.

### 10.4 Graceful shutdown

Design upfront. Pattern:

```
1. Shutdown signal received (SIGTERM, Ctrl+C)
2. Root CancellationToken triggered
3. HTTP server stops accepting new connections
4. In-flight requests given a grace period (e.g. 30s) to complete
5. Child tasks receive cancellation, finish current unit of work
6. Connection pools drained
7. Exit with the right code
```

`tokio::select!` between a shutdown signal and the work loop in every long-running task. See `async-strategy.md` for templates.

---

## 11. Resilience Planning

See `services-architecture.md` for full treatment.

### 11.1 Timeouts

- **Every external call has a timeout.** No exceptions — `tokio::time::timeout` wrapping HTTP/DB/RPC calls.
- **Cascade correctly**: outer timeout > middle > inner. If HTTP handler has 30s timeout and DB query has 60s timeout, the DB timeout is meaningless — handler will return 504 first, wasting DB capacity.
- **Timeout values are intentional**, not "some large number." Base them on SLO + margin.

### 11.2 Retries and idempotency

- **Retries only on idempotent operations.** A retried non-idempotent operation (charge the credit card) duplicates effects.
- **Retry with exponential backoff + jitter** to avoid thundering herd.
- **Cap retries** — after N attempts, fail and escalate (circuit break, escalate to operator).
- **Idempotency keys** for APIs that accept retryable writes. Client sends `Idempotency-Key: <uuid>`; server deduplicates.

### 11.3 Circuit breakers

- In the **adapter**, wrapping an external call — not in domain.
- States: closed (calls pass), open (calls fail fast), half-open (trial calls after cooldown).
- Thresholds are SLO-driven: how many failures in what window trigger opening?
- Fallback behavior when open: cached-last-good? Default value? User-visible error?

### 11.4 Graceful degradation

For each external dependency, plan: **what happens if it's unavailable?**

- Payment processor down → queue for later? Reject with retry-after? Switch to backup processor?
- Cache down → fall back to DB (slower)?
- Metrics down → log locally? Drop?
- Analytics down → drop quietly, core flow unaffected?

### 11.5 Health and readiness

- `/health` (liveness) — is the process alive? Returns 200 if the HTTP server is up. Must not check external dependencies (if DB is down, you don't want Kubernetes to kill an otherwise-fine pod).
- `/ready` (readiness) — can this instance serve traffic? Checks downstream deps with fast timeouts. Returns 503 if not ready.
- Kubernetes `readinessProbe` uses `/ready`; `livenessProbe` uses `/health`.

---

## 12. Test Strategy (first-class)

See `test-strategy.md` for the full treatment. This is NOT optional — testability IS an architectural concern.

### 12.1 Plan before writing

Before writing production code, answer:
- What are the test doubles? Which dependencies need trait-based injection for mocking?
- What's the test pyramid shape? Most unit → some integration → few E2E.
- Which pieces need property-based testing? Parsers, serializers, state machines, arithmetic.
- Which pieces need fuzzing? Anything processing untrusted input.
- Which pieces need snapshot testing? CLI output, error messages, serialized formats.

### 12.2 Trait-first design

For every external boundary — and any internal piece you want to unit-test in isolation — define the trait first, then the implementation:

```rust
// 1. Define the trait (this is the test surface)
trait OrderRepository {
    async fn save(&self, order: &Order) -> Result<(), RepoError>;
    async fn find(&self, id: OrderId) -> Result<Option<Order>, RepoError>;
}

// 2. Production implementation
struct PgOrderRepository { pool: PgPool }
impl OrderRepository for PgOrderRepository { /* ... */ }

// 3. Test: mock or fake
#[cfg(test)]
struct InMemoryOrderRepository { orders: Arc<Mutex<HashMap<OrderId, Order>>> }
#[cfg(test)]
impl OrderRepository for InMemoryOrderRepository { /* ... */ }
```

This is the **foundation of testability** in Rust. Every architectural boundary has a trait, and tests inject fake implementations.

### 12.3 TDD as design driver

Test-driven development in Rust:
1. Write the **call site** first — what does the use case want from its dependencies?
2. That call shape becomes the **trait**.
3. Implement a **fake** satisfying the trait.
4. Write the failing test.
5. Implement the real adapter.

The trait is DESIGNED by the caller's need, not dictated by the implementor's convenience. This is Dependency Inversion made concrete.

### 12.4 The test pyramid (planning view)

| Level | Scope | Speed | Count |
|---|---|---|---|
| Unit | One module, no I/O | μs-ms | 1000s |
| Integration | Across modules, possibly real deps (Docker) | ms-s | 100s |
| E2E | Real HTTP server, real DB, real external (or VCR'd) | s | 10s |
| Property | Generative — runs many cases | ms-s | 10s (files), 1000s (cases) |
| Fuzz | Untrusted input, runs for hours in CI | mins-hours | A few campaigns |

**Investment at planning time:** decide which of your crates/modules needs which levels. Domain crate: unit-test-heavy. API crate: integration + E2E. Parsers: fuzz. Serializers: property + snapshot.

### 12.5 Mocking strategy

| Need | Tool |
|---|---|
| Trait-based mocks with expectations | `mockall` |
| Simple in-memory fakes | Hand-written `struct Fake` impls |
| HTTP call mocking | `wiremock` |
| Time control | `tokio::time::pause()` |
| DB transactions rolled back per test | `#[sqlx::test]` or hand-rolled transaction fixtures |
| Compile-fail tests | `trybuild` |
| Loom (concurrency model checking) | `loom` |
| Coverage | `cargo-llvm-cov` |

### 12.6 Anti-patterns at the planning level

- **Testing against concrete types instead of traits** — locks you to one implementation; breaks when infrastructure changes.
- **Heavy mocking at unit level** — if a unit test requires mocking 5 trait objects, the unit is doing too much.
- **E2E-only test suite** — slow, flaky, and doesn't localize failures. You need unit tests.
- **Deciding "we'll add tests later"** — usually means "we'll rewrite to make it testable later." Design for testability from line 1.

---

## 13. Growing Architecture

Revisit §5 for the stage definitions. Here are the **transition triggers**:

| From | To | Trigger |
|---|---|---|
| Script (single `.rs`) | Stage 1 (single crate) | Multiple source files needed; reusable types emerging |
| Stage 1 (single crate, flat) | Stage 1 (single crate, modules) | File > 500 lines; clear subtopics |
| Stage 1 (modules) | Stage 2 (lib + bin) | Second binary needed OR tests need to mock external deps (introduce traits) |
| Stage 2 | Stage 3 (workspace) | Compile time > 30s incremental OR multi-team ownership OR independent publishing OR feature-gated binaries needed |
| Stage 3 | Stage 4 (distributed) | Different languages, compliance isolation, different scaling profiles, separate deployment lifecycle |

**Common mistakes:**
- Jumping to Stage 3 for a solo project — adds complexity without benefit.
- Staying at Stage 1 when tests are untestable — refactor to traits (Stage 2) before adding more features.
- Introducing microservices to avoid refactoring — the underlying boundary problem just moves to the network, with more failure modes.

---

## 14. Refactoring Signals

When reviewing an existing architecture, these signals indicate specific refactorings:

| Signal | What to refactor | How |
|---|---|---|
| `domain/Cargo.toml` lists `sqlx` / `axum` / `reqwest` | Dependency direction violation | Move infra-dependent code out of domain; domain defines traits, infra implements |
| Several `#[cfg(feature = "...")]` in domain logic | Feature gates leaking into domain | Move feature gates to composition root; swap trait implementations |
| `.unwrap()` / `.expect()` scattered in request handlers | Error handling not planned | Define `ApiError` enum, implement `IntoResponse`, use `?` everywhere |
| `Arc<Mutex<T>>` around most state | Over-use of shared mutable state | Introduce channels for producer-consumer; split state so fewer callers share |
| One giant `AppState` struct with ~20 fields | Insufficient modular boundaries | Split into per-subsystem state; inject what each component needs |
| Tests require real DB for unit tests | Missing trait abstraction | Introduce trait at boundary; mock for unit tests, real adapter for integration |
| Compile times > 60s for any change | Single mega-crate | Split into crates along natural domain boundaries |
| `cargo check` fast but `cargo build` slow | Heavy macro / generic use | Move macro-heavy code into separate crate that rebuilds independently |
| Handler functions > 50 lines | Business logic leaking into HTTP layer | Move to use case; handler becomes thin adapter |
| Error types duplicated across crates | Missing shared error module | Centralize error taxonomy in domain / shared crate |
| `#[cfg(test)]` helpers duplicated across modules | Missing test utility module | `tests/common/` or a `test-support` crate |

---

## 15. Anti-Patterns Catalog (planning-level)

Anti-patterns specific to **architectural decisions**. For implementation-level anti-patterns (which construct, which derive), see `rust-implementing`. For review-time anti-pattern detection, see `rust-reviewing`.

### 15.1 Layering violations

- **Domain imports infrastructure.** Move the import to infra; domain defines a trait if needed.
- **HTTP handler contains business logic.** Move to a use case; handler becomes parse → call use case → format response.
- **Repository returns DTOs.** Repository returns domain entities; translation happens in the adapter or use case.
- **Domain entity has framework annotations (`#[derive(FromRow)]`, `#[serde]`).** Put adapter types in infra/DTO layer; domain is plain Rust.

### 15.2 State management

- **Global singletons for services** (`lazy_static!`, `LazyLock<Mutex<T>>` for mutable state). Use `Arc` + injection.
- **Hidden mutable state inside a trait implementation.** Make state explicit — either passed in or owned by the struct, not a global.
- **AppState as a grab-bag** — one giant struct with every client, pool, and service. Split by subsystem.

### 15.3 Async anti-patterns

- **`std::thread::sleep` in an async context.** Use `tokio::time::sleep`.
- **Blocking calls (synchronous DB, sync file I/O) in async handlers.** Use `tokio::task::spawn_blocking` or switch to an async client.
- **Holding a `MutexGuard` across `.await`.** Use `tokio::sync::Mutex`, or restructure to release the lock first.
- **Fire-and-forget `tokio::spawn` for important work.** Track handles in `JoinSet`.
- **Per-request `tokio::spawn` without need.** Let axum handle per-request concurrency.

### 15.4 Dependency injection

- **Service locator / container injection passed everywhere.** Inject specific dependencies, not the container.
- **Hidden dependencies via `LazyLock<Service>`.** Make them explicit parameters.
- **Constructors that take `&Config` for one field.** Pass just the field; document the dependency.

### 15.5 Testing anti-patterns

- **Unit tests require a real DB.** Introduce a trait; mock for unit tests.
- **Mocks return types that don't match production.** Use the same trait for both.
- **E2E-only test suite.** Slow, flaky, can't localize. Add unit + integration levels.
- **`#[cfg(test)]` branches in production code paths.** Dependency injection replaces this cleanly.

### 15.6 Error handling (planning-level)

- **`Box<dyn Error>` in public library APIs.** Define a typed enum.
- **`.unwrap()` / `.expect()` in request handlers.** Proper error mapping to HTTP status.
- **Untyped string errors (`Result<T, String>`).** Typed enum with variants.
- **`anyhow::Error` in library public APIs.** `anyhow` is for applications, not libraries.
- **Panics in initialization that should be errors.** Return `Result` from `new()` / `build()`.

### 15.7 Resilience

- **No timeouts on external calls.** Always wrap with `tokio::time::timeout`.
- **Retries on non-idempotent operations.** Either make idempotent or don't retry.
- **Circuit breakers in domain code.** Move to adapter.
- **No plan for graceful shutdown.** Design upfront, not as an afterthought when SIGTERM causes data loss.

---

## 16. Architectural BAD/GOOD Pairs

Fifteen concrete anti-patterns that Claude commonly produces despite the rules above. Each pair shows the rule firing at decision-time. Deliberately overlaps with §1 Rules and §15 Anti-Patterns Catalog — these are the critical cases that benefit from side-by-side code.

### 1. Dependency direction violation

```toml
# BAD: domain/Cargo.toml — framework leaks into the domain layer
[dependencies]
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }
axum = "0.8"
```

```toml
# GOOD: domain/Cargo.toml — zero infrastructure deps
[dependencies]
uuid = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
# Database/HTTP/queue crates live in infra/Cargo.toml, not here.
```

### 2. Trait defined next to its implementation

```rust
// BAD: infra/postgres.rs defines BOTH the trait AND the impl.
// Now the trait is an implementation detail of Postgres — every
// consumer of the trait pulls in sqlx transitively.
pub trait OrderRepository { /* ... */ }
impl OrderRepository for PgOrderRepository { /* ... */ }
```

```rust
// GOOD: domain/ports.rs owns the trait; infra/postgres.rs implements.
// domain/ports.rs
pub trait OrderRepository {
    async fn save(&self, order: &Order) -> Result<(), RepoError>;
}
// infra/postgres.rs
use domain::ports::OrderRepository;
pub struct PgOrderRepository { pool: PgPool }
impl OrderRepository for PgOrderRepository { /* ... */ }
```

### 3. `Box<dyn Error>` in a published library API

```rust
// BAD: caller can't pattern-match, can't extract typed data, can't
// distinguish recoverable from fatal. Hides all variants behind a trait.
pub fn parse(s: &str) -> Result<Config, Box<dyn std::error::Error>> { /* ... */ }
```

```rust
// GOOD: typed enum with meaningful variants. Caller branches on kind.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum ParseError {
    #[error("line {line}: unexpected token {token:?}")]
    Syntax { line: u32, token: String },
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
pub fn parse(s: &str) -> Result<Config, ParseError> { /* ... */ }
```

### 4. `Result<T, String>` for errors

```rust
// BAD: string errors lose all structure. Caller parses the message
// to decide what to do — fragile, unstable across versions.
pub fn load() -> Result<Config, String> {
    std::fs::read_to_string("config.toml").map_err(|e| e.to_string())
}
```

```rust
// GOOD: typed error preserves cause chain, variant, structured fields.
#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("reading {path}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
}
pub fn load() -> Result<Config, LoadError> { /* ... */ }
```

### 5. Hidden global mutable service state

```rust
// BAD: global mutable singleton. Cannot swap for tests. Cannot have
// two instances per process. Ordering bugs on first use.
use std::sync::{LazyLock, Mutex};
static POOL: LazyLock<Mutex<Option<PgPool>>> = LazyLock::new(|| Mutex::new(None));

pub async fn init(url: &str) {
    *POOL.lock().unwrap() = Some(PgPool::connect(url).await.unwrap());
}
```

```rust
// GOOD: constructor injection. Explicit ownership, trivially testable.
pub struct OrderService { pool: Arc<PgPool> }
impl OrderService {
    pub fn new(pool: Arc<PgPool>) -> Self { Self { pool } }
}
// main.rs wires everything once:
let pool = Arc::new(PgPool::connect(&cfg.db_url).await?);
let orders = OrderService::new(pool.clone());
```

### 6. Feature flags in domain logic

```rust
// BAD: compile-time conditional business logic. Domain changes
// shape per feature set. Tests must enumerate features.
pub fn charge(card: &Card, amount: Money) -> Result<PaymentId, Error> {
    #[cfg(feature = "stripe")]  { stripe::charge(card, amount) }
    #[cfg(feature = "paypal")]  { paypal::charge(card, amount) }
    #[cfg(not(any(feature = "stripe", feature = "paypal")))]
    { panic!("no payment processor configured") }
}
```

```rust
// GOOD: domain defines the trait; feature gates live in the
// composition root (main.rs) and pick the concrete impl.
pub trait PaymentProcessor { /* ... */ }

// main.rs
#[cfg(feature = "stripe")]
let processor: Arc<dyn PaymentProcessor> = Arc::new(StripeProcessor::new());
#[cfg(feature = "paypal")]
let processor: Arc<dyn PaymentProcessor> = Arc::new(PayPalProcessor::new());
```

### 7. Passing the whole `&Config` to a service

```rust
// BAD: EmailService's dependencies are opaque. Reading the struct
// you can't tell what it actually uses. Testing requires a full Config.
pub struct EmailService { config: Arc<Config> }
impl EmailService {
    pub fn new(config: Arc<Config>) -> Self { Self { config } }
    pub fn send(&self, to: &str, msg: &str) -> Result<()> {
        smtp_send(&self.config.smtp_url, &self.config.from_addr, to, msg)
    }
}
```

```rust
// GOOD: inject only what's used. Dependencies are self-documenting.
// Tests construct with just smtp_url + from_addr.
pub struct EmailService { smtp_url: String, from_addr: String }
impl EmailService {
    pub fn new(smtp_url: impl Into<String>, from_addr: impl Into<String>) -> Self {
        Self { smtp_url: smtp_url.into(), from_addr: from_addr.into() }
    }
    pub fn send(&self, to: &str, msg: &str) -> Result<()> {
        smtp_send(&self.smtp_url, &self.from_addr, to, msg)
    }
}
```

### 8. Business logic in an HTTP handler

```rust
// BAD: handler contains pricing + validation + persistence. Untestable
// without axum. Can't be called from a background job or CLI binary.
async fn place_order(State(db): State<PgPool>, Json(req): Json<OrderReq>) -> Response {
    if req.items.is_empty() { return bad_request("no items"); }
    let total: Decimal = req.items.iter().map(|i| i.price * i.qty).sum();
    let tax = total * Decimal::new(7, 2);
    sqlx::query!("INSERT INTO orders ...").execute(&db).await.unwrap();
    Json(OrderResp { total: total + tax }).into_response()
}
```

```rust
// GOOD: handler is a thin adapter. Use case contains the logic.
async fn place_order(
    State(uc): State<Arc<PlaceOrderUseCase>>, Json(req): Json<OrderReq>,
) -> Result<Json<OrderResp>, ApiError> {
    let out = uc.execute(req.into()).await?;
    Ok(Json(out.into()))
}
// use_case.rs — pure, testable, callable from HTTP or CLI
pub struct PlaceOrderUseCase<R: OrderRepository> { orders: R, pricer: OrderPricer }
```

### 9. Fire-and-forget `tokio::spawn`

```rust
// BAD: handle dropped — if the task panics, the error is swallowed
// and nothing else notices. Task also leaks if the runtime outlives
// the expected completion.
tokio::spawn(async move {
    process_batch(items).await;
});
```

```rust
// GOOD: track handles in a JoinSet or store them in your state.
let mut set = tokio::task::JoinSet::new();
set.spawn(async move { process_batch(items).await });

while let Some(result) = set.join_next().await {
    if let Err(e) = result? { tracing::error!(?e, "batch failed"); }
}
```

### 10. No timeout on an external call

```rust
// BAD: a slow external host can block this call indefinitely.
// In a web handler, the request hangs; in a worker, it stalls the queue.
let body = reqwest::get("https://api.example.com/data").await?.text().await?;
```

```rust
// GOOD: wrap every outbound call with an explicit timeout.
let body = tokio::time::timeout(
    Duration::from_secs(5),
    reqwest::get("https://api.example.com/data"),
).await??
.text()
.await?;
```

### 11. Non-idempotent retryable operation

```rust
// BAD: webhook handler or Oban/queue-consumer. Retry duplicates the
// charge — classic billing bug.
async fn handle_webhook(event: StripeEvent, db: &PgPool) -> Result<()> {
    let amount = event.amount;
    charge_customer(event.customer_id, amount).await?;  // Retried? → charged twice.
    Ok(())
}
```

```rust
// GOOD: idempotency key de-duplicates retries.
async fn handle_webhook(event: StripeEvent, db: &PgPool) -> Result<()> {
    let mut tx = db.begin().await?;
    let inserted = sqlx::query!(
        "INSERT INTO processed_events (event_id) VALUES ($1) ON CONFLICT DO NOTHING",
        event.id,
    ).execute(&mut *tx).await?.rows_affected();
    if inserted == 0 { return Ok(()); }  // Already processed.
    charge_customer(event.customer_id, event.amount).await?;
    tx.commit().await?;
    Ok(())
}
```

### 12. Circuit breaker in domain code

```rust
// BAD: domain logic knows about network failures, retries, breakers.
// The business rule is polluted with infrastructure concerns.
impl OrderService {
    pub async fn fulfill(&self, id: OrderId) -> Result<(), DomainError> {
        for attempt in 0..3 {
            match self.shipping.ship(id).await {
                Ok(_) => return Ok(()),
                Err(_) if attempt < 2 => tokio::time::sleep(backoff(attempt)).await,
                Err(_) => return Err(DomainError::ShippingDown),
            }
        }
        unreachable!()
    }
}
```

```rust
// GOOD: resilience wraps the infra adapter, not the domain.
// domain: ResilientShippingGateway impls ShippingGateway, delegates to
// inner with retries/circuit-break. Domain sees only ShippingGateway.
pub struct ResilientShipping<S: ShippingGateway> { inner: S, breaker: CircuitBreaker }
impl<S: ShippingGateway> ShippingGateway for ResilientShipping<S> {
    async fn ship(&self, id: OrderId) -> Result<(), GatewayError> {
        self.breaker.call(|| retry_with_backoff(|| self.inner.ship(id))).await
    }
}
// OrderService still just calls `self.shipping.ship(id).await?` — clean.
```

### 13. Unit test requires a real database

```rust
// BAD: any change to the use case forces a DB round-trip. Slow suite,
// flaky on CI, hides logic bugs behind environment issues.
#[tokio::test]
async fn places_order() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await.unwrap();
    let uc = PlaceOrderUseCase::new(pool);
    assert!(uc.execute(input()).await.is_ok());
}
```

```rust
// GOOD: inject the trait, mock it for unit tests. Reserve real-DB
// tests for integration-level paths (#[sqlx::test] or testcontainers).
#[tokio::test]
async fn places_order_when_repo_saves() {
    let mut mock = MockOrderRepo::new();
    mock.expect_save().times(1).returning(|_| Ok(()));
    let uc = PlaceOrderUseCase::new(mock);
    assert!(uc.execute(input()).await.is_ok());
}
```

### 14. Domain entity with framework annotations

```rust
// BAD: domain type coupled to sqlx AND serde AND the HTTP response
// shape. Change the DB schema → change domain. Change the JSON schema
// → change domain. Domain now knows about transport/storage details.
#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Order {
    #[sqlx(rename = "order_id")]
    pub id: i64,
    #[serde(rename = "customerId")]
    pub customer: i64,
    pub status: String,
}
```

```rust
// GOOD: domain is plain Rust. Separate DTOs in infra (for DB) and
// api (for HTTP). Translate at the adapter.
// domain/order.rs — framework-free
pub struct Order { pub id: OrderId, pub customer: CustomerId, pub status: OrderStatus }

// infra/postgres.rs
#[derive(sqlx::FromRow)]
struct OrderRow { order_id: i64, customer_id: i64, status: String }
impl From<OrderRow> for Order { fn from(r: OrderRow) -> Self { /* ... */ } }

// api/dto.rs
#[derive(serde::Serialize)]
struct OrderResponse { id: String, customer_id: String, status: String }
impl From<Order> for OrderResponse { fn from(o: Order) -> Self { /* ... */ } }
```

### 15. `new()` panics on config / resource error

```rust
// BAD: panic on startup → no Result surface, no graceful handling,
// no way for tests to exercise the failure path.
impl DbClient {
    pub fn new(url: &str) -> Self {
        let pool = PgPool::connect_lazy(url).expect("bad DATABASE_URL");
        Self { pool }
    }
}
```

```rust
// GOOD: return Result. main.rs decides whether to panic, retry, or
// degrade; tests can cover the error branch.
#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
}

impl DbClient {
    pub fn new(url: &str) -> Result<Self, InitError> {
        let pool = PgPool::connect_lazy(url)?;
        Ok(Self { pool })
    }
}
```

### 16. Duplicated business rule across layers (SSOT violation)

```rust
// BAD: the "order must have at least one line item and total > 0" rule
// is redeclared in the API validator, the domain constructor, and the
// DB trigger. Three sources of truth for one fact — they will drift.
fn validate_api(req: &OrderReq) -> Result<(), ApiError> {
    if req.items.is_empty() { return Err(ApiError::Empty); }
    if req.total() <= Decimal::ZERO { return Err(ApiError::ZeroTotal); }
    Ok(())
}
impl Order {
    pub fn new(items: Vec<LineItem>) -> Result<Self, DomainError> {
        if items.is_empty() { return Err(DomainError::Empty); }
        let total: Decimal = items.iter().map(|i| i.price * i.qty).sum();
        if total <= Decimal::ZERO { return Err(DomainError::ZeroTotal); }
        Ok(Self { items, total })
    }
}
// plus a CHECK constraint in the migration, easily forgotten when rules change
```

```rust
// GOOD: the domain constructor IS the rule. API and DB delegate to it.
// One place to change when the rule evolves; compiler/constructor enforces
// that no invalid Order value can exist.
impl Order {
    pub fn place(items: Vec<LineItem>) -> Result<Self, OrderError> {
        if items.is_empty() { return Err(OrderError::Empty); }
        let total: Decimal = items.iter().map(|i| i.price * i.qty).sum();
        if total <= Decimal::ZERO { return Err(OrderError::ZeroTotal); }
        Ok(Self { items, total })
    }
}
// API handler: map OrderError -> ApiError via From.
// DB layer: inserts are fed from Order values only, which are already valid.
// Migration CHECK constraint (if kept at all) is a belt-and-braces safety,
// not the primary source of truth.
```

### 17. Magic value duplicated as literal across modules (SSOT violation)

```rust
// BAD: the 30-second request timeout lives as a literal in five places.
// When the SLA changes to 45s, someone updates two of them and misses three.
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30)).build()?;
// and later, in another file:
tokio::time::timeout(Duration::from_secs(30), db.query("...")).await?;
// and in a test:
#[tokio::test] async fn completes_within_budget() {
    assert!(run().await.duration < Duration::from_secs(30));
}
```

```rust
// GOOD: one const, one place to change, grep-verifiable.
// domain/timeouts.rs
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

// Everywhere else:
use domain::timeouts::REQUEST_TIMEOUT;
let client = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build()?;
tokio::time::timeout(REQUEST_TIMEOUT, db.query("...")).await?;
```

**The `policy.rs` pattern — one module, all the knobs.** A single file that holds every tunable (timeouts, retry counts, redirect limits, user-agent strings, concurrency defaults) is more discoverable than scattering individual constants by topic. The module's own doc comment states the grep litmus — future contributors have a mechanical check to verify SSOT on their diff.

```rust
//! Single Source of Truth for policy constants.
//!
//! Every timeout, retry count, concurrency default, and wire-format
//! string lives here. CLI flags may override at the call site — but
//! the *default* lives only in this file.
//!
//! Litmus: `grep -rn "Duration::from" src/` returns hits only in this
//! file and test fixtures.

use std::time::Duration;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
pub const DEFAULT_CONCURRENCY: usize = 16;
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const INITIAL_BACKOFF: Duration = Duration::from_millis(250);
pub const MAX_BACKOFF: Duration = Duration::from_secs(10);
pub const REDIRECT_LIMIT: u8 = 10;
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
```

Why one file rather than per-topic constants next to the code that uses them: the operator who changes the SLA thinks "retry and timeout," not "the retry constant is in retry.rs and the timeout is in client.rs." One file is the contract.

---

## 17. Architectural Paradigms Not Covered Here

This skill assumes conventional application / service / library architectures with traits, generics, and the type system as the primary organizing mechanisms. A handful of domains use fundamentally different paradigms where this skill's rules apply only partially:

- **Entity-Component-System (ECS)** for games and simulations (Bevy, hecs, specs). State lives in a `World`; components are data; systems are functions that query. Archetypes group entities with identical component sets for cache efficiency. Dependency inversion, "traits as ports," and hexagonal layering don't apply the same way — the ECS framework IS the architecture, and your job is authoring systems and components. If you're building on Bevy, lean on Bevy's documentation first; use this skill for the non-ECS parts (error handling, async, testing, workspace layout).
- **Incremental-query compiler tooling** (rust-analyzer, rustc's query system) uses **salsa** or similar memoization frameworks. The architecture is a graph of queries where results are cached and invalidated on input changes. Trait placement, composition-root, and hexagonal layering apply differently — the DB-of-queries is the application. If you're building query-heavy derived-state tooling (compiler, static analyzer, build tool, LSP server), learn salsa's patterns first.
- **Kernel / bare-metal / `no_std` systems** (Redox kernel, embassy-based firmware). No heap by default, no unwinding (`panic = "abort"`), no std collections. Errors are often integer codes at ABI boundaries rather than `Result<T, E>`. Hardware registers accessed via raw pointers within documented safety contracts. Use this skill for the organizational parts (workspace, feature flags, lints); lean on kernel/embedded-specific references for primitives (`spin`, `linked_list_allocator`, `bitfield`, etc.) and the [rust-planning/error-strategy.md](error-strategy.md) §7.5 syscall-boundary pattern.
- **Async-first embedded firmware** (embassy, embassy-rp for RP2040/RP2350, esp-hal with embassy integration, embassy-nrf, embassy-stm32). Similar to kernel/bare-metal but organized around an async executor that runs tasks cooperatively. The ecosystem is: `critical-section` (pluggable critical-section primitive), `portable-atomic` (atomics polyfill for platforms without them), `defmt` (binary logging), `heapless` (compile-time-bounded no_std collections), `static_cell` (statically-allocated runtime-init cells for singletons like the executor), plus platform-specific HALs. Architecture selection is done via Cargo features (`platform-cortex-m`, `platform-riscv32`, etc.) and chip selection via mutually-exclusive features (`rp2040` vs `rp235xa` vs `rp235xb`). Async peripherals are the default API — there is no sync/async toggle. For chip-specific deep dives, load the chip skill: `rp2040`, `rp2350`, `esp32-c`. Use this skill for the organizational parts.
- **Actor-model frameworks** (Actix, Ractor) have their own architectural rules — messages, addresses, supervisors — that supersede the generic "dependency-inject via trait" pattern.
- **Declarative GUI DSLs** (Leptos, Dioxus, Yew) use a signal/reactive model. Same story: framework-specific patterns take precedence.

For these, load the relevant ecosystem documentation alongside this skill rather than treating this skill as authoritative for their paradigm.

---

## 18. Related Skills

- **[rust-implementing](../rust-implementing/SKILL.md)** — The moment of writing code. Decision tables for `?` vs `match`, `impl Trait` vs `dyn Trait`, `Arc<Mutex>` vs channels. Idiomatic templates. TDD workflow. Anti-patterns Claude commonly produces.
- **[rust-reviewing](../rust-reviewing/SKILL.md)** — Reviewing PRs, debugging bugs (panics, OOM, deadlock, UB), profiling (flamegraph, perf, DHAT, tokio-console).
- **[rust-nif](../rust-nif/SKILL.md)** — Rust NIFs with Rustler for Elixir/BEAM integration. Load alongside this skill when planning a NIF crate.
- **[elixir-planning](../elixir-planning/SKILL.md)** — Elixir architecture planning; load when designing BEAM-side of a Rust-NIF application.
- **[c-programming](../c-programming/SKILL.md)** — C patterns for the other side of an FFI boundary.
- **[skill-authoring](../skill-authoring/SKILL.md)** — For extending or authoring skills.

---

## 19. References

**Rust language and ecosystem:**
- [The Rust Programming Language](https://doc.rust-lang.org/book/) — official book
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — example-driven guide
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) — library design checklist
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) — formatting and layout
- [The Cargo Book](https://doc.rust-lang.org/cargo/) — workspace, features, dependencies

**Architecture and patterns:**
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/) — Alistair Cockburn's original
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html) — Robert Martin
- [Domain-Driven Design](https://www.domainlanguage.com/ddd/) — Eric Evans
- Rust production codebases studied: tokio, axum, hyper, serde, ripgrep, rust-analyzer

**Async:**
- [Tokio documentation](https://tokio.rs/) — runtime, channels, utilities
- ["Async: What is blocking?" by alice rhyl](https://ryhl.io/blog/async-what-is-blocking/) — foundational

**Testing:**
- [cargo test documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [mockall book](https://docs.rs/mockall/latest/mockall/)
- [proptest book](https://proptest-rs.github.io/proptest/)
- [cargo-fuzz book](https://rust-fuzz.github.io/book/)
- [insta documentation](https://insta.rs/)

**Resilience and production:**
- [Designing Distributed Systems](https://www.oreilly.com/library/view/designing-distributed-systems/9781491983638/) — Burns
- [Release It!](https://pragprog.com/titles/mnee2/release-it-second-edition/) — Nygard (circuit breakers, bulkheads, timeouts)
