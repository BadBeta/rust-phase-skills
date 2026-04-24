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
2. Three SKILL.md hubs (sequential, hand-authored)
3. Subskill migration from existing rust-programming (hand-authored, file by file)
4. New authoring: debugging-playbook, security-audit, refactor-templates, test-quality-review
5. Deploy + commit

## Ground rules

- **No agents for authoring.** Research via agents is OK; content generation is hand-authored.
- **Preserve all content.** Every rule, BAD/GOOD pair, decision table, code example, and explanation from the old skill must land somewhere in the new skill family. Restructuring is lossless.
- **No hurry.** Multi-session work is expected. Commit after each logical milestone.

## Source-file coverage tracker

Track every source file through its migration so nothing is dropped.

| Source file | Lines | Status | Destination(s) |
|---|---|---|---|
| SKILL.md | 2714 | pending | rust-implementing/SKILL.md (trimmed core) + split of Testing Essentials, Profiling & Performance, Related Skills |
| architecture.md | 3036 | pending | rust-planning/SKILL.md (planning workflow, rules, decision tables) + rust-planning/architecture-patterns.md (hexagonal, DDD, facade, growing architecture) + rust-planning/workspace-layout.md (workspaces, feature flags) |
| architecture-examples.md | 2662 | pending | rust-implementing/architecture-examples.md (lift-and-shift) + cross-linked from planning |
| async-concurrency.md | 3694 | pending | rust-planning/async-strategy.md (sync-vs-async, runtime choice, actor-vs-channels) + rust-implementing/async-patterns.md (code patterns) + rust-reviewing/debugging-playbook.md (deadlock, mailbox buildup) |
| cli-tools.md | 1784 | pending | rust-implementing/cli-tools.md (lift-and-shift) |
| database.md | 1506 | pending | rust-planning/data-strategy.md (store choice, migrations) + rust-implementing/database.md (queries, connection pools) |
| data-structures.md | 2541 | pending | rust-implementing/data-structures.md (collections, algorithms) + rust-reviewing/profiling-playbook.md (benchmarking, flamegraph) + rust-reviewing/performance-catalog.md |
| deployment.md | 2322 | pending | rust-planning/deployment-strategy.md (profile choices, CI architecture) + rust-implementing/observability.md (tracing, metrics) + rust-reviewing/profiling-playbook.md |
| documentation.md | 708 | pending | rust-implementing/documentation.md (lift-and-shift) |
| domain-patterns.md | 2633 | pending | rust-planning/domain-patterns.md (lift-and-shift with minor reframing) |
| error-handling.md | 1512 | pending | rust-planning/error-strategy.md (error boundaries, layer translation strategy) + rust-implementing/error-handling.md (derive patterns, `?`, context) |
| gui-wasm.md | 1346 | pending | rust-implementing/gui-wasm.md (lift-and-shift) |
| language-patterns.md | 2294 | pending | rust-implementing/language-patterns.md (lift-and-shift) |
| macros.md | 1845 | pending | rust-implementing/macros.md (lift-and-shift) |
| quick-reference.md | 1937 | pending | rust-implementing/quick-reference.md (lift-and-shift) |
| serde-serialization.md | 1055 | pending | rust-implementing/serde-patterns.md (lift-and-shift) |
| services.md | 3544 | pending | rust-planning/services-architecture.md (microservices, kernel, resilience) + rust-planning/distributed-rust.md (TCP, TLS, service discovery) |
| skill-production-references.md | 249 | pending | rust-reviewing/anti-patterns-catalog.md (evidence-based rule challenges) |
| testing.md | 3340 | pending | rust-planning/test-strategy.md (test pyramid, mocking strategy, CI) + rust-implementing/testing-patterns.md (cargo test, mockall, insta, proptest, TDD templates) + rust-reviewing/test-quality-review.md (reviewing test quality) |
| type-system.md | 2115 | pending | rust-implementing/type-system.md (lift-and-shift) |
| unsafe-ffi.md | 2601 | pending | rust-planning/unsafe-strategy.md (when unsafe is justified) + rust-implementing/ffi-patterns.md (code patterns, bindgen/cbindgen) |

**Coverage goal:** Sum of new skill line counts ≥ sum of old skill line counts. Cross-reference each migrated section.

## New authoring (not from source)

| New file | Scope |
|---|---|
| rust-planning/SKILL.md | Planning workflow, rules for architecting, master planning decision table |
| rust-implementing/SKILL.md | Implementing rules, master "which construct?" decision table, TDD workflow |
| rust-reviewing/SKILL.md | Three modes (review/debug/profile), workflows, severity, checklists |
| rust-reviewing/debugging-playbook.md | Symptom→tool flow: panic, OOM, deadlock, slow async, flaky test, UB, Miri findings |
| rust-reviewing/security-audit.md | Input validation, crypto, unsafe audit, cargo-audit, dependency hygiene |
| rust-reviewing/refactor-templates.md | Common before/after: Arc<Mutex>→channels, Box<dyn>→enum, etc. |
| rust-reviewing/test-quality-review.md | Flaky tests, brittle tests, mocking mistakes, coverage gaps |
