# Rust Phase Skills — Migration Plan

Split `rust-programming` (single 22-file skill) into three phase-aligned skills, mirroring the elixir-planning / elixir-implementing / elixir-reviewing pattern.

## Source

`~/repos/Rust_programming_skill/` — 22 .md files, ~48K lines total.

## Target structure

```
rust-phase-skills/
├── rust-planning/       (what to build)
├── rust-implementing/   (how to type it)
└── rust-reviewing/      (what's wrong with it)
```

## Phase assignment

### rust-planning — architectural decisions BEFORE writing code

| Subskill | Source | Content |
|---|---|---|
| SKILL.md | NEW (drawing on architecture.md + current SKILL.md) | Rules, planning workflow, master decision tables, workspace/crate shape, architecture styles, error strategy, async strategy, unsafe budget, dependency philosophy, resilience planning |
| architecture-patterns.md | architecture.md (most of it) | Hexagonal, layered, DDD, facade, enum-dispatch, Tower composition, growing architecture |
| workspace-layout.md | architecture.md §workspaces + SKILL.md Cargo sections | `[workspace.dependencies]`, `[workspace.lints]`, feature flag architecture, project-layout decisions |
| domain-patterns.md | domain-patterns.md | DDD, aggregates, CQRS, event sourcing, bounded contexts as crates |
| async-strategy.md | async-concurrency.md planning parts | Sync vs async, runtime choice, actor vs channels, task budget, structured concurrency |
| error-strategy.md | error-handling.md planning parts | thiserror vs anyhow, error boundaries, hand-rolled Error+ErrorKind, multi-layer translation |
| unsafe-strategy.md | unsafe-ffi.md planning parts | When unsafe is justified, FFI strategy, safety contracts, abort-on-panic at boundary |
| services-architecture.md | services.md | Microservices, kernel pattern, resilience (circuit breakers, retries), service discovery |
| data-strategy.md | database.md planning + domain-patterns.md | Store choice (SQL vs KV vs embedded), ownership, migration strategy, caching |
| test-strategy.md | testing.md planning parts + NEW | **FIRST CLASS**: test pyramid, mocking strategy, property test scope, fuzzing scope, CI strategy, coverage goals, test as design driver |
| distributed-rust.md | services.md distributed parts + NEW | (If applicable) Multi-node patterns, gRPC/tonic, distributed consensus |

### rust-implementing — idiomatic code at the keyboard

| Subskill | Source | Content |
|---|---|---|
| SKILL.md | Current SKILL.md, trimmed | Rules, master "which construct" table, daily-coding patterns, BAD/GOOD, TDD section |
| language-patterns.md | language-patterns.md | Ownership patterns, iterators, closures, pattern matching, trait patterns |
| error-handling.md | error-handling.md impl parts | `?`, thiserror derive, anyhow context, error conversion |
| type-system.md | type-system.md | Traits, generics, Pin/Unpin, GATs, const generics, type state |
| async-patterns.md | async-concurrency.md impl parts | Tokio, channels, JoinSet, tower Service, async closures |
| serde-patterns.md | serde-serialization.md | Derives, attributes, custom serde, zero-copy |
| macros.md | macros.md | Declarative, procedural, derive, attribute macros |
| ffi-patterns.md | unsafe-ffi.md impl parts | Raw pointers, `repr(C)`, CString/CStr, bindgen/cbindgen |
| web-apis.md | web-apis.md | Axum, extractors, middleware, auth |
| database.md | database.md impl parts | SQLx, Diesel, queries, transactions |
| cli-tools.md | cli-tools.md | clap, indicatif, crossterm |
| gui-wasm.md | gui-wasm.md | egui, iced, Leptos/Yew, WASM |
| testing-patterns.md | testing.md impl parts + NEW | **FIRST CLASS**: cargo test, mockall, insta, proptest, cargo-fuzz, TDD templates, async test patterns |
| documentation.md | documentation.md | Rustdoc, doc tests, intra-doc links |
| quick-reference.md | quick-reference.md | Daily-use function reference |

### rust-reviewing — inspecting existing code

| Subskill | Source | Content |
|---|---|---|
| SKILL.md | NEW | Rules, three modes (review/debug/profile), workflows, severity, review checklists |
| anti-patterns-catalog.md | skill-production-references.md + extracted BAD cases | Organized anti-patterns by area |
| debugging-playbook.md | NEW | Symptom→tool: panic, memory growth, deadlock, slow async, flaky test, UB |
| profiling-playbook.md | data-structures.md profiling + deployment.md | Tool selection: criterion, flamegraph, perf, DHAT, tokio-console, samply |
| performance-catalog.md | Extracted from data-structures.md + async-concurrency.md | Common pitfalls with symptom→cause→fix |
| security-audit.md | NEW + extracted from web-apis.md auth + unsafe-ffi.md | Input validation, crypto primitives, unsafe review, dep audit |
| test-quality-review.md | testing.md review parts + NEW | **FIRST CLASS**: reviewing test quality, flaky tests, brittle tests, coverage gaps |
| refactor-templates.md | NEW | Common before/after patterns (Arc<Mutex>→channels, Box<dyn>→enum, etc.) |

## Size expectations (matching elixir-phase-skills scale)

| Skill | SKILL.md | Subskill total | Combined |
|---|---|---|---|
| rust-planning | ~2000-2500 L | ~8000 L | ~10K L |
| rust-implementing | ~2000-2500 L | ~15000 L | ~17K L |
| rust-reviewing | ~1500 L | ~5000 L | ~6K L |

## Cross-references

Each SKILL.md links the other two skills with brief summaries ("load X for Y"). Subskills link peers across skills where content relates (e.g., planning/test-strategy.md ↔ implementing/testing-patterns.md ↔ reviewing/test-quality-review.md).

## Deployment

Matching elixir pattern: each skill dir copied to `~/.claude/skills/<skill-name>/`.

## Work plan

1. Skeleton + this plan (done)
2. Three SKILL.md hubs (parallel)
3. Subskill migration from existing rust-programming (parallel, lift-and-shift where possible)
4. New authoring: debugging-playbook, security-audit, refactor-templates, test-quality-review
5. Deploy + commit
