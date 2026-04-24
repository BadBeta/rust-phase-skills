# Validation Against Production Code (2026-04-24)

Evidence-based check of rules, decision tables, and specific claims across the three phase skills. Methodology: search GitHub / official crate docs for real-world production usage of each claim; flag any that can't be found or are contradicted by high-quality wild code.

This file documents both **corrections applied** (in the immediate validation pass) and **findings still to monitor** (for future validation rounds).

## Baseline — inherited evidence

The `rust-reviewing/anti-patterns-catalog.md` file already contains an evidence-based review of the old `rust-programming/SKILL.md`'s 18 rules (verified against tokio, axum, hyper, serde, sqlx, ripgrep, clap, reqwest, crossbeam, rust-analyzer). Every CHALLENGED and PARTIAL rule from that review was already nuanced in the new skill family during the initial split:

- Rule 6 (thiserror) → `rust-implementing/SKILL.md` Rule 6 reframed as PREFER; notes hand-rolled `impl Display + impl Error` from tokio/axum/hyper/serde/ripgrep
- Rule 11 (clippy) → Rule 11 rewritten to avoid blanket `clippy::pedantic`; recommends curated `[workspace.lints.clippy]`
- Rule 12 (newtypes) → Rule 12 softened to PREFER
- Rule 14 (async closures) → corrected: async closures are edition-independent
- Rule 17 (`.clone()` to silence borrow checker) → nuanced: Arc clone is idiomatic
- Plus Rule 1 (unwrap) and Rule 9 (shared-state options) soft-framed

## New-authoring findings (this pass)

### Corrections applied

| Finding | Severity | Location | Fix |
|---|---|---|---|
| `async-std` wording was "declining" / "no longer actively developed" | Inaccurate | `rust-planning/SKILL.md` Rule 23, master decision table row, §10.2 table; `rust-planning/async-strategy.md` runtime table | Strengthened to **"discontinued in March 2025 (v1.13.1 final); migrate off to Tokio or smol"** (source: async-rs/async-std repo, corrode.dev async state blog, Fedora deprecation) |
| OpenTelemetry OTLP code sample used deprecated `new_pipeline().tracing().with_exporter(...)` API | Outdated API | `rust-planning/distributed-rust.md` §Observability | Replaced with current `SpanExporter::builder().with_tonic()` pattern (opentelemetry-otlp 0.28+). Added note that exact API shifts per release. |
| Claim "Axum `TestServer`" implied it was part of the `axum` crate | Misleading | `rust-planning/SKILL.md` §3.8 + subskill index, `rust-planning/test-strategy.md` pyramid, `rust-implementing/testing-patterns.md` section heading | Clarified: `TestServer` lives in the separate **`axum-test` crate**. The in-process pattern using `tower::ServiceExt::oneshot` does not require a server. |
| Decision-table row "Shared mutable, short critical sections … NOT `tokio::sync::Mutex` unless held across `.await`" | Confusingly worded | `rust-implementing/SKILL.md` master decision table (Concurrency primitive) | Reworded: "Shared mutable, short critical sections NOT held across `.await` → `std::sync::Mutex` or `parking_lot::Mutex`; NOT `tokio::sync::Mutex` (unnecessary overhead here)." |
| Custom-actor example in `async-patterns.md` used `async-std` without flagging status | Could mislead readers to adopt discontinued runtime | `rust-implementing/async-patterns.md` §Custom Actors | Prepended note: async-std discontinued; pattern shape identical across runtimes; Tokio/smol are the replacement targets. |

### Claims verified (no change needed)

