---
name: rust-reviewing
description: >
  Rust code inspection — reviewing PRs and diffs, debugging bugs, and profiling performance.
  Covers the full audit toolkit: review checklists by area (architecture, ownership/lifetimes,
  control flow, async correctness, error handling, unsafe, testing, security, performance,
  API design), severity classification, the debugging playbook (panic + backtrace, OOM,
  deadlock, slow async, flaky test, UB from Miri, undefined behavior from sanitizers,
  compile errors, lifetime errors), the profiling playbook (criterion, flamegraph, perf,
  DHAT, tokio-console, samply, cargo-llvm-lines, iai, massif), performance pitfall catalog,
  refactor templates (`Arc<Mutex>` → channels, `Box<dyn>` → enum, `String` → `&str`,
  `Vec<T>` → `&[T]`, nested match → `?`), security audit (cargo-audit, dependency hygiene,
  unsafe review, input validation, crypto primitives), and review-comment style.
  ALWAYS use when reviewing Rust pull requests, diffs, or existing modules.
  ALWAYS use when debugging Rust bugs — panics, memory errors, deadlocks, slow async, flaky tests.
  ALWAYS use when profiling Rust performance — finding bottlenecks, measuring, optimizing.
  ALWAYS use when asked to audit, critique, review, debug, or profile Rust code.
  For designing from scratch load rust-planning; for writing code load rust-implementing.
---

# Rust — Reviewing Skill

The skill for **inspecting existing Rust code**. Three distinct modes, all covered here:

- **Review** — proactive audit of a PR or diff, looking for structural, correctness, safety, and performance issues
- **Debug** — reactive investigation of a specific bug or crash (panic, deadlock, OOM, UB, flaky test)
- **Profile** — reactive investigation of a performance problem (latency, throughput, memory, compile time)

This skill is the third in the Rust family. The three divide labor by phase:

| Phase | Skill | Question it answers |
|---|---|---|
| Plan (before writing) | **[rust-planning](../rust-planning/SKILL.md)** | What to build, how to structure it |
| Implement (at keyboard) | **[rust-implementing](../rust-implementing/SKILL.md)** | How to type it idiomatically |
| Inspect (after writing) | **rust-reviewing** (this) | What's wrong with it / where's the bug / why is it slow |

**Primary mode:** decision tables (where to look, what to flag, which tool) + BAD/GOOD pairs. Anti-pattern catalogs live across the three skills; this skill cross-references them and frames each pattern as "flag if you see this" rather than "don't output this."

## How to use this skill

1. **Reviewing a PR / diff?** — Read §1 (Rules), §3 (Review Workflow), §7 (Checklists by area). Scan the diff against each checklist.
2. **Debugging a specific bug?** — §4 (Debug Workflow), §8 (Debugging Playbook — find the symptom).
3. **Profiling performance?** — §5 (Profile Workflow), §9 (Profiling Playbook — pick the right tool), §10 (Common pitfalls).
4. **Writing review comments?** — §6 (Severity Classification), §12 (Comment style).
5. **Unsure if something is worth flagging?** — §6 Severity Classification.

### Subskills — deep inspection references

The core SKILL.md carries the always-loaded rules, severity guidance, workflows, checklists, and a tight playbook. For extended depth on a specific area, load the matching subskill:

| Subskill | Purpose | Load when... |
|---|---|---|
| [anti-patterns-catalog.md](anti-patterns-catalog.md) | Organized anti-patterns by category (architecture, ownership/lifetime, control flow, async, error handling, unsafe/FFI, API design, testing, security, performance) with BAD/GOOD for each, plus evidence-based rule challenges from reading ripgrep/tokio/axum/serde source | General review / scanning a diff for named anti-patterns |
| [debugging-playbook.md](debugging-playbook.md) | Symptom → diagnosis flow for: panic+backtrace, OOM / memory growth, deadlock / no-progress, slow async / contention, flaky test, UB from Miri, UB from sanitizers, compile errors (E0XXX), lifetime errors, macro errors | Investigating a specific bug |
| [profiling-playbook.md](profiling-playbook.md) | Tool selection + usage: `criterion` (micro-benchmark), `flamegraph` (sampling profiler), `perf` (Linux perf counters), `DHAT` (heap profiler), `tokio-console` (async runtime inspector), `samply` (cross-platform sampling), `cargo-llvm-lines` (code-gen bloat), `iai` (cachegrind-based), `massif` (Valgrind heap), memory analysis, compile-time profiling (`cargo-timings`, `-Zself-profile`) | Measuring or optimizing performance |
| [performance-catalog.md](performance-catalog.md) | 30+ common pitfalls with symptom → root cause → fix (ownership/clone, iterators, collections, async, mutex/channel, allocation, monomorphization, hash, string, I/O, serialization) | Looking for performance issues in a review |
| [security-audit.md](security-audit.md) | Security checklist: input validation, injection (SQL, command, log injection), auth/authz, session management, crypto primitive decision table (rustls / ring / RustCrypto / age), secrets management, logging discipline, `cargo-audit` / `cargo-deny`, unsafe review, supply chain (`cargo-vet`) | Conducting a security review |
| [test-quality-review.md](test-quality-review.md) | **FIRST-CLASS** test review: flaky tests (timing, ordering, shared state), brittle tests (implementation coupling), mocking mistakes (mock != production behavior), coverage gaps, test smells, proptest/fuzz coverage assessment, async test patterns, compile-fail tests | Reviewing test files; investigating flaky CI |
| [refactor-templates.md](refactor-templates.md) | Common before/after patterns: `Arc<Mutex<T>>` → channel, `Box<dyn Trait>` → enum dispatch, `String` arg → `&str`/`impl AsRef<str>`, `Vec<T>` → iterator, nested `match` → `?` chain, panic-prone `unwrap` → typed `Result`, blocking call → `spawn_blocking` | Proposing a structural fix during review |

