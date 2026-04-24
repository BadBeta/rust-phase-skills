# Performance Pitfall Catalog

Common Rust performance pitfalls, organized by symptom → root cause → fix. The SKILL.md hub (§10) has the tight table; this file expands each entry with code and context.

For profiling (measuring before fixing), see [profiling-playbook.md](profiling-playbook.md).

## How to use this file

1. You profiled (right?) and found a hotspot.
2. The symptom matches one below.
3. Try the fix; re-profile to verify.

## Catalog

### 1. `.clone()` in hot path

**Symptom:** Flamegraph shows significant time in `String::clone`, `Vec::clone`, or similar.

**Root cause:** Ownership not planned; borrowing would work, but `.clone()` was added to silence the borrow checker.

**Fix:**
```rust
// BEFORE
fn process(items: Vec<String>) -> Vec<Summary> {
    items.iter().map(|item| analyze(item.clone())).collect()
}
fn analyze(s: String) -> Summary { /* reads s, returns summary */ }

// AFTER — take &str, no allocation per item
fn process(items: &[String]) -> Vec<Summary> {
    items.iter().map(|item| analyze(item)).collect()
}
fn analyze(s: &str) -> Summary { /* reads s */ }
```

If a `.clone()` is truly needed (storing in a long-lived struct), consider `Arc<T>` for shared ownership — cloning an `Arc` is one atomic increment.

### 2. `Vec::push` in loop resizes multiple times

**Symptom:** Allocations dominate; DHAT shows many small reallocations.

**Root cause:** `Vec::new()` grows 4→8→16→... during pushes, copying each time.

**Fix:**
```rust
// BEFORE
let mut v = Vec::new();
for item in items { v.push(transform(item)); }

// AFTER
let mut v = Vec::with_capacity(items.len());
for item in items { v.push(transform(item)); }

// BEST — iterator chain with collect (collect pre-sizes)
let v: Vec<_> = items.iter().map(transform).collect();
```

### 3. String concatenation in loop

**Symptom:** Flamegraph shows `String::push_str`, `String::realloc`.

**Fix:**
```rust
// BEFORE
let mut s = String::new();
for item in items { s.push_str(&format!("{item}\n")); }

// AFTER — pre-size + write!
use std::fmt::Write;
let mut s = String::with_capacity(items.len() * 20);
for item in items { write!(&mut s, "{item}\n").unwrap(); }

// OR — iterator + join
let s: String = items.iter().map(|i| format!("{i}\n")).collect();
// OR — use itertools::join
```

### 4. `HashMap::insert` dominates flamegraph

**Symptom:** Significant time in HashMap insertion or lookup.

**Root cause:** Default hasher (SipHash) is deliberately slow to resist collision attacks. For trusted input (internal keys), faster hashers give big speedups.

**Fix:**
```rust
// Use aHash (faster, DoS-safe for internal)
use ahash::AHashMap;
let mut m: AHashMap<Key, Value> = AHashMap::new();

// Or fxhash (fastest, not DoS-resistant — internal use only)
use fxhash::FxHashMap;
let mut m: FxHashMap<Key, Value> = FxHashMap::default();
```

### 5. `collect` + `iter` chain back-to-back

**Symptom:** Intermediate `Vec` allocations visible in DHAT.

**Root cause:** `collect` materializes; next `iter` re-iterates.

**Fix:**
```rust
// BEFORE — materializes intermediate Vec
let sum: u32 = items.iter()
    .map(|i| i.value)
    .collect::<Vec<_>>()   // UNNECESSARY
    .iter()
    .sum();

// AFTER — fused pipeline
let sum: u32 = items.iter().map(|i| i.value).sum();
```

### 6. Sort in a loop

**Symptom:** O(n² log n) complexity when n² was expected.

**Fix:**
```rust
// BEFORE — sorts N times
for q in queries {
    let mut items = data.clone();
    items.sort();
    search(&items, q);
}

// AFTER — sort once
let mut items = data;
items.sort();
for q in queries { search(&items, q); }

// OR — use BTreeMap / BinaryHeap for always-sorted access
```

### 7. Mutex contention in `tokio-console`

**Symptom:** Tasks spending significant time waiting on a lock.

**Root cause:** Lock scope too wide, or wrong primitive.