| Claim | Source |
|---|---|
| `samply` cross-platform sampling profiler, releases active in 2024-2025 (0.13.x) | github.com/mstange/samply releases |
| `#[sqlx::test]` creates new DB per test, runs migrations, cleans up | docs.rs/sqlx attr.test docs |
| `moka::future::Cache` uses TinyLFU by default, async `insert`/`invalidate` | docs.rs/moka, moka-rs README |
| `tokio_util::sync::CancellationToken` + `.child_token()` — hierarchical cancellation where child cancel does NOT cancel parent | docs.rs/tokio-util, tokio.rs/tokio/topics/shutdown |
| `cargo-llvm-lines` + `cargo-bloat` — active, stable, correct descriptions | github.com/dtolnay/cargo-llvm-lines, perf-book |
| `argon2` API — `SaltString::generate(&mut OsRng)` + `Argon2::default()` + `.hash_password(pw.as_bytes(), &salt)?.to_string()` + `PasswordHash::new(hash)?` | docs.rs/argon2, RustCrypto book |
| `secrecy::Secret<T>` + `ExposeSecret` trait redacts Debug output | docs.rs/secrecy |
| `loom` model checking — run with `RUSTFLAGS="--cfg loom"`, intercepts atomics/synchronization | github.com/tokio-rs/loom, matklad 2024 post |
| Native `async fn` / RPITIT stable since Rust 1.75 (Dec 2023); not dyn-compatible | blog.rust-lang.org 2023-12-21 |
| Edition 2024 stabilized in Rust 1.85 (Feb 2025); async closures edition-independent | blog.rust-lang.org 2025-02-20 |
| `if-let` chains in Rust 2024 edition, stable in 1.88 (Jun 2025) | blog.rust-lang.org 2025-06-26 |
| `LazyLock` stable since 1.80 (Jul 2024) | blog.rust-lang.org 2024-07-25 |
| `let-else` stable since 1.65 (Nov 2022) | blog.rust-lang.org 2022-11-03 |
| `LocalSet` + `spawn_local` are the way to run `!Send` futures (e.g. with `Rc<RefCell<T>>`) | docs.rs/tokio task::LocalSet |
| axum 4x downloads vs actix-web; repo is `tokio-rs/axum` | crates.io stats |
| `#[non_exhaustive]` on public enums used by axum, tokio-postgres | axum, tokio-postgres source |
| reqwest uses `log` by default (notable holdout among the big crates); `reqwest-tracing` middleware crate bridges to tracing | crates.io/reqwest, reqwest-tracing |

### Nuances to monitor (not correcting now, worth a re-check later)

| Claim | Why to watch |
|---|---|
| "axum ~4x actix-web downloads" | Download ratios drift; verify annually. |
| "`impl Into<String>` / `impl AsRef<str>` more common than raw `&str` in public APIs" | Context-dependent; remains true for major libraries (clap, axum). |
| `secrecy::Secret<T>` recommendation | Crate v0.10+ added `SecretBox`, `SecretString`, `SecretVec` type aliases. Examples in `security-audit.md` still use the `Secret<String>` form, which works but isn't the newest idiom. |
| OpenTelemetry crate surface evolves each release | Pin versions in consumer code; re-verify example on next OTLP major. |
| Tool version lists in `profiling-playbook.md` | `divan`, `iai`, `pprof-rs`, and newer entrants move quickly; re-verify at major-version bumps. |
| Rule on `[workspace.lints]` (stable since 1.74) | Still correct; axum's example lint list is a living document — may shift. |

## Methodology notes for future passes

1. **Start with the master decision tables and numbered LLM rules.** They're the highest-leverage claims — wrong ones produce wrong code repeatedly.
2. **For each crate reference, verify the crate exists, is maintained, and the API signature matches.** Don't trust cached knowledge on fast-moving crates (opentelemetry, tokio-util, anything 0.x).
3. **Prefer primary sources:** docs.rs, the crate's README/changelog, official blog. Secondary sources (tutorials, blog posts) can lag.
4. **When a rule is "ALWAYS":** look for contradictions in top-tier production code. If tokio/axum/ripgrep/serde disagree, soften to "PREFER" and note both approaches.
5. **When a rule is specific to tooling:** verify the tooling is current, not deprecated, and the command flags still apply. Sanitizer, Miri, criterion, and cargo-extension tool surfaces change.
6. **Keep this file local** (do not push to a public remote) — it references specific crate versions and tooling that date quickly.
