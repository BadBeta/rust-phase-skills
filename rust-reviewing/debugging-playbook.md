# Debugging Playbook — Deep Reference

Symptom-driven diagnostic flows for Rust bugs. The SKILL.md hub (§4, §8) has the tight version; this file has detailed workflows, tool commands, and the full decision trees.

For profiling performance problems, see [profiling-playbook.md](profiling-playbook.md).

## How to use this file

Find the symptom in the table of contents, read that section's flow, follow the tools in the order given. Stop when you've identified the root cause. Don't jump to heavy tools until the light ones rule out the obvious.

## Contents

1. Panic with backtrace
2. Out-of-memory / memory growth
3. Deadlock / no progress
4. Slow async (stalled tasks)
5. Flaky test
6. Miri UB
7. Sanitizer findings
8. Compile errors (borrow checker, lifetimes, trait bounds)
9. Cross-crate / macro errors
10. Dependency-specific bugs (common crates)

---

## 1. Panic with backtrace

### Flow

1. Run with `RUST_BACKTRACE=full cargo run` (or `RUST_LIB_BACKTRACE=1` for library-code panics).
2. Read the top of the panic output:
   - **Panic message** — identifies the class (unwrap-on-None, index OOB, overflow, assertion failure).
   - **Source line** — exact file:line from the backtrace (look past `core::panicking::...` frames).
3. Locate the source line. What value caused it?
4. Form a hypothesis about the input.
5. Add `dbg!` at the prior step or write a targeted test.
6. Fix with a typed `Result`, bounds check, saturating arithmetic, or `Option` handling.

### Common patterns

| Panic message | Meaning | Fix |
|---|---|---|
| `called Option::unwrap() on a None value` | Expected `Some`, got `None` | `ok_or(...)?` or `match`/`if let` |
| `called Result::unwrap() on an Err value` | Expected `Ok`, got `Err` | `?` operator with typed error |
| `index out of bounds: the len is X but the index is Y` | Slice/Vec indexed past end | `.get(i)` returns `Option`; check bounds first |
| `slice index starts at X but ends at Y` | Bad range slicing | Check range logic; use `.get(range)` |
| `attempt to subtract with overflow` | `a - b` where `b > a` on `u*` | `checked_sub`, `saturating_sub`, or `i*` type |
| `attempt to add with overflow` | `u*` overflow | `checked_add`, `saturating_add`, `wrapping_add` |
| `attempt to divide by zero` | `a / 0` | Check before divide or `checked_div` |
| `integer overflow` (debug only) | Arithmetic overflow | Same as above; release builds wrap silently |
| `assertion failed: X == Y` | `assert_eq!` failed | Inspect values with `dbg!`; reconstruct the invariant |

### If no backtrace

- `RUST_BACKTRACE=1` — short
- `RUST_BACKTRACE=full` — full (more informative)
- In `#[should_panic]` tests, the panic is intentional — read the #\[should_panic\] attribute
- In library code, stderr may be redirected — check log config / capture
- Panic handlers can transform panics — `std::panic::set_hook` may be active

### Converting panic to `Result`

When you find a panic site in non-test code, the fix is usually:

```rust
// BEFORE
let x = maybe.unwrap();

// AFTER (typed error)
let x = maybe.ok_or(MyError::Missing)?;

// OR (if the context is `anyhow`)
let x = maybe.ok_or_else(|| anyhow::anyhow!("expected ..."))?;
```

---

## 2. Out-of-memory / memory growth

### Flow

1. Confirm the symptom — RSS growing unboundedly? Or one-time spike to OOM?
2. Instrument with process-level metrics first (RSS over time).
3. Run with `DHAT` or `heaptrack` for allocation-pattern analysis.
4. Check common culprits in order:
   - Unbounded channels (`mpsc::unbounded_channel`)
   - Unbounded `Vec` / `HashMap` (never freeing old entries)
   - Caches without eviction (`HashMap` used as cache)
   - Connection pools without size limit
   - `Arc` cycles (rare in idiomatic Rust but possible)
   - Retained futures (long-lived `spawn` handles with `Vec` grows forever)
   - Leaked `tokio::spawn` handles where stored but never awaited