**Fix (narrow scope):**
```rust
// BEFORE — lock held across heavy work
let mut state = state.lock().unwrap();
let result = state.process(input); // Slow; others blocked

// AFTER — copy out, release, process, write back
let input_processed = {
    let state = state.lock().unwrap();
    state.prepare(input)
};
let result = heavy_computation(input_processed);
state.lock().unwrap().commit(result);
```

**Fix (partition state):**
```rust
// BEFORE — single lock for whole cache
struct Cache { inner: Mutex<HashMap<Key, Value>> }

// AFTER — per-shard lock; many concurrent writers to different keys
use dashmap::DashMap;
struct Cache { inner: DashMap<Key, Value> }
```

**Fix (channel instead of shared mutation):**
See [refactor-templates.md §1](refactor-templates.md).

### 8. `Arc<Mutex<HashMap>>` contention

**Symptom:** All writers serialize on a single lock.

**Fix:** `DashMap` (sharded lock), `moka` (async, lock-free read path), or actor pattern (single writer, channel).

### 9. N+1 database queries

**Symptom:** Per-request latency scales with result set size; tracing shows many queries per request.

**Fix:**
```rust
// BEFORE
let orders = sqlx::query_as!(Order, "SELECT * FROM orders").fetch_all(&pool).await?;
for order in &orders {
    let items = sqlx::query_as!(Item, "SELECT * FROM items WHERE order_id = $1", order.id)
        .fetch_all(&pool).await?;  // N+1
}

// AFTER — single JOIN
let results = sqlx::query_as!(
    OrderWithItems,
    "SELECT o.*, i.* FROM orders o LEFT JOIN items i ON i.order_id = o.id"
).fetch_all(&pool).await?;
// Group in Rust

// OR — batch fetch
let order_ids: Vec<i64> = orders.iter().map(|o| o.id).collect();
let all_items = sqlx::query_as!(
    Item,
    "SELECT * FROM items WHERE order_id = ANY($1)",
    &order_ids
).fetch_all(&pool).await?;
```

### 10. `serde_json` in hot loop

**Symptom:** JSON parsing dominates flamegraph.

**Fix options:**
1. **`serde_json::from_slice`** instead of `from_str` (avoids UTF-8 validation if bytes are trusted)
2. **`simd-json`** — SIMD-accelerated JSON, 2-3x faster
3. **Binary format** — `bincode`, `postcard`, `rkyv` — 10-100x faster; requires schema change
4. **`serde_with`** for lazy-parse of expensive fields

### 11. `async` function with no `.await`

**Symptom:** Unnecessary async overhead (future state machine, task frame allocation).

**Fix:**
```rust
// BEFORE
async fn greet(name: &str) -> String { format!("hello {name}") }

// AFTER — just sync
fn greet(name: &str) -> String { format!("hello {name}") }
```

### 12. `Box<dyn Trait>` in hot call path

**Symptom:** Dynamic dispatch per call visible in flamegraph.

**Fix:** Monomorphize with generics if only one impl per target:
```rust
// BEFORE
fn process(handler: Box<dyn Handler>) { handler.handle() }

// AFTER — monomorphized, inlined
fn process<H: Handler>(handler: H) { handler.handle() }
// Or: fn process(handler: impl Handler) { handler.handle() }
```

Alternative: enum dispatch (see [refactor-templates.md §2](refactor-templates.md)).

### 13. `tokio::spawn` per request with state setup

**Symptom:** High task spawn rate; allocation per-spawn dominates.

**Fix:** Pool workers; dispatch via channel.

```rust
// BEFORE
async fn handle_request(req: Req) {
    tokio::spawn(async move {
        let client = HttpClient::new();  // Allocated per request
        client.call(req).await
    });
}

// AFTER — pooled worker, shared client
struct Workers { tx: mpsc::Sender<Req> }
impl Workers {
    async fn new(n: usize) -> Self {
        let (tx, mut rx) = mpsc::channel::<Req>(1000);
        for _ in 0..n {
            let mut rx2 = rx.resubscribe();  // or clone tx/share receiver
            tokio::spawn(async move {
                let client = HttpClient::new();  // Allocated once per worker
                while let Some(req) = rx2.recv().await {
                    client.call(req).await;
                }
            });
        }
        Self { tx }
    }
}
```