**Cross-skill references:** for WHAT to write (implementation) → `rust-implementing`. For WHY it's shaped that way (architecture) → `rust-planning`.

---

## 1. Rules for Reviewing, Debugging, and Profiling Rust (LLM)

1. **ALWAYS separate severity from correctness.** A review finds *many* things; not all are worth blocking on. Use §6 to classify each finding as block / request-change / suggest / nitpick.
2. **ALWAYS start debugging with the smallest-scope tool** — compile errors, `dbg!`, or a targeted test before reaching for `tokio-console`, `flamegraph`, or Miri. Most bugs surface from reading the compile error or inspecting values at suspect points.
3. **ALWAYS start profiling by measuring, not guessing.** Use `criterion` for micro-benchmarks, `flamegraph` for where time is spent, `tokio-console` for async task/lock behavior. Never "optimize" without evidence.
4. **ALWAYS use `Instant::now()` / `Instant::elapsed()` for duration measurement** — never `SystemTime::now()` (wall-clock, NTP sync causes jumps and non-monotonic readings).
5. **NEVER run heavyweight profilers (`perf`, `DHAT`, Valgrind) in production.** Use sampling profilers with bounded overhead (`samply`, `flamegraph`) or in-process instrumentation. For production use `tokio-console` in targeted debug builds only.
6. **ALWAYS read the symptom before reading the code.** What does "slow" mean — p50, p99, tail? What does "crash" mean — panic with which error? SIGSEGV? abort from panic-across-FFI? A vague symptom wastes time.
7. **ALWAYS suggest the idiomatic refactor** when flagging an anti-pattern. "This is bad" without "do this instead" is low-value feedback. Use `refactor-templates.md` for common before/after patterns.
8. **ALWAYS cross-reference rust-implementing and rust-planning** when flagging a finding — point reviewers to the section that explains *why* the idiomatic form is better.
9. **PREFER letting the type system prevent bugs** over adding runtime checks. Before suggesting a new `assert!` or `if` guard, ask whether a newtype, enum state machine, or sealed trait would prevent the invariant violation at compile time.
10. **NEVER flag style issues that `rustfmt` or `clippy` would catch.** Reviews are for things tools miss: architecture, idioms, subtle correctness, lifetime/ownership shape, async correctness, unsafe invariants, testability, performance.
11. **ALWAYS flag missing tests for new behavior and missing safety docs on `unsafe`** — these are easy to add, compound in value over time, and are the strongest signals that a change was thought through.
12. **PREFER benchmark-driven refactor suggestions over intuition.** "This could be faster" is weak; "this is 40% slower in the linked `criterion` run, with the flamegraph showing 60% time in `alloc::alloc`" is actionable.
13. **ALWAYS verify `unsafe` by reading the block AND its safety comment** — an `unsafe` block with no `// SAFETY:` comment is a block-severity finding by itself.
14. **ALWAYS check for lifetime/borrow smells**: `'static` bounds that shouldn't be needed (suggest refactoring), self-referential structs (suggest `Pin` / indices), lifetimes elided in ways that obscure relationships.
15. **ALWAYS check async correctness**: `MutexGuard` across `.await`, blocking calls in async, missed `Send` bounds, fire-and-forget `tokio::spawn`, missing timeout on external calls.

---

## 2. The Three Modes of Inspection

The three modes of this skill are not interchangeable. Each has a different question, different tools, different output.

| Mode | Question | Primary artifact | Primary output |
|---|---|---|---|
| **Review** | "What's wrong with this diff?" | PR / branch diff | Comments with severity + suggested fix |
| **Debug** | "Why is this bug happening?" | Failing test, crash report, observed misbehavior | Root cause + the minimum fix |
| **Profile** | "Why is this slow / heavy?" | Production telemetry, benchmark, complaint | Identified bottleneck + measured improvement |

**They share one method:** read existing code, form hypotheses about what is wrong, verify with evidence (static — read the code and `Cargo.toml`; dynamic — run it with inspection). They differ in what "wrong" means and what "evidence" means.

**Choose the right mode:**

- Pre-merge, proactive → **Review**
- Post-merge, something's broken → **Debug**
- Post-merge, something's slow / too big / uses too much memory → **Profile**

Mixing modes wastes time: full review on a bug report, ad-hoc debugging on a PR diff, micro-optimization without measurement.

---

## 3. Review Workflow (PR / diff review)

### 3.1 Step-by-step

