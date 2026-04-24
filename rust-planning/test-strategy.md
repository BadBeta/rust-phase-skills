# Test Strategy (First-Class)

Planning-phase decisions for testing: test pyramid, mocking strategy, property-based testing scope, fuzzing scope, snapshot testing scope, CI strategy, coverage goals, and using tests to drive API design.

For implementation-side code (writing `#[test]`s, mockall macros, proptest strategies, insta snapshots, cargo-fuzz targets, TDD templates), see [rust-implementing/testing-patterns.md](../rust-implementing/testing-patterns.md). For test-quality review, see [rust-reviewing/test-quality-review.md](../rust-reviewing/test-quality-review.md).

**Testing is a planning concern, not an afterthought.** Design for testability BEFORE writing the first line of production code. If you can't test the business rule without a database, introduce a trait at the boundary. If you can't mock the HTTP client, the dependency isn't inverted.

## Decision 1 — The test pyramid for Rust

| Level | Scope | Speed | Count | Tool |
|---|---|---|---|---|
| Unit | One module, no I/O | μs-ms | 1000s | `cargo test`, `mockall` for trait mocks |
| Integration | Across modules, possibly real deps | ms-s | 100s | `tests/*.rs`, `#[sqlx::test]`, `wiremock`, `testcontainers` |
| E2E | Real HTTP server + real DB + real externals (or VCR'd) | s | 10s | `reqwest`, `assert_cmd`, Axum `TestServer` |
| Property | Generative — many cases per test | ms-s | 10s of fns, 1000s of cases | `proptest`, `quickcheck` |
| Fuzz | Untrusted input, long campaigns | mins-hours | A few fuzz targets | `cargo-fuzz` (libFuzzer), `afl.rs` |
| Compile-fail | Verify certain patterns don't compile | ms | Per invariant | `trybuild` |
| Concurrency | Model-check lock-free code | s-mins | Per primitive | `loom` |
| Snapshot | Complex stable output | ms | Per output shape | `insta` |
| Doc | Runnable examples in `///` comments | ms | Per public function | `cargo test --doc` |

**Investment at planning time:** decide which layers each crate needs. Domain crate: unit + property (many). API crate: integration + E2E + unit. Parsers: fuzz + property + unit. Serializers: property + snapshot + unit.

## Decision 2 — Trait-first design

For every external boundary — and any internal piece you want to unit-test in isolation — **define the trait first, then the implementation.** This is the foundation of testable Rust.

```rust
// 1. Trait is the test surface
trait OrderRepository {
    async fn save(&self, order: &Order) -> Result<(), RepoError>;
    async fn find(&self, id: OrderId) -> Result<Option<Order>, RepoError>;
}

// 2. Production implementation
struct PgOrderRepository { pool: PgPool }
impl OrderRepository for PgOrderRepository { /* ... */ }

// 3. Test: mock with mockall
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;
    
    mock! {
        OrderRepo {}
        #[async_trait]
        impl OrderRepository for OrderRepo {
            async fn save(&self, order: &Order) -> Result<(), RepoError>;
            async fn find(&self, id: OrderId) -> Result<Option<Order>, RepoError>;
        }
    }
    
    #[tokio::test]
    async fn places_order_when_repo_saves() {
        let mut mock = MockOrderRepo::new();
        mock.expect_save().times(1).returning(|_| Ok(()));
        let uc = PlaceOrderUseCase::new(mock);
        let result = uc.execute(/* input */).await;
        assert!(result.is_ok());
    }
}
```

## Decision 3 — TDD as a design driver

TDD in Rust:

1. Write the **call site** first — what does the use case want from its dependencies?
2. That call shape becomes the **trait**.
3. Implement a **fake** satisfying the trait.
4. Write the failing test.
5. Implement the real adapter.

The trait is designed by the caller's need, not dictated by the implementor's convenience. This is Dependency Inversion made concrete.

### The TDD cycle

```
1. RED: write a failing test that expresses the new behavior
2. GREEN: write the minimum production code to pass
3. REFACTOR: clean up duplication, names, structure (tests still green)
```

### When TDD fits

- **New behavior in a well-defined domain** (business rule, parser, state machine, calculation)
- **Bug reproduction** — the regression test IS a TDD step
- **API design** — let the test shape the public API before implementation details leak in

### When TDD doesn't fit (as well)

- **Exploratory spikes** — write a throwaway to understand the problem space, then write tests for the keeper version
- **GUI / rendering** — snapshot + visual inspection often more useful
- **Library bindings / FFI** — shape is dictated by the foreign API; tests verify not design

## Decision 4 — Mocking strategy

| Need | Tool | When |
|---|---|---|
| Trait-based mocks with expectations | **mockall** | Most common. Trait → `#[mockall::automock]` → `MockTrait::new()` in tests |
| Hand-rolled fake impl | Implement trait with in-memory state | When mockall is overkill (simple traits) |
| HTTP call mocking | **wiremock** | Testing reqwest/http clients |
| Time control | `tokio::time::pause()` + `advance()` | Async timers, scheduled work |
| DB rollback per test | `#[sqlx::test]` | Postgres-backed tests |
| Docker deps | **testcontainers** | Redis, full Postgres, Elasticsearch, etc. |
| Recorded HTTP interactions | **vcr** (record-and-replay) | External APIs in integration tests |

**Rule:** mock boundaries, not internals. Mocking a module's private function means you're testing implementation, not behavior.

## Decision 5 — Property-based testing scope

Property tests generate many inputs and verify an invariant holds for all of them. Plan `proptest` for:

- **Parsers**: parse(s) succeeds ⟹ parse(print(parse(s))) == parse(s) (round-trip)
- **Serializers**: `decode(encode(x)) == x` (round-trip)
- **State machines**: any valid sequence of transitions leaves the FSM in a valid state
- **Arithmetic on newtypes**: associativity, commutativity, identity
- **Pure functions with invariants**: sort is idempotent, reverse is involutive, etc.

```rust
proptest! {
    #[test]
    fn roundtrip(order in arb_order()) {
        let bytes = serialize(&order);
        let parsed = deserialize(&bytes).unwrap();
        prop_assert_eq!(order, parsed);
    }
}
```

### Shrinking

When proptest finds a failing case, it automatically shrinks to a minimal counterexample. Design custom strategies (`arb_order`) to generate realistic inputs.

### When NOT property test

- Pure example-based tests are clearer for simple cases
- External-dependency-heavy code (the property framework can't generate realistic DB states)
- Code with massive search space where shrinking takes too long

## Decision 6 — Fuzzing scope

Fuzzing runs for minutes-hours and finds crashes in untrusted-input handling. Plan `cargo-fuzz` for:

- **Parsers** — anything that reads a byte stream
- **Deserializers** — serde formats, network protocols
- **Input validation** — email, URL, file path, any format validator
- **Anything calling `unsafe`** with input-dependent logic

Fuzzing requires fuzz targets:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = my_parser::parse(data);  // Should not panic on any input
});
```

Run: `cargo fuzz run my_target`. Continuous fuzzing in CI is ideal; short bursts (15-60 min) on PRs catch most regressions.

### Budget

- **Libraries handling untrusted input:** MUST have fuzz targets in CI.
- **Server applications with parsers:** fuzz targets recommended.
- **Domain code:** property tests usually sufficient.

## Decision 7 — Snapshot testing scope

`insta` captures output to a file; re-running compares. Plan for:

- **CLI stdout/stderr** — the user-visible output shape
- **Error messages** — especially multi-line diagnostic output
- **Serialized representations** — JSON/YAML/TOML output for config, wire formats
- **Generated code** — macro expansion, code generator output
- **Complex Debug output** for data structures

```rust
#[test]
fn error_message() {
    let err = parse("bad input").unwrap_err();
    insta::assert_snapshot!(format!("{err:#}"));
}
```

### Discipline

- **Review snapshots in PRs.** `cargo insta review` before commit.
- **Snapshot file in version control.** Changes to snapshots signal behavior changes.
- **Fail CI on uncommitted snapshots.** `INSTA_UPDATE=no` in CI.

## Decision 8 — Compile-fail tests

`trybuild` verifies that certain patterns DON'T compile. Essential for:

- **Type-state machines** — "can't call `.send()` without `.connect()` first"
- **Sealed traits** — "external crates can't implement"
- **API guarantees** — "two borrows can't alias"

```rust
// tests/compile-fail/cant_send_before_connect.rs
fn main() {
    let conn = Connection::new();
    conn.send("hello");  //~ ERROR expected `Connected`, found `Disconnected`
}
```

## Decision 9 — Concurrency testing

- **loom** — model-checks lock-free/unsafe concurrency across possible interleavings. Expensive (exponential); use for small critical regions.
- **Sanitizers** — TSan, ASan — run on integration tests with threads.
- **Stress tests** — deliberately run with many threads and check invariants.

See [rust-planning/unsafe-strategy.md](unsafe-strategy.md) §5 for sanitizer CI setup.

## Decision 10 — Coverage goal

Pick a **target range**, not an absolute number.

- **Domain crate**: 90%+ line coverage; 100% on business rules
- **Application crate**: 80%+
- **Infrastructure adapters**: 60-80% (integration tests cover the rest)
- **API handlers**: 80%+ (thin; most logic in use cases)
- **Main/composition root**: untested (glue; verified by E2E)

**Don't chase 100%.** The last few percent usually require ugly test setup for tiny benefit. Focus on:
- Every public function called at least once
- Every error branch exercised
- Every invariant has a test

Tools:
- `cargo-llvm-cov` — recommended; uses LLVM source-based coverage
- `cargo-tarpaulin` — older alternative

## Decision 11 — CI strategy

Minimum for every Rust project:

```yaml
jobs:
  check:
    - cargo fmt --check
    - cargo clippy --all-targets -- -D warnings
    - cargo test
    - cargo doc --no-deps
  
  msrv:
    - cargo +<msrv-version> test       # Test against stated MSRV
  
  coverage:
    - cargo llvm-cov --lcov --output-path lcov.info