### Tools

```sh
# DHAT (Valgrind) — most detailed
valgrind --tool=dhat ./target/release/my-app

# heaptrack (Linux)
heaptrack ./target/release/my-app
heaptrack_gui heaptrack.my-app.*.zst

# In-process heap tracking (dhat-rs)
# Cargo.toml: dhat = "0.3"
fn main() {
    let _profiler = dhat::Profiler::new_heap();
    // ... your code ...
}
# Generates dhat-heap.json, view at https://nnethercote.github.io/dh_view/dh_view.html

# jemalloc runtime stats
# Cargo.toml: tikv-jemallocator = "0.5", tikv-jemalloc-ctl = "0.5"
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
# Then query tikv_jemalloc_ctl::{stats, epoch, ...}
```

### Specific patterns

**Unbounded channel:**
```rust
// BAD
let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
// Fast producer, slow consumer → rx queue grows forever

// GOOD
let (tx, rx) = tokio::sync::mpsc::channel(1000);  // Backpressure at 1000
```

**Cache without eviction:**
```rust
// BAD
lazy_static! {
    static ref CACHE: Mutex<HashMap<Key, Value>> = Mutex::new(HashMap::new());
}

// GOOD
use moka::future::Cache;
static CACHE: LazyLock<Cache<Key, Value>> = LazyLock::new(|| {
    Cache::builder()
        .max_capacity(10_000)
        .time_to_live(Duration::from_secs(300))
        .build()
});
```

**Arc cycle:**
Rare in idiomatic Rust because `Weak<T>` is usually preferred for back-references. If you have `Arc<Foo>` containing `Arc<Bar>` containing `Arc<Foo>`, neither drops.

---

## 3. Deadlock / no progress

### Flow

1. Attach a debugger or use `tokio-console` to see what tasks/threads are blocked.
2. Check lock ordering — is thread A holding L1 waiting for L2, while thread B holds L2 waiting for L1?
3. Check for `MutexGuard` held across `.await` in async code.
4. Check for `parking_lot::deadlock_detection` (if using parking_lot).
5. Check for missed `wake()` in custom `Future`.

### Tools

```sh
# Tokio tasks + lock state
# Cargo.toml: tokio = { ..., features = ["tracing"] }, console-subscriber = "0.4"
console_subscriber::init();
# Run: tokio-console

# parking_lot deadlock detector
# Cargo.toml: parking_lot = { version = "0.12", features = ["deadlock_detection"] }
std::thread::spawn(|| {
    loop {
        std::thread::sleep(Duration::from_secs(10));
        let deadlocks = parking_lot::deadlock::check_deadlock();
        if !deadlocks.is_empty() {
            eprintln!("{} deadlocks detected!", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                eprintln!("Deadlock {i}:");
                for t in threads {
                    eprintln!("  tid {}: \n{:?}", t.thread_id(), t.backtrace());
                }
            }
        }
    }
});

# GDB on a running process
gdb -p <pid>
(gdb) thread apply all bt
# Read stack traces — who's blocked on what?

# LLDB equivalent
lldb -p <pid>
(lldb) thread list
(lldb) thread backtrace all
```

### `MutexGuard` across `.await`

This is the single most common async deadlock cause.

```rust
// DEADLOCK RISK
async fn bad(state: Arc<Mutex<State>>) {
    let guard = state.lock().unwrap();
    do_async_work().await;     // If another task on same thread tries
                                // to lock, we're deadlocked
    // guard dropped here
}

// FIX: clone out, drop guard, then await
async fn good(state: Arc<Mutex<State>>) {
    let value = {
        let guard = state.lock().unwrap();
        guard.value.clone()
    };
    do_async_work(value).await;
}

// OR: use tokio::sync::Mutex if you MUST hold across await
async fn alt(state: Arc<tokio::sync::Mutex<State>>) {
    let mut guard = state.lock().await;
    do_async_work(&mut *guard).await;  // OK — tokio::sync::Mutex is yield-aware
}
```

---

## 4. Slow async / no progress

### Flow