1. **Read the PR description.** What is the stated intent? Does the diff match?
2. **Read the tests first.** Tests describe behavior; code describes implementation. If the tests are missing or weak, that's the first finding.
3. **Check `Cargo.toml` changes.** New dependencies? Feature flag changes? Version bumps? MSRV bumps? These affect the entire crate.
4. **Scan the diff for architectural smells** (§7.1) — wrong layer (domain importing infra), cross-crate internal access, framework references in domain. Block-severity by default.
5. **Scan for ownership/lifetime smells** (§7.2) — spurious `'static`, unnecessary `.clone()`, `Arc<Mutex>` where a channel fits, self-referential structs without `Pin`.
6. **Scan for correctness issues** (§7.3) — control flow, error propagation, pattern matching, off-by-one, integer overflow.
7. **Scan for async correctness** (§7.4) — `MutexGuard` across `.await`, blocking in async, `Send` bounds, task supervision, timeouts.
8. **Scan for error-handling issues** (§7.5) — `.unwrap()` in production, `String` errors, missing `From` conversions at boundaries, `Box<dyn Error>` in public API.
9. **Scan for `unsafe` issues** (§7.6) — missing `// SAFETY:` comment, unsafe surface too large, unsafe in async without `Send`/`Sync` analysis, FFI panics crossing boundary.
10. **Scan for testing gaps** (§7.7) — missing tests for new behavior, tests that assert implementation instead of behavior, flaky tests, missing property/fuzz for parsers.
11. **Scan for documentation gaps** (§7.8) — missing `/// # Errors`, `/// # Panics`, `/// # Safety` on public functions; missing module-level docs (`//!`).
12. **Scan for security issues** (§7.9) — `String::from_utf8_unchecked` on user input, format-string injection, SQL injection via interpolation, leaked secrets in logs, unvalidated external data, insecure defaults.
13. **Scan for performance** (§7.10, §10) — N+1 queries, quadratic algorithms, excessive allocations in hot paths, contention on `Mutex`, missing iterator fusion.
14. **Classify each finding** (§6) — block / request-change / suggest / nitpick.
15. **Write review comments** (§12) — specific, actionable, linked to the skill section that explains *why*.
16. **Check that the suggested fix is correct** before posting. An incorrect suggestion is worse than no suggestion.

### 3.2 Review decision — what to flag

| Observation | Action |
|---|---|
| Architectural smell (see §7.1) | Flag. Severity: block if it crosses a boundary; request-change if it reinforces a bad pattern |
| Ownership / lifetime smell (spurious `.clone()`, `'static`, etc.) | Flag. Severity: suggest unless it's wrong (then request-change) |
| `unsafe` without `// SAFETY:` comment | Flag. Severity: **block** |
| Control-flow anti-pattern (see §7.3) | Flag. Severity: request-change or suggest depending on reach |
| Missing `# Errors` / `# Panics` / `# Safety` on new public function | Flag. Severity: request-change |
| Missing test for new behavior | Flag. Severity: **block** |
| Test that asserts implementation, not behavior | Flag. Severity: request-change — brittle test, blocks future refactors |
| Style inconsistency `rustfmt` / `clippy` would catch | **Do NOT flag.** Let the tools handle it |
| `clippy` finding already present | **Do NOT flag in the PR** — raise separately or add to baseline |
| Taste preference with no evidence of harm | **Do NOT flag** — save capital for real findings |
| Security issue (injection, unvalidated input, leaked secret) | Flag. Severity: **block** |
| Performance issue with no measurement | Flag as question: "have you measured this?" — do not block on speculation |
| Performance issue with measurement | Flag. Severity depends on impact |
| `MutexGuard` held across `.await` | Flag. Severity: **block** (can deadlock) |
| Blocking call (sync I/O, `std::thread::sleep`) in `async` | Flag. Severity: request-change or block if in hot path |
| Fire-and-forget `tokio::spawn` | Flag. Severity: request-change |
| External call without timeout | Flag. Severity: request-change |

### 3.3 What to check in every review

- [ ] Tests exist for the new behavior, and they pass
- [ ] Tests assert behavior, not internal calls
- [ ] `# Errors` / `# Panics` / `# Safety` sections on new/modified public functions
- [ ] No `#![allow(...)]` without justification
- [ ] No `unsafe` without `// SAFETY:` comment
- [ ] No cross-crate / cross-layer access to `pub(crate)` internals via workarounds
- [ ] No `.unwrap()` / `.expect()` in non-test, non-init code paths
- [ ] No `MutexGuard` / `RwLockGuard` held across `.await`
- [ ] No blocking calls (sync I/O, `std::thread::sleep`) in async contexts
- [ ] No fire-and-forget `tokio::spawn` (handles tracked via `JoinSet` or stored)
- [ ] Idempotent for retried operations (webhook handlers, queue consumers, background jobs)
- [ ] Timeouts set where appropriate (HTTP, DB, RPC)
- [ ] No `panic!` / `unreachable!` that should be `Result::Err`
- [ ] `Cargo.toml` changes reviewed (new deps, feature bumps, MSRV changes)
- [ ] No dependencies from the domain crate on infrastructure crates

---

## 4. Debug Workflow (finding a specific bug)

### 4.1 Step-by-step