### 14. Debug build in benchmarks

**Symptom:** Numbers wildly off from expected.

**Fix:** `cargo bench` uses release by default. When running benchmarks manually, always pass `--release`.

### 15. `Regex::new` per call

**Symptom:** Regex compile time dominates for every call.

**Fix:**
```rust
// BEFORE
fn validate(s: &str) -> bool {
    let re = regex::Regex::new(r"^\d+$").unwrap();  // Compiled per call
    re.is_match(s)
}

// AFTER
use std::sync::LazyLock;
static DIGITS: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"^\d+$").unwrap());

fn validate(s: &str) -> bool {
    DIGITS.is_match(s)
}
```

### 16. `println!` in hot path

**Symptom:** stdout lock contention; I/O dominates.

**Fix:**
```rust
// BEFORE
for item in items { println!("processing {item}"); }

// AFTER — buffered writer, locked once
use std::io::{BufWriter, Write};
let stdout = std::io::stdout();
let mut out = BufWriter::new(stdout.lock());
for item in items { writeln!(out, "processing {item}").unwrap(); }
// Or: use tracing::info! with structured logging
```

### 17. Unnecessary allocations in formatting

**Symptom:** Format + print hot paths allocate strings.

**Fix:** Use `write!` to a pre-allocated buffer instead of `format!` + print.

### 18. `Vec::retain` vs `filter + collect`

**Symptom:** Modifying Vec in place when `filter + collect` would need less reasoning.

**Fix:** Pick based on whether you need the original Vec. `retain` modifies in place (no allocation). `filter + collect` creates new Vec (one allocation). If you're allocating anyway, `collect` is clearer.

### 19. `String::new() + push_str` for known format

**Symptom:** Per-field allocation/push cost.

**Fix:** `format!` (or `write!` into pre-sized buffer) — compiler optimizes known formats.

### 20. Spurious `.to_string()` / `.to_owned()`

**Symptom:** Many small `String` allocations for temporary use.

**Fix:** `&str` throughout; only convert to `String` at the owning boundary.

### 21. `Vec<Vec<T>>` for 2D data

**Symptom:** Poor cache locality, per-row allocation.

**Fix:** Flat `Vec<T>` with computed indexing; or `ndarray` / `nalgebra` for numerical work; or `grid` crate.

### 22. Single-threaded when parallel would work

**Symptom:** CPU cores underutilized on embarrassingly parallel work.

**Fix:** `rayon::par_iter()` for CPU-bound; `JoinSet` + `tokio::spawn` for async I/O-bound.

### 23. `async-trait` overhead

**Symptom:** Trait methods heap-allocate a future per call.

**Fix:** Use native `async fn` in traits (stable since Rust 1.75) where object safety isn't needed:
```rust
// With async-trait
#[async_trait]
trait Handler {
    async fn handle(&self) -> Result<()>;  // Allocates Box<dyn Future>
}

// Native (Rust 1.75+)
trait Handler {
    fn handle(&self) -> impl std::future::Future<Output = Result<()>> + Send;
}
```

Trade-off: native `async fn` in trait isn't object-safe; need generics.

### 24. Channel per item instead of batching

**Symptom:** High channel ops rate; overhead dominates actual work.

**Fix:** Batch sends and receives:
```rust
// BEFORE
for item in items { tx.send(item).await?; }

// AFTER
tx.send(items).await?;  // Or chunked: tx.send(chunk).await? for each chunk
```

### 25. Blocking I/O in `async fn`

**Symptom:** `tokio-console` shows task Running forever; other tasks starve.

**Fix:** `tokio::task::spawn_blocking` or async equivalent. See [debugging-playbook.md §4](debugging-playbook.md#4-slow-async--no-progress).

---

## Related

- [rust-reviewing/SKILL.md §10](SKILL.md#10-common-performance-pitfalls-catalog--full-treatment-in-performance-catalogmd) — compact table of this catalog
- [profiling-playbook.md](profiling-playbook.md) — measure before optimizing
- [refactor-templates.md](refactor-templates.md) — common before/after structural fixes
- [rust-implementing/data-structures.md](../rust-implementing/data-structures.md) — criterion setup, benchmark patterns