1. `tokio-console` — see task states:
   - Many `Waiting` tasks? → check what they're waiting on (channels, locks, I/O)
   - One task in `Running` forever? → blocking in async (sync I/O, CPU-heavy work)
   - Tasks polled but not progressing? → missed `wake()` in custom Future
2. Check for blocking calls in async:
   - `std::fs::read` (sync I/O)
   - `std::thread::sleep`
   - Heavy CPU loops without `tokio::task::yield_now().await`
3. Check for backpressure propagation. Fast producer + slow consumer + unbounded channel = memory leak, not stall. But bounded channel → producer blocks → upstream stalls.
4. Check for `Notify` / `CancellationToken` that was never triggered.

### Tools

- `tokio-console` — see async state live
- `RUST_LOG=tokio=trace,runtime=trace` — firehose of runtime events
- Add `tracing::instrument` to suspect async fns; enable with `RUST_LOG=my_mod=debug`

### Common causes

- Blocking sync I/O → wrap in `tokio::task::spawn_blocking`
- Heavy loop → insert `tokio::task::yield_now().await` periodically
- Channel backpressure → bounded channel + backoff; log warnings on slow send
- Missing wake → if custom Future, audit `Context::waker()` handling

---

## 5. Flaky test

### Flow

1. Reproduce: run in a loop
   ```sh
   for i in 1..100; do cargo test test_name || break; done
   ```
2. Single-threaded:
   ```sh
   cargo test test_name -- --test-threads=1
   ```
   If this makes it pass, it's a shared-state or ordering issue.
3. Check common causes:
   - Shared global state (tests mutating static)
   - Wall-clock dependency (`SystemTime::now`) — use `tokio::time::pause()` / inject clock
   - Unseeded randomness — fix seed with `StdRng::seed_from_u64(42)`
   - Unordered iteration (`HashMap`) — use `BTreeMap` in tests
   - File-system race (tests sharing `/tmp`) — use `tempfile` per test
   - Port collision (tests binding same port) — use port 0 or a port pool
   - Database race (shared DB) — use transactions with rollback, or DB-per-test (`#[sqlx::test]`)
4. Add instrumentation and re-run to capture the failure.

### Common patterns

```rust
// BAD — race on shared state
static COUNTER: Mutex<u32> = Mutex::new(0);
#[test] fn a() { *COUNTER.lock().unwrap() += 1; }
#[test] fn b() { assert_eq!(*COUNTER.lock().unwrap(), 1); }  // FLAKY

// BAD — sleep for async behavior
#[tokio::test]
async fn timing() {
    tokio::spawn(async { tokio::time::sleep(Duration::from_millis(10)).await });
    tokio::time::sleep(Duration::from_millis(5)).await;  // FLAKY
    assert!(/* ... */);
}
// FIX: use Notify, explicit synchronization, or tokio::time::pause()

// BAD — unordered
#[test] fn map_order() {
    let m: HashMap<_, _> = [(1, "a"), (2, "b")].into();
    assert_eq!(m.keys().next(), Some(&1));  // FLAKY
}
// FIX: BTreeMap, or sort before asserting
```

---

## 6. Miri UB

Miri executes your code in an abstract machine and reports UB. Run nightly only.

```sh
rustup toolchain install nightly --component miri
cargo +nightly miri test
```

### Common findings

| Miri output (paraphrased) | Meaning | Fix |
|---|---|---|
| "trying to retag with ... which is not a parent of ..." | Aliasing violation (Stacked Borrows) | Two `&mut` to overlapping memory; restructure |
| "memory access is out of bounds" | Pointer arithmetic past allocation | Check arithmetic; use `add_bounded` or slice APIs |
| "attempting to read N bytes of uninitialized memory" | Reading uninit | Use `MaybeUninit` correctly; initialize before read |
| "dangling pointer" | Pointer to dropped memory | Restructure ownership; `Pin` if self-referential |
| "data race" | Concurrent unsynchronized access | Add sync (`Mutex`, `Atomic`), or use `loom` to verify |

Miri is slow (10-50x). Run on unit tests, especially unsafe-heavy ones. Budget CI time accordingly.

---

## 7. Sanitizer findings

Run with `RUSTFLAGS="-Zsanitizer=X"` and `cargo +nightly`:

- `address` — heap UB, use-after-free, double-free, buffer overflow
- `thread` — data races
- `leak` — memory leaks
- `memory` — uninitialized reads (requires rebuilding deps with MSan)

```sh
RUSTFLAGS="-Zsanitizer=address" RUSTDOCFLAGS="-Zsanitizer=address" \
  cargo +nightly test --target x86_64-unknown-linux-gnu
```

ASan is much faster than Miri and catches FFI memory bugs. Use ASan when linking C/C++.

TSan for multithreaded unsafe code. Loom is better for lock-free code (smaller state space, faster).

---

## 8. Compile errors

### General approach

1. Read the error code: `rustc --explain E0XXX`
2. rust-analyzer shows errors inline — often with fix suggestions
3. Start from the **first** error — later ones are usually cascades

### Common error categories

**E0502 — cannot borrow X as mutable because it's also borrowed as immutable**
```rust
let mut v = vec![1, 2, 3];
let first = &v[0];
v.push(4);  // E0502 — v is borrowed immutably (via first)
println!("{first}");
```
Fix: narrow the borrow scope; don't hold the reference across the mutation.

**E0597 — borrowed value does not live long enough**
```rust
fn f<'a>() -> &'a str {
    let s = String::from("hi");
    &s  // E0597 — s dropped at end of function
}
```
Fix: return an owned `String`, or take `&'a str` as input, or use `'static`.

**E0277 — the trait `X` is not implemented for `T`**
```rust
fn f<T>(x: T) { println!("{x:?}") }  // E0277 — T doesn't implement Debug
```
Fix: add trait bound `fn f<T: Debug>(x: T)`.

**E0502 variant — "cannot borrow as mutable in a `Fn` closure":**
Fix: use `FnMut` or `FnOnce`, or use `RefCell` if interior mutability is actually needed.

**E0382 — borrow of moved value**
```rust
let s = String::from("hi");
let t = s;        // s moved
println!("{s}");  // E0382
```
Fix: `s.clone()` (if needed), or restructure ownership, or use `&s`.

**E0521 — borrowed data escapes outside of closure**
Fix: clone the data, use `Arc`, or restructure.

**E0275 — overflow evaluating the requirement**
Infinite trait bound resolution. Usually a trait-impl cycle. Simplify bounds.

### Macro errors

- **Error inside a macro expansion** — expand manually with `cargo expand` to see what's generated.
- **"no rules expected the token `X`"** — macro_rules pattern doesn't match your input.
- Errors from proc macros often have the span pointing at the attribute — hover shows the generated code.

---

## 9. Lifetime errors

Common patterns:

- **Returning a reference from a function that owns the data** → return owned type or take `&'a` input
- **Struct holding a reference** → needs `'a` lifetime parameter
- **`'static` bounds in async** → `tokio::spawn` needs `'static`. Clone/Arc the captured data.
- **Lifetime elision confusion** → add explicit lifetimes; often reveals the issue

See rust-implementing/SKILL.md §Ownership for elision rules.

---

## 10. Dependency-specific bugs

Look up the crate's changelog and GitHub issues. Common culprits:

| Crate | Typical issues |
|---|---|
| tokio | `MutexGuard` across await; spawn requiring `Send + 'static`; runtime shutdown |
| axum | Extractor ordering (consumes body); `State` type mismatches; rejection types |
| sqlx | Compile-time query macros need `DATABASE_URL` or `sqlx prepare`; offline mode |
| serde | `rename_all` mismatches; `#[serde(flatten)]` ordering; untagged enum surprise |
| reqwest | Connection pool exhaustion on `Client::new()` per-request; rustls vs native-tls |
| clap | Derive vs builder conflicts; arg ordering rules |
| rustls | Root certificates; handshake failures need OS certificate store |

---

## Related

- [rust-reviewing/SKILL.md §8](SKILL.md#8-debugging-playbook-tight-full-detail-in-debugging-playbookmd) — tight version of this playbook
- [profiling-playbook.md](profiling-playbook.md) — for performance (not correctness) bugs
- [rust-implementing/SKILL.md](../rust-implementing/SKILL.md) — for how to write idiomatic code that avoids these issues