```

For unsafe-heavy or library crates, add:
- Miri (nightly)
- Sanitizers
- cargo-fuzz short campaigns
- cargo-deny (dependency audit)
- cargo-audit (security advisories)

## Decision 12 — Test organization

- **Unit tests** in `#[cfg(test)] mod tests { ... }` alongside source — same file, share private access
- **Integration tests** in `tests/*.rs` — separate binaries, treat your crate as external dep
- **Benchmarks** in `benches/*.rs` with `criterion`
- **Examples** in `examples/*.rs` — doubles as documentation
- **Fuzz targets** in `fuzz/fuzz_targets/*.rs`

## Decision 13 — Test-quality review

Tests themselves are code that needs review. See [rust-reviewing/test-quality-review.md](../rust-reviewing/test-quality-review.md) for what to flag in test files: flaky tests (timing, ordering, shared state), brittle tests (implementation coupling), mocks that don't match production, coverage gaps, test smells.

## Related

- [rust-implementing/testing-patterns.md](../rust-implementing/testing-patterns.md) — implementation: `#[test]`, `#[tokio::test]`, mockall, insta, proptest, cargo-fuzz, loom, trybuild, async test patterns, DB fixtures, TDD workflow with complete examples
- [rust-reviewing/test-quality-review.md](../rust-reviewing/test-quality-review.md) — reviewing tests: flaky/brittle tests, mocking mistakes, coverage assessment
- [rust-planning/SKILL.md §12](SKILL.md#12-test-strategy-first-class) — planning-rules summary
