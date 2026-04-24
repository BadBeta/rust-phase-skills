# Test Quality Review (First-Class)

Reviewing Rust test files. Tests are code — they deserve review with the same rigor as production code, but with different concerns: flakiness, brittleness, mock fidelity, coverage gaps, and test smells.

For the tight version, see [SKILL.md §7.7](SKILL.md#77-testing). For test planning strategy, see [rust-planning/test-strategy.md](../rust-planning/test-strategy.md). For writing tests, see [rust-implementing/testing-patterns.md](../rust-implementing/testing-patterns.md).

## Why test quality matters

- **Flaky tests** destroy trust in CI. People learn to re-run instead of investigating. Bad tests slip through.
- **Brittle tests** make refactoring painful. When a reasonable refactor breaks 50 tests, the team stops refactoring.
- **Mocks that don't match production** create false confidence. Tests pass; production fails.
- **Coverage gaps** hide real bugs behind a coverage-percent number.

## How to use this file

When reviewing test files, walk the checklists below. Flag issues with severity per [SKILL.md §6](SKILL.md#6-severity-classification). Most test-quality issues are **request-change** or **suggest**.

## Contents

1. Flaky test patterns
2. Brittle test patterns
3. Mock quality
4. Coverage gap patterns
5. Assertion quality
6. Async test patterns
7. Test organization
8. Property & fuzz test review
9. Snapshot test review
10. Compile-fail test review

---

## 1. Flaky test patterns

Flaky tests fail intermittently. Causes:

### 1.1 Wall-clock dependency

```rust
// BAD
#[test]
fn token_expires() {
    let token = Token::new();
    std::thread::sleep(Duration::from_secs(1));
    assert!(token.expired(Duration::from_millis(500)));  // FLAKY on slow CI
}

// GOOD — inject clock
#[test]
fn token_expires() {
    let clock = MockClock::new();
    let token = Token::new(&clock);
    clock.advance(Duration::from_secs(1));
    assert!(token.expired(Duration::from_millis(500)));
}

// GOOD (async) — tokio time control
#[tokio::test]
async fn token_expires() {
    tokio::time::pause();
    let token = Token::new();
    tokio::time::advance(Duration::from_secs(1)).await;
    assert!(token.expired(Duration::from_millis(500)));
}
```

### 1.2 Sleep for async synchronization

```rust
// BAD
#[tokio::test]
async fn eventually_processed() {
    let tx = start_processor().await;
    tx.send(item).await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;  // FLAKY
    assert_eq!(get_state(), State::Processed);
}

// GOOD — await explicit completion signal
#[tokio::test]
async fn processed() {
    let (tx, mut done) = start_processor().await;
    tx.send(item).await.unwrap();
    done.recv().await.unwrap();   // Deterministic
    assert_eq!(get_state(), State::Processed);
}
```

### 1.3 Shared global state

```rust
// BAD — all tests share COUNTER
static COUNTER: Mutex<u32> = Mutex::new(0);

#[test]
fn test_a() {
    *COUNTER.lock().unwrap() += 1;
    assert_eq!(*COUNTER.lock().unwrap(), 1);  // Fails if test_b ran first
}

// GOOD — per-test state
#[test]
fn test_a() {
    let counter = Arc::new(Mutex::new(0));
    *counter.lock().unwrap() += 1;
    assert_eq!(*counter.lock().unwrap(), 1);
}
```

### 1.4 Unseeded randomness

```rust
// BAD
#[test]
fn random_input() {
    let input: u32 = rand::random();   // Different each run
    assert!(process(input) < 100);
}

// GOOD
#[test]
fn random_input() {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    let input: u32 = rng.gen();
    assert!(process(input) < 100);
}
```

### 1.5 Unordered iteration

```rust
// BAD
#[test]
fn first_key() {
    let m: HashMap<_, _> = [(1, "a"), (2, "b")].into();
    assert_eq!(m.keys().next(), Some(&1));  // Non-deterministic order
}

// GOOD
#[test]
fn sorted_keys() {
    let m: BTreeMap<_, _> = [(1, "a"), (2, "b")].into();
    assert_eq!(m.keys().next(), Some(&1));
}
```

### 1.6 Port collision

```rust
// BAD — two tests binding port 8080 race
#[test]
fn server_a() {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    // ...
}

// GOOD — port 0 lets OS assign
#[test]
fn server_a() {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    // use `port`
}
```

### 1.7 Filesystem races

```rust
// BAD
#[test]
fn writes_file() {
    let path = "/tmp/test_output.txt";
    // Two concurrent tests overwrite
}

// GOOD — tempfile per test
#[test]
fn writes_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output.txt");
    // ...
}
```

### 1.8 Database races

```rust
// BAD — shared DB state
#[tokio::test]
async fn creates_user() {
    let pool = shared_pool().await;
    let user = User::create(&pool, "alice").await.unwrap();
    assert_eq!(user.name, "alice");
    // Next test creates "alice" → unique violation
}

// GOOD — per-test DB via sqlx
#[sqlx::test]
async fn creates_user(pool: PgPool) {
    let user = User::create(&pool, "alice").await.unwrap();
    assert_eq!(user.name, "alice");
}
```

`#[sqlx::test]` creates a fresh DB per test, runs migrations, drops after.

---

## 2. Brittle test patterns

Brittle tests break on reasonable refactors. They test *how* instead of *what*.

### 2.1 Testing implementation, not behavior

```rust
// BAD — asserts internal method call
#[test]
fn processes_order() {
    let mut mock_repo = MockRepo::new();
    mock_repo.expect_internal_helper().times(1);   // Couples to impl
    mock_repo.expect_save().times(1);
    let uc = PlaceOrderUseCase::new(mock_repo);
    uc.execute(/* input */);
}

// GOOD — asserts observable behavior
#[test]
fn processes_order() {
    let mock_repo = MockRepo::new();
    mock_repo.expect_save().times(1).returning(|_| Ok(()));
    let uc = PlaceOrderUseCase::new(mock_repo);
    let result = uc.execute(/* input */);
    assert!(result.is_ok());
}
```

### 2.2 Over-mocking

If a unit test mocks 5 trait objects, the unit is doing too much. Split the unit.

### 2.3 Testing through private reflection

If you're using `#[cfg(test)] pub(crate)` to expose internals for testing, that's a sign the test should use the public API instead.

### 2.4 Exact-string error assertions

```rust
// BAD — breaks if error message changes
#[test]
fn rejects_bad_input() {
    let err = parse("bad").unwrap_err();
    assert_eq!(err.to_string(), "invalid syntax at line 1, col 1");
}

// GOOD — match on error variant
#[test]
fn rejects_bad_input() {
    let err = parse("bad").unwrap_err();
    assert!(matches!(err, ParseError::InvalidSyntax { .. }));
}
```

For user-facing messages, consider snapshot tests (`insta`) — still captures the full message but localized in one place to update.

### 2.5 Hardcoded expected values from implementation

```rust
// BAD — copying implementation's internals
#[test]
fn hash_output() {
    let h = hash("hello");
    assert_eq!(h, 0x7a95a8b3);  // Brittle; any hash change breaks
}

// GOOD — assert a property
#[test]
fn hash_is_deterministic() {
    assert_eq!(hash("hello"), hash("hello"));
    assert_ne!(hash("hello"), hash("world"));
}
```

---

## 3. Mock quality

### 3.1 Mock contract matches production contract

Mocks must obey the same trait as production impls. If production's `save` returns `Ok(())` when succeeding, so must the mock.

### 3.2 Default behavior for unused methods

In `mockall`, if a method is called but not `.expect()`'d, it panics. Good default — catches unintended calls. If you want a no-op default, set `.returning(|_| Ok(()))` explicitly.

### 3.3 Return values match production

```rust
// BAD — mock returns data that production never would
mock.expect_find().returning(|_| {
    Ok(Some(Order { id: OrderId(0), status: OrderStatus::DeliveredButPaymentFailed }))
});
// Production code can never produce this impossible state
```

Keep mock return values within the possible-states envelope of production.

### 3.4 Call counts

- `.times(1)` — exactly one call (catches extra calls)
- `.times(0..=1)` — at most one call (less strict)
- No `.times()` — any number of calls
- `.times(0)` — must NOT be called (useful for negative assertions)

---

## 4. Coverage gaps

### 4.1 Missing error-branch tests

Look for `Result`-returning functions where only `Ok` is tested. Each `Err` variant should have a test.

```rust
fn parse_age(s: &str) -> Result<u8, ParseError> {
    let n: u32 = s.parse().map_err(|_| ParseError::NotANumber)?;
    if n > 150 { return Err(ParseError::TooLarge); }
    Ok(n as u8)
}

// BAD — only happy path tested
#[test]
fn parses() { assert_eq!(parse_age("42").unwrap(), 42); }

// GOOD — each branch
#[test] fn happy() { assert_eq!(parse_age("42").unwrap(), 42); }
#[test] fn not_number() { assert!(matches!(parse_age("abc"), Err(ParseError::NotANumber))); }
#[test] fn too_large() { assert!(matches!(parse_age("200"), Err(ParseError::TooLarge))); }
#[test] fn boundary() { assert_eq!(parse_age("150").unwrap(), 150); }
```

### 4.2 Boundary conditions missing

For numeric inputs, test: 0, 1, max, max+1 (should fail), max-1. For collections: empty, single, two, many.

### 4.3 Concurrent behavior untested

If the production code will be called from multiple tasks/threads, test concurrent access. `loom` for lock-free; integration test with N threads for others.

### 4.4 Panics / invariant violations untested

For each `panic!` / `unreachable!`, either:
- Have a test for the impossible-state (confirms it's impossible in practice)
- Have a `#[should_panic(expected = "...")]` test (confirms panic behavior)

### 4.5 Integration tests missing

Unit tests pass doesn't mean the crate as a whole works. A `tests/*.rs` integration test that uses the crate's public API verifies linking and observable behavior.

---

## 5. Assertion quality

### 5.1 Use specific assertions

```rust
// BAD
assert!(result.is_ok());  // Doesn't tell what failed

// GOOD
let value = result.expect("failed to parse");
assert_eq!(value.name, "alice");
```

### 5.2 Useful failure messages

```rust
// BAD — prints `assertion failed: x == y`
assert_eq!(actual, expected);

// GOOD — prints values on failure (assert_eq does this automatically for Debug types)
#[derive(Debug)]  // So assert_eq's default message is useful
struct Thing { /* ... */ }

// With custom context
assert_eq!(actual, expected, "for input {input:?}");
```

### 5.3 Pretty-printed diffs

`pretty_assertions` crate gives colored diff output for `assert_eq!`. For large structs, way more readable.

```toml
[dev-dependencies]
pretty_assertions = "1"
```

```rust
#[cfg(test)]
use pretty_assertions::assert_eq;  // Replace std assert_eq
```

---

## 6. Async test patterns

### 6.1 `#[tokio::test]` is the standard

```rust
#[tokio::test]
async fn processes() {
    let result = process_async().await;
    assert!(result.is_ok());
}
```

### 6.2 Time control

```rust
#[tokio::test]
async fn timeout_fires() {
    tokio::time::pause();   // Freeze time
    let fut = operation_with_timeout();
    tokio::time::advance(Duration::from_secs(30)).await;
    assert!(fut.await.is_err());
}
```

### 6.3 Deterministic concurrency

If you need to test concurrent scenarios deterministically, use `tokio::task::yield_now()` to force task interleaving points.

---

## 7. Test organization

### 7.1 Unit vs integration

- Unit tests: `#[cfg(test)] mod tests` in same file. Shares private access.
- Integration tests: `tests/foo.rs`. Sees only public API. Verifies the crate as a library.

### 7.2 Helper modules

```rust
// tests/common/mod.rs — shared across tests/*.rs
pub fn setup() -> TestApp { /* ... */ }

// tests/foo.rs
mod common;
#[test]
fn x() { let app = common::setup(); /* ... */ }
```

### 7.3 Test naming

```rust
// BAD
#[test] fn test1() {}
#[test] fn test_user() {}

// GOOD — describes behavior under test
#[test] fn creates_user_when_email_unique() {}
#[test] fn rejects_user_when_email_duplicate() {}
#[test] fn hashes_password_before_storage() {}
```

### 7.4 Arrange-Act-Assert

```rust
#[test]
fn places_order() {
    // Arrange
    let repo = MockOrderRepo::new();
    repo.expect_save().returning(|_| Ok(()));
    let uc = PlaceOrderUseCase::new(repo);
    let input = PlaceOrderInput { items: vec![...] };
    
    // Act
    let result = uc.execute(input);
    
    // Assert
    assert!(result.is_ok());
}
```

---

## 8. Property & fuzz test review

### Property tests (`proptest`)

- [ ] Properties are real invariants, not "does it not crash"
- [ ] Custom strategies (`arb_user`, `arb_order`) produce realistic data
- [ ] Shrinking behavior verified — a failure shrinks to a minimal counterexample
- [ ] Budget reasonable — default 256 cases; increase for critical code

```rust
// GOOD property
proptest! {
    #[test]
    fn roundtrip(v: MyStruct) {
        let bytes = serialize(&v);
        let parsed = deserialize(&bytes).unwrap();
        prop_assert_eq!(v, parsed);
    }
}

// WEAK property — doesn't assert what should hold
proptest! {
    #[test]
    fn doesnt_crash(v: MyStruct) {
        let _ = serialize(&v);  // Just doesn't panic
    }
}
```

### Fuzz tests (`cargo-fuzz`)

- [ ] Targets cover parsers, deserializers, format handlers, `unsafe` with input-dep logic
- [ ] Corpus seeds from real production data where possible
- [ ] Targets are minimal (one focused operation per target)
- [ ] Regression tests in `fuzz/corpus/` for past crashes

---

## 9. Snapshot test review (`insta`)

- [ ] Snapshots in version control (`.snap` files committed)
- [ ] CI fails on uncommitted snapshots (`INSTA_UPDATE=no`)
- [ ] Snapshot content is stable — no timestamps, random IDs, or pointer addresses
- [ ] Snapshots are small enough to review; large snapshots should be redacted or summarized

```rust
insta::assert_snapshot!(output);            // Stores in .snap file
insta::assert_json_snapshot!(response);     // JSON-specific formatter
insta::assert_yaml_snapshot!(config);       // YAML

// With redactions for unstable fields
insta::assert_json_snapshot!(response, { ".id" => "[UUID]", ".timestamp" => "[TIMESTAMP]" });
```

---

## 10. Compile-fail test review (`trybuild`)

- [ ] Tests that certain patterns FAIL to compile for the right reason
- [ ] Error message excerpts match specific error codes or text

```rust
// tests/compile-fail/type_state.rs
use my_crate::Connection;
fn main() {
    let conn = Connection::new();
    conn.send("hello");  //~ ERROR expected `Connected`, found `Disconnected`
}
```

```rust
// tests/trybuild.rs
#[test]
fn compile_fails() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
```

---

## Related

- [rust-reviewing/SKILL.md §7.7](SKILL.md#77-testing) — compact review checklist
- [rust-planning/test-strategy.md](../rust-planning/test-strategy.md) — planning tests (test pyramid, mocking, property/fuzz scope)
- [rust-implementing/testing-patterns.md](../rust-implementing/testing-patterns.md) — writing tests (cargo test, mockall, insta, proptest, cargo-fuzz, TDD)