1. **Reproduce the bug** — a bug you can't reproduce can't be fixed with confidence. Try: exact inputs from the report, similar inputs, boundary conditions, concurrent access.
2. **Write the failing test** (TDD-for-bugs) — the reproduction IS a regression test. See `rust-implementing/testing-patterns.md`.
3. **Read the error** — full backtrace, panic message, address if SIGSEGV, thread name. `RUST_BACKTRACE=full` if needed.
4. **Form a hypothesis** about which code path produces the bug.
5. **Verify with evidence** — add `dbg!(value)` at the hypothesized point, or use a debugger (`rust-lldb`, `rust-gdb`, VS Code's debugger). Is the value what you expected?
6. **If the hypothesis is wrong**, widen the inspection outward. Walk backwards through the call chain.
7. **For async-level bugs** (timeouts, stalls, mailbox buildup, crashes), use `tokio-console`, `RUST_LOG=trace`, or instrument with `tracing::debug!`.
8. **For UB/memory bugs**, run under Miri (`cargo +nightly miri test`) or AddressSanitizer (`RUSTFLAGS="-Zsanitizer=address"`).
9. **For concurrency/flaky bugs**, check shared state, timing assumptions, `Drop` order, panic-in-thread handling. Consider `loom` for lock-free code.
10. **Once found, consider whether other places have the same bug.** A single bug often has family members — grep for the same pattern.
11. **Commit the failing test alongside the fix** so the bug is guarded against regression.

### 4.2 Debug decision — which tool

| Question | Tool | Load for detail |
|---|---|---|
| What is this value at this point? | `dbg!(value)` — prints file:line + value | — |
| What does this expression produce? | `dbg!(expr)` — evaluates and returns, can be inserted mid-expr | — |
| Which branch ran? | `eprintln!("branch A")` / `tracing::debug!(target: "my-mod", "branch A")` | — |
| Backtrace | `RUST_BACKTRACE=1` (short) or `RUST_BACKTRACE=full` | — |
| Step through code | `rust-lldb ./target/debug/foo` or VS Code debugger | `debugging-playbook.md` |
| Inspect binary / assembly | `cargo asm`, `cargo-show-asm`, `cargo-llvm-lines` | `profiling-playbook.md` |
| Async task/lock state | `tokio-console` (needs `tokio = { features = ["tracing"] }` + `console-subscriber`) | `debugging-playbook.md` |
| UB / data race | `cargo +nightly miri test` | `debugging-playbook.md` |
| Memory error / leak | AddressSanitizer / LeakSanitizer (nightly); Valgrind for non-sanitizer-compatible; DHAT for heap profiling | `debugging-playbook.md`, `profiling-playbook.md` |
| Flaky test | Repeat until fail: `cargo test -- --test-threads=1 --ignored` loops; add `println!`s; `loom` for lock-free | `debugging-playbook.md` |
| Compile error | Read E0XXX code; `rustc --explain EXXXX`; `cargo check` faster than build | `debugging-playbook.md` |

### 4.3 Start small, escalate only when needed

- **First reach**: read the compile error / panic backtrace / test output carefully.
- **Next**: `dbg!` at suspect points. Most bugs found here.
- **Next**: targeted test case that exercises the bug in isolation.
- **Next**: debugger for step-through; Miri for UB; `tokio-console` for async stalls.
- **Next**: `loom` for lock-free concurrency bugs; `cargo-fuzz` for input-dependent bugs.
- **Never jump to heavy tools first.** Reading the error and `dbg!` solve 80%.

---

## 5. Profile Workflow (finding a performance problem)

### 5.1 Step-by-step

1. **Define the metric.** Latency (p50/p95/p99)? Throughput (req/s)? Memory (RSS, allocations/s)? Compile time? CPU usage? Each has different tools.
2. **Establish a baseline** with `criterion` (micro-benchmark), `wrk`/`oha` (HTTP load), or production telemetry. You can't know if you improved things without a baseline.
3. **Find the hotspot.** Sampling profiler (`flamegraph`, `samply`) for CPU time. `DHAT` or `heaptrack` for heap. `tokio-console` for async runtime stalls. `cargo-llvm-lines` for codegen bloat.
4. **Form a hypothesis** about the cause (cloning, allocation, mutex contention, N+1, missing index, unnecessary async hop).
5. **Verify with evidence** — a flamegraph showing 60% time in `HashMap::insert` points to allocation or hash cost; a tokio-console showing a task waiting 500ms on a lock points to contention.
6. **Make the fix** (smallest possible change). Re-measure.
7. **Compare against baseline.** Did it improve? By how much? Regression-test the improvement with `criterion` in CI if possible.
8. **Update the flamegraph** — the next hotspot is now different.
9. **Stop when the benefit no longer justifies the complexity.**

### 5.2 Profile decision — which tool

| Need | Tool | Details |
|---|---|---|
| Micro-benchmark a function | `criterion` | Statistical, compares runs, CI-friendly |
| Where does CPU time go (user mode)? | `flamegraph` / `samply` / `perf record` + `perf report` | Sampling profiler, bounded overhead |
| Where does CPU time go (kernel too)? | `perf` (Linux), `Instruments` (macOS) | Captures kernel + user |
| Cache misses / branch mispredicts | `perf stat -e cache-misses,branch-misses` | Low-level hardware counters |
| Heap allocation pattern | `DHAT` (Valgrind), `heaptrack` (Linux), `dhat-rs` in-process | Shows where allocations happen |
| Memory leak | `valgrind --tool=memcheck`, LeakSanitizer | Finds forgotten memory |
| Async runtime behavior (task state, locks, I/O) | `tokio-console` | Specific to Tokio; enable via `tracing` + `console-subscriber` |
| Lock contention | `tokio-console` + `parking_lot` deadlock detection | Shows wait times |
| Compile time | `cargo build --timings`, `-Zself-profile`, `cargo-llvm-lines` | Find expensive-to-compile crates/functions |
| Binary size | `cargo-bloat`, `cargo-llvm-lines`, `strip` analysis | Find what's big in the output |
| Cachegrind (deterministic CPU cycles) | `iai` | Reproducible, unaffected by system noise |

### 5.3 Don't optimize

- **Don't optimize without a benchmark.** Intuition is wrong more often than right in Rust because of zero-cost abstractions.
- **Don't optimize outside the hotspot.** The flamegraph tells you what's hot; everything else is wasted effort.
- **Don't micro-optimize before algorithmic fixes.** Going from O(n²) to O(n log n) beats any constant-factor optimization.

---

## 6. Severity Classification

When writing review comments, classify each finding. Block-severity findings prevent merge; lower severities ride along.

| Severity | Meaning | Examples |
|---|---|---|
| **block** | Merge must not happen until fixed | Security hole, `unsafe` without SAFETY comment, `MutexGuard` across `.await`, missing test for new public behavior, domain importing infra crates, UB risk |
| **request-change** | Should fix before merge, but mergeable if author justifies | Control-flow anti-pattern, error-handling smell, missing docs on public fn, brittle test, slow CI test |
| **suggest** | Better if changed; merge-OK without | Clearer naming, slightly more idiomatic construct, minor refactor |
| **nitpick** | Opinion; author may ignore | Prefer `if let` over `match`, `.iter()` vs `.into_iter()` in context where both work |

**Budget your capital.** Over-flagging (lots of nitpicks) trains authors to ignore you. Reserve block/request-change for things that matter.

---

## 7. Review Checklists (by area)

### 7.1 Architecture

- [ ] Dependency direction: domain has no infra imports (check `Cargo.toml`)
- [ ] Traits defined where USED (domain/app), not where implemented (infra)
- [ ] New trait: small and focused (not a fat catch-all)
- [ ] Framework annotations only in infra/api layers (not domain entities)
- [ ] `#[non_exhaustive]` on public enums and errors that may grow
- [ ] Composition root (`main.rs`) is the only place that names all concrete types
- [ ] No global mutable state / `LazyLock<Mutex<T>>` / `lazy_static!` for mutable services
- [ ] No `Box<dyn Error>` in public library APIs

### 7.2 Ownership, lifetimes, borrowing

- [ ] No `.clone()` added to silence the borrow checker without justification
- [ ] No `'static` bounds that shouldn't be needed (usually indicates wrong ownership)
- [ ] No self-referential structs without `Pin` or indexing
- [ ] Function signatures prefer `&str` / `&[T]` over `&String` / `&Vec<T>`
- [ ] Function signatures prefer `impl AsRef<str>` / `impl Into<String>` where flexibility helps
- [ ] Return types: prefer `impl Iterator` over `Vec` when caller may collect
- [ ] No unnecessary `Arc` when a reference would do
- [ ] No unnecessary `RefCell` / `Cell` (smell for bad ownership design)

### 7.3 Control flow

- [ ] `match` for exhaustive enum dispatch, not chained `if let`
- [ ] `let-else` for "bind or diverge" (replaces `if let ... else return`)
- [ ] `if let` chains (Rust 2024) for multi-pattern chains instead of nested
- [ ] `matches!` for boolean pattern tests
- [ ] `?` for error propagation, not `match { Ok/Err }`
- [ ] No `if let Some(x) = opt { ... }` when `.map` / `.and_then` fits
- [ ] No nested `match` that a `with`-style `?` chain would flatten
- [ ] Exhaustive match arms; if adding `_` catch-all, prefer `#[non_exhaustive]` consideration

### 7.4 Async correctness

- [ ] No `std::sync::MutexGuard` held across `.await`
- [ ] No `std::thread::sleep` / synchronous I/O in async context
- [ ] `tokio::task::spawn_blocking` for unavoidable blocking work
- [ ] All `tokio::spawn` handles are tracked (`JoinSet`, stored, or awaited)
- [ ] `Send` bounds correct for spawned tasks
- [ ] Timeouts on every external I/O (`tokio::time::timeout`)
- [ ] Cancellation/shutdown handled (`select!` with `CancellationToken` or channel close)
- [ ] No `.await` inside a hot loop that should be `rayon` parallel
- [ ] No `async` functions that do no awaiting (just make them sync)

### 7.5 Error handling

- [ ] No `Box<dyn Error>` in public library APIs
- [ ] No `Result<T, String>` in non-main code
- [ ] Typed error enums with meaningful variants, `#[non_exhaustive]` if public
- [ ] `From` conversions at layer boundaries (domain never surfaces `sqlx::Error`)
- [ ] No `.unwrap()` / `.expect()` in production paths (tests/init OK)
- [ ] `anyhow::Context` / `.map_err` at crate boundaries for useful messages
- [ ] `# Errors` docs section on public `Result`-returning functions
- [ ] `panic!` / `unreachable!` only for true impossible states

### 7.6 Unsafe and FFI

- [ ] `// SAFETY:` comment on every `unsafe { }` block, explaining invariants
- [ ] Unsafe surface is minimal; safe wrapper around it
- [ ] `catch_unwind` at Rust-to-C boundary
- [ ] `#[repr(C)]` on types crossing FFI
- [ ] `CString` / `CStr` used correctly (null-termination, lifetime)
- [ ] Miri run on unsafe-heavy code in CI
- [ ] Sanitizers (ASan/TSan) run on code linking C

### 7.7 Testing

- [ ] Tests exist for the new behavior
- [ ] Unit tests assert behavior, not internal method calls
- [ ] Integration tests in `tests/` for cross-module behavior
- [ ] Async tests use `#[tokio::test]`
- [ ] Mocks (`mockall` or hand-rolled) have the same contract as production
- [ ] Property tests (`proptest`) for parsers, serializers, state machines
- [ ] Fuzz targets (`cargo-fuzz`) for anything processing untrusted input
- [ ] No `std::thread::sleep` / `tokio::time::sleep` for synchronization in tests
- [ ] No tests that depend on wall-clock time (use `tokio::time::pause()`)
- [ ] Flaky / time-sensitive tests are isolated or marked `#[ignore]` with rationale

### 7.8 Documentation

- [ ] Module-level `//!` doc on new modules
- [ ] `///` doc on every new public item
- [ ] `# Examples` in public function docs (doubles as doc test)
- [ ] `# Errors` when function returns `Result`
- [ ] `# Panics` when function can panic
- [ ] `# Safety` on `unsafe fn`
- [ ] Intra-doc links use `[\`Type\`]` / `[\`Type::method\`]` syntax
- [ ] Doc tests runnable (`#` hiding setup, `no_run` / `compile_fail` where appropriate)

### 7.9 Security

- [ ] Input validation at system boundaries (HTTP, CLI args, config files, untrusted deserialization)
- [ ] No `String::from_utf8_unchecked` on user input
- [ ] No SQL injection (parameterized queries only)
- [ ] No format-string injection (no `format!("{}", user_input)` where user_input is a format string)
- [ ] No log injection (sanitize user input before logging)
- [ ] No secrets in source, logs, or error messages (use `secrecy` crate or equivalent)
- [ ] Crypto primitives from vetted crates (`rustls`, `ring`, `RustCrypto`) — not hand-rolled
- [ ] Authentication / authorization at the right layer (middleware, not handler)
- [ ] `cargo-audit` run in CI; `cargo-deny` configured for supply chain
- [ ] `unsafe` reviewed; Miri run

### 7.10 Performance

- [ ] No N+1 queries (use joins, batch loads, `IN (...)`)
- [ ] No O(n²) in loops over user-controlled input
- [ ] No `.clone()` / `.to_owned()` in hot paths
- [ ] No allocation per-item in hot loops (reuse buffers, pre-allocate `Vec::with_capacity`)
- [ ] No `String::new()` + `push_str` in hot loops (use `write!` to pre-allocated buffer)
- [ ] No contention on a single `Mutex` — partition state, use `DashMap`, or channel
- [ ] `HashMap` hasher choice (default `SipHash` is slow; `ahash` / `fxhash` for trusted input)
- [ ] `Vec::retain` / `drain_filter` instead of `filter + collect` when modifying in place
- [ ] Serde avoided for micro-formats (bincode, postcard, or hand-rolled)

---

## 8. Debugging Playbook (tight; full detail in `debugging-playbook.md`)

### 8.1 Panic with backtrace

1. Run with `RUST_BACKTRACE=full`
2. Read the panic message — does it match a known pattern (`unwrap on None`, `index out of bounds`, `attempt to divide by zero`, `slice index starts at X but ends at Y`)?
3. Locate the source line from the backtrace
4. Form hypothesis: which input / state caused this?
5. Add `dbg!` or a test case that exercises the input
6. Fix with typed error / bounds check / saturating arithmetic

### 8.2 Memory growth / OOM

1. Run with `DHAT` or `heaptrack` to see where allocations happen
2. Check for retained data — common culprits: unbounded channels, unbounded `Vec`s, caches without eviction, connection pools without size limit, `Arc` cycles
3. Check for leaks — `cargo +nightly` with `-Zsanitizer=leak`
4. Instrument with `jemalloc` + `jemalloc-ctl` for runtime stats
5. Fix with bounded channels / LRU cache / weak refs (`Weak<T>`) / explicit drop

### 8.3 Deadlock

1. Run with `parking_lot::deadlock_detection` if using parking_lot
2. Or: `tokio-console` to see which task is blocked on which lock
3. Or: GDB/LLDB, attach, inspect thread stacks
4. Check lock ordering — is one thread A→B and another B→A?
5. Check if a `MutexGuard` is held across `.await`
6. Fix with consistent lock order / narrow lock scope / channels / `try_lock` with backoff

### 8.4 Slow async (no progress)

1. `tokio-console` to see task states — are tasks `Waiting` on a lock or a channel?
2. Check for missing `wake()` in a custom `Future`
3. Check for blocking I/O in async (synchronous file reads, CPU-heavy work without `spawn_blocking`)
4. Check for backpressure propagation — sender fast, receiver slow, bounded channel backs up
5. Fix per the identified cause

### 8.5 Flaky test

1. Run with `--test-threads=1` — does it still fail? (If no → test depends on ordering or shared state)
2. Run in a loop: `for i in 1..100; cargo test test_name; done` — how often does it fail?
3. Check for wall-clock dependencies (`SystemTime`), random seeds without fixing, shared global state, unordered iteration
4. Check for race between teardown and assertions
5. Fix with `tokio::time::pause()` for time control, fixed seeds, isolated state per test

### 8.6 Miri-reported UB

1. Read the Miri output — which unsafe block / operation?
2. Common patterns:
   - Aliasing violation (two `&mut` to overlapping memory)
   - Out-of-bounds pointer arithmetic
   - Dangling pointer after the referent was dropped
   - Data race (use TSan for threaded)
3. Fix by restructuring the unsafe, using `UnsafeCell`, or using a safe alternative

### 8.7 Compile error (E0XXX)

1. Read `rustc --explain E0XXX` first — it's almost always enough
2. Common borrow/lifetime errors: E0502 (mutable + immutable), E0597 (doesn't live long enough), E0521 (lifetime bound), E0277 (trait bound)
3. Use rust-analyzer's "Apply suggestion" where it's present
4. For lifetime errors: usually the fix is to restructure ownership, not add `'static`

---

## 9. Profiling Playbook (tight; full detail in `profiling-playbook.md`)

### 9.1 Quick wins

- **`cargo build --release` before profiling.** Debug builds are 10-100x slower and mislead about where time goes.
- **Fix one thing at a time.** Measure → change → measure. Otherwise you don't know which change helped.
- **Profile in realistic conditions.** Cold cache vs warm cache, single-threaded vs contended, real data size vs 10 items.

### 9.2 CPU profiling

```sh
# flamegraph (easiest, cross-platform via inferno)
cargo install flamegraph
cargo flamegraph --bin my-app -- --args

# samply (cross-platform, no root needed)
cargo install samply
samply record ./target/release/my-app --args

# perf (Linux, most powerful)
cargo build --release
perf record -g ./target/release/my-app --args
perf report
```

### 9.3 Micro-benchmark

```rust
// criterion.rs (standard for Rust)
// Cargo.toml:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
// [[bench]]
// name = "my_bench"
// harness = false

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    let input = std::fs::read_to_string("data.json").unwrap();
    c.bench_function("parse_json", |b| {
        b.iter(|| serde_json::from_str::<MyStruct>(black_box(&input)).unwrap())
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

### 9.4 Heap profiling

```sh
# DHAT (Valgrind) — detailed heap analysis
valgrind --tool=dhat ./target/release/my-app

# heaptrack (Linux, GUI)
heaptrack ./target/release/my-app
heaptrack_gui heaptrack.my-app.NNNN.zst

# In-process (dhat-rs for selective tracking)
# Cargo.toml: dhat = "0.3"
// in main:
let _profiler = dhat::Profiler::new_heap();
```

### 9.5 Async runtime profiling

```rust
// Cargo.toml:
// tokio = { version = "1", features = ["full", "tracing"] }
// console-subscriber = "0.4"

// in main:
console_subscriber::init();  // connect with `tokio-console` CLI

// Run: tokio-console http://127.0.0.1:6669
// Shows: task states, lock wait times, channel fill levels, task CPU usage
```

### 9.6 Compile-time

```sh
# Identify slow-to-compile crates
cargo build --timings
open target/cargo-timings/cargo-timing-*.html

# Find expensive functions (monomorphization cost)
cargo install cargo-llvm-lines
cargo llvm-lines --release

# Find big dependencies
cargo install cargo-bloat
cargo bloat --release --crates
```

---

## 10. Common Performance Pitfalls (catalog — full treatment in `performance-catalog.md`)

| Symptom | Root cause | Fix |
|---|---|---|
| `.clone()` in hot path | Ownership not planned; borrowing would work | Take `&T`, `&str`, `&[T]`; lifetime elision often sufficient |
| `Vec::push` in loop, resizes multiple times | No pre-allocation | `Vec::with_capacity(expected_size)` |
| `String::new()` + `push_str` in loop | Many allocations | `String::with_capacity` or `format!` + pre-size |
| `HashMap::insert` dominates flamegraph | SipHash (default) is slow for trusted input | `ahash::AHashMap` or `fxhash` |
| `collect` + `iter` chain back-to-back | Materializes intermediate `Vec` | Chain the iterator directly (don't `collect`) |
| `sort` in a loop | O(n log n) × iterations | Sort once, use `BinaryHeap`, or `BTreeMap` |
| `Mutex` contention visible in `tokio-console` | Lock scope too wide, or wrong primitive | Narrow scope; split state; use `DashMap` / channels |
| `Arc<Mutex<HashMap>>` | Contention across async tasks | `DashMap` (sharded lock) or `moka` (async cache) |
| N+1 database queries | Loop with query inside | Join / batch `IN (...)` / preload related |
| `serde_json` in hot loop | JSON parse is expensive | Use `serde_json::from_slice`; or switch to `simd-json`; or use binary format (`bincode`, `postcard`) |
| `async` function not doing any await | Unnecessary async overhead | Make it `fn`, not `async fn` |
| `Box<dyn Trait>` in hot call | vtable lookup per call | Monomorphize (generics) if one impl per target |
| `tokio::spawn` per-request with state | Per-task allocation + scheduler overhead | Pool workers; channel-based dispatch |
| Debug build in benchmarks | 10-100x slower | `cargo bench` or `--release` explicitly |
| `Regex::new` per call | Compiling regex is slow | `lazy_static!` / `LazyLock` for regex |
| `println!` in hot path | Locks stdout | `BufWriter<Stdout>` or `tracing::info!` |

---

## 11. Refactor Templates (tight; full treatment in `refactor-templates.md`)

### 11.1 `Arc<Mutex<T>>` → channel

**Before:**
```rust
let state = Arc::new(Mutex::new(State::new()));
for item in inputs {
    let s = state.clone();
    tokio::spawn(async move {
        let mut g = s.lock().unwrap();
        g.update(item);
    });
}
```

**After:**
```rust
let (tx, mut rx) = tokio::sync::mpsc::channel(100);
tokio::spawn(async move {
    let mut state = State::new();
    while let Some(item) = rx.recv().await {
        state.update(item);
    }
});
for item in inputs { tx.send(item).await.unwrap(); }
```

### 11.2 `Box<dyn Trait>` → enum dispatch

**Before:**
```rust
let formatters: Vec<Box<dyn Formatter>> = vec![
    Box::new(JsonFormatter),
    Box::new(YamlFormatter),
];
```

**After:**
```rust
enum Formatter { Json(JsonFormatter), Yaml(YamlFormatter) }
impl Formatter {
    fn format(&self, v: &Value) -> String {
        match self { Self::Json(f) => f.format(v), Self::Yaml(f) => f.format(v) }
    }
}
let formatters: Vec<Formatter> = vec![
    Formatter::Json(JsonFormatter),
    Formatter::Yaml(YamlFormatter),
];
```

### 11.3 `String` arg → `&str` / `impl AsRef<str>`

**Before:**
```rust
fn greet(name: String) -> String { format!("Hello, {}", name) }
```

**After:**
```rust
fn greet(name: impl AsRef<str>) -> String { format!("Hello, {}", name.as_ref()) }
// Callers: greet("world"); greet(&s); greet(s.clone());  — all work
```

### 11.4 Nested `match` → `?` chain

**Before:**
```rust
let user = match get_user(id) {
    Ok(u) => u,
    Err(e) => return Err(e),
};
let order = match get_order(user.order_id) {
    Ok(o) => o,
    Err(e) => return Err(e),
};
process(order)
```

**After:**
```rust
let user = get_user(id)?;
let order = get_order(user.order_id)?;
process(order)
```

### 11.5 `.unwrap()` → typed `Result`

**Before:**
```rust
fn parse_age(s: &str) -> u32 {
    s.parse().unwrap()  // Panics on bad input
}
```

**After:**
```rust
fn parse_age(s: &str) -> Result<u32, ParseIntError> {
    s.parse()
}
// Or if domain: define `struct Age(u32)` with `fn parse(s: &str) -> Result<Age, AgeError>`
```

### 11.6 Blocking in async → `spawn_blocking`

**Before:**
```rust
async fn read_config() -> Config {
    let s = std::fs::read_to_string("config.toml").unwrap();  // Blocks runtime
    toml::from_str(&s).unwrap()
}
```

**After:**
```rust
async fn read_config() -> anyhow::Result<Config> {
    // Option 1: async file API
    let s = tokio::fs::read_to_string("config.toml").await?;
    Ok(toml::from_str(&s)?)
    // Option 2: spawn_blocking if no async alternative
    // tokio::task::spawn_blocking(|| { let s = std::fs::read_to_string(...)?; ... }).await?
}
```

---

## 12. Comment Style

Good review comments are specific, actionable, and linked to the "why." Bad ones are vague, prescriptive without rationale, or stylistic-only.

### Good

> **block**: `MutexGuard` held across `.await` on line 47 — will deadlock under contention. See rust-implementing §16 "Concurrency Primitive". Suggest: clone the needed data out, drop the guard, then `.await`:
> ```rust
> let value = {
>     let guard = state.lock().unwrap();
>     guard.value.clone()
> };
> do_something(value).await;
> ```

> **request-change**: Public function `parse_config` returns `Box<dyn Error>` — callers can't match on error kinds. Define a typed error:
> ```rust
> #[derive(thiserror::Error, Debug)]
> #[non_exhaustive]
> pub enum ConfigError {
>     #[error("io: {0}")]
>     Io(#[from] std::io::Error),
>     #[error("parse: {0}")]
>     Parse(#[from] toml::de::Error),
> }
> ```

> **suggest**: `if let Some(x) = opt { use(x) } else { return Err(...) }` on line 102 reads better as `let Some(x) = opt else { return Err(...) };` (let-else, since Rust 1.65).

### Bad

> `Box<dyn Error>` is wrong.   — *vague, no suggested fix*
>
> Use `thiserror`.              — *prescriptive without rationale or example*
>
> This should be refactored.    — *nitpick without specifics*

### Structure

Every block/request-change comment should have:
1. **Severity marker** ([block], [request-change], [suggest])
2. **Specific location** (line, function)
3. **What's wrong** (one sentence)
4. **Why it matters** (link to skill or example)
5. **Suggested fix** (code, even partial)

---

## 13. Related Skills

- **[rust-planning](../rust-planning/SKILL.md)** — Architectural planning: project layout, crate boundaries, trait placement, error strategy, async strategy, test strategy, resilience. Load when you need to decide *what should be built*.
- **[rust-implementing](../rust-implementing/SKILL.md)** — The moment of writing code. Decision tables for constructs, idiomatic templates, BAD/GOOD anti-patterns Claude commonly produces, TDD. Load when writing or explaining how code SHOULD be written.
- **[rust-nif](../rust-nif/SKILL.md)** — Rust NIFs with Rustler for Elixir/BEAM integration. Load for NIF-specific review / debug.
- **[c-programming](../c-programming/SKILL.md)** — C for embedded/FFI. Load when reviewing FFI boundaries.
- **[skill-authoring](../skill-authoring/SKILL.md)** — For extending or authoring skills.

---

## 14. References

**Debugging:**
- [The Rust Book — Debugging](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [Rust LLDB tutorial](https://michaelwoerister.github.io/2015/03/27/rust-xxdb.html)
- [Rust GDB tutorial](https://sourceware.org/gdb/wiki/Rust)
- [Tokio Console](https://github.com/tokio-rs/console)

**Profiling:**
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [flamegraph (cargo-flamegraph)](https://github.com/flamegraph-rs/flamegraph)
- [samply](https://github.com/mstange/samply)
- [criterion.rs](https://bheisler.github.io/criterion.rs/book/)
- [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat)

**UB detection:**
- [Miri](https://github.com/rust-lang/miri)
- [Sanitizers](https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html)
- [loom](https://docs.rs/loom/)

**Security:**
- [Rust Security Advisory Database](https://rustsec.org/)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/)
- [cargo-vet](https://mozilla.github.io/cargo-vet/)
