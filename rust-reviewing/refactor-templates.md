# Refactor Templates

Common before/after Rust refactoring patterns. The SKILL.md hub (§11) has the tight version; this file has more patterns with detailed walkthroughs.

For reviewing, see [SKILL.md](SKILL.md). For anti-patterns to flag, see [anti-patterns-catalog.md](anti-patterns-catalog.md).

## How to use this file

When you want to propose a refactor in a review, find the closest pattern below and point to it. Quote the before/after in the review comment. Each template has: symptom, motivation, transformation.

## Contents

1. `Arc<Mutex<T>>` → channel
2. `Box<dyn Trait>` → enum dispatch
3. `String` arg → `&str` / `impl AsRef<str>`
4. `Vec<T>` arg → iterator / `&[T]`
5. Nested `match` → `?` chain
6. `.unwrap()` → typed `Result`
7. Blocking in async → `spawn_blocking` / async alternative
8. Loop with `if let` else continue → `let-else`
9. Trait with many methods → split traits
10. Feature flag in domain → composition-root swap
11. One giant `AppState` → per-subsystem state
12. `panic!` in library → typed error variant
13. `#[cfg(test)]` helper branch → test-only trait impl
14. Global `LazyLock<Mutex<T>>` → injection
15. `HashMap<String, V>` → typed key enum
16. Manual polling → `select!`

---

## 1. `Arc<Mutex<T>>` → channel

**Symptom:** Many tasks mutating shared state; lock contention or cognitive complexity.

**Motivation:** Channels give single-writer discipline. State is owned by the receiver's task; writers send messages. No lock, no contention, clear ownership.

```rust
// BEFORE
let state = Arc::new(Mutex::new(State::new()));
for item in inputs {
    let s = state.clone();
    tokio::spawn(async move {
        let mut guard = s.lock().unwrap();
        guard.update(item);
    });
}

// AFTER
let (tx, mut rx) = tokio::sync::mpsc::channel(100);
tokio::spawn(async move {
    let mut state = State::new();
    while let Some(item) = rx.recv().await {
        state.update(item);
    }
});
for item in inputs {
    tx.send(item).await.unwrap();
}
```

**When NOT to apply:** Single short critical section, read-heavy workload — Mutex/RwLock may be simpler.

---

## 2. `Box<dyn Trait>` → enum dispatch

**Symptom:** Virtual call per iteration, in a hot path, with a fixed set of implementations.

**Motivation:** Enum dispatch avoids vtable lookup and enables monomorphization + inlining. Also exhaustive matching catches new variants at compile time.

```rust
// BEFORE
let formatters: Vec<Box<dyn Formatter>> = vec![
    Box::new(JsonFormatter),
    Box::new(YamlFormatter),
    Box::new(XmlFormatter),
];
for formatter in &formatters {
    formatter.format(&value)?;  // vtable call
}

// AFTER
enum Formatter {
    Json(JsonFormatter),
    Yaml(YamlFormatter),
    Xml(XmlFormatter),
}
impl Formatter {
    fn format(&self, v: &Value) -> Result<String> {
        match self {
            Self::Json(f) => f.format(v),
            Self::Yaml(f) => f.format(v),
            Self::Xml(f) => f.format(v),
        }
    }
}
let formatters: Vec<Formatter> = vec![
    Formatter::Json(JsonFormatter),
    Formatter::Yaml(YamlFormatter),
    Formatter::Xml(XmlFormatter),
];
```

**When NOT to apply:** Open set of impls (plugins), heterogeneous storage needed, API boundary where adding variants breaks callers.

---

## 3. `String` arg → `&str` / `impl AsRef<str>`

**Symptom:** Function takes owned `String` but doesn't store it; callers must clone/own when they could borrow.

**Motivation:** `&str` covers `&String`, `&'static str`, and substrings. `impl AsRef<str>` is even more flexible. Both avoid unnecessary allocation.

```rust
// BEFORE
fn greet(name: String) -> String { format!("Hello, {}", name) }
// Callers: greet("world".to_string()); greet(s.clone());

// AFTER
fn greet(name: impl AsRef<str>) -> String { format!("Hello, {}", name.as_ref()) }
// Callers: greet("world"); greet(&s); greet(s.clone());
```

**When NOT to apply:** Function stores the string in a struct (take `String` or `impl Into<String>`); builds an async task that must own it (`'static` needed).

---

## 4. `Vec<T>` arg → `&[T]` / iterator

**Symptom:** Function takes owned `Vec<T>` but doesn't store it; callers must clone.

**Motivation:** `&[T]` covers `&Vec<T>`, arrays, slices. `impl Iterator<Item = T>` allows lazy chains.

```rust
// BEFORE
fn sum(items: Vec<i32>) -> i32 { items.into_iter().sum() }
fn longest(strings: Vec<String>) -> Option<String> { /* ... */ }

// AFTER
fn sum(items: &[i32]) -> i32 { items.iter().sum() }
fn longest(strings: &[String]) -> Option<&str> {
    strings.iter().max_by_key(|s| s.len()).map(|s| s.as_str())
}

// For iterator composability
fn sum_iter<I: IntoIterator<Item = i32>>(items: I) -> i32 { items.into_iter().sum() }
```

---

## 5. Nested `match` → `?` chain

**Symptom:** Cascading `match` on `Result` values with identical error propagation.

```rust
// BEFORE
fn flow(id: UserId) -> Result<Summary, Error> {
    let user = match get_user(id) {
        Ok(u) => u,
        Err(e) => return Err(e),
    };
    let order = match get_order(user.order_id) {
        Ok(o) => o,
        Err(e) => return Err(e),
    };
    let summary = match summarize(&order) {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    Ok(summary)
}

// AFTER
fn flow(id: UserId) -> Result<Summary, Error> {
    let user = get_user(id)?;
    let order = get_order(user.order_id)?;
    let summary = summarize(&order)?;
    Ok(summary)
}
```

When error types differ, add `.map_err` or ensure `From` conversions are in scope.

---

## 6. `.unwrap()` → typed `Result`

**Symptom:** `.unwrap()` in non-test, non-initialization code; risk of production panic.

```rust
// BEFORE
fn parse_port(s: &str) -> u16 {
    s.parse().unwrap()
}

// AFTER
#[derive(thiserror::Error, Debug)]
#[error("invalid port: {0}")]
pub struct InvalidPort(String);

fn parse_port(s: &str) -> Result<u16, InvalidPort> {
    s.parse().map_err(|_| InvalidPort(s.to_string()))
}

// Caller uses ?
let port = parse_port(&s)?;
```

In `main()` with `anyhow::Result<()>`, `.context("parsing port")?` is fine.

---

## 7. Blocking in async → `spawn_blocking` / async alternative

**Symptom:** Tokio-console shows a task running-and-not-yielding; blocking calls in async.

```rust
// BEFORE
async fn read_config() -> Config {
    let s = std::fs::read_to_string("config.toml").unwrap();  // Blocks runtime
    toml::from_str(&s).unwrap()
}

// AFTER — async alternative
async fn read_config() -> anyhow::Result<Config> {
    let s = tokio::fs::read_to_string("config.toml").await?;
    Ok(toml::from_str(&s)?)
}

// ALTERNATIVE — spawn_blocking if no async version exists (CPU-heavy work)
async fn compute() -> u64 {
    tokio::task::spawn_blocking(|| {
        expensive_sync_computation()
    }).await.unwrap()
}
```

---

## 8. `if let ... else` → `let-else`

**Symptom:** `if let Some(x) = opt { ... use x ... } else { return Err(...) }` pattern.

```rust
// BEFORE
fn use_it(opt: Option<Thing>) -> Result<Summary, Error> {
    if let Some(x) = opt {
        Ok(summarize(x))
    } else {
        Err(Error::Missing)
    }
}

// AFTER
fn use_it(opt: Option<Thing>) -> Result<Summary, Error> {
    let Some(x) = opt else { return Err(Error::Missing); };
    Ok(summarize(x))
}
```

`let-else` flattens code without indentation cost.

---

## 9. Trait with many methods → split traits

**Symptom:** Trait has 10+ methods; any client depends on the whole trait; mock is 500 lines.

```rust
// BEFORE
trait Repository {
    async fn find(&self, id: Id) -> Result<Entity, Error>;
    async fn save(&self, e: &Entity) -> Result<(), Error>;
    async fn delete(&self, id: Id) -> Result<(), Error>;
    async fn search(&self, q: &str) -> Result<Vec<Entity>, Error>;
    async fn count(&self) -> Result<usize, Error>;
    async fn export_csv(&self) -> Result<String, Error>;
    // ... 5 more
}

// AFTER — split by capability
trait Find<T> { async fn find(&self, id: Id) -> Result<Option<T>, Error>; }
trait Save<T> { async fn save(&self, e: &T) -> Result<(), Error>; }
trait Delete { async fn delete(&self, id: Id) -> Result<(), Error>; }
trait Search<T> { async fn search(&self, q: &str) -> Result<Vec<T>, Error>; }

// Use case only requires what it uses
async fn get_order(
    repo: &(impl Find<Order> + Send + Sync),
    id: OrderId,
) -> Result<Order, Error> {
    repo.find(id.into()).await?.ok_or(Error::NotFound)
}
```

Implementations can impl multiple traits.

---

## 10. Feature flag in domain → composition-root swap

**Symptom:** `#[cfg(feature = "...")]` scattered through domain logic.

```rust
// BEFORE
// domain/payment.rs
pub fn charge(card: &Card, amount: Money) -> Result<PaymentId, Error> {
    #[cfg(feature = "stripe")]
    { stripe::charge(card, amount) }
    #[cfg(feature = "paypal")]
    { paypal::charge(card, amount) }
    #[cfg(not(any(feature = "stripe", feature = "paypal")))]
    { panic!("no payment processor configured") }
}

// AFTER
// domain/payment.rs — no feature gates
pub trait PaymentProcessor {
    async fn charge(&self, card: &Card, amount: Money) -> Result<PaymentId, Error>;
}

// infra/stripe.rs — feature-gated whole file
#[cfg(feature = "stripe")]
pub struct StripeProcessor { /* ... */ }
#[cfg(feature = "stripe")]
impl PaymentProcessor for StripeProcessor { /* ... */ }

// main.rs — composition root picks impl
#[cfg(feature = "stripe")]
let processor: Arc<dyn PaymentProcessor> = Arc::new(StripeProcessor::new());
#[cfg(feature = "paypal")]
let processor: Arc<dyn PaymentProcessor> = Arc::new(PayPalProcessor::new());
```

---

## 11. One giant `AppState` → per-subsystem state

**Symptom:** `AppState` struct with 20 fields, every handler takes `State<AppState>`.

```rust
// BEFORE
#[derive(Clone)]
struct AppState {
    pub db: PgPool,
    pub cache: Arc<RedisPool>,
    pub http: reqwest::Client,
    pub email: EmailClient,
    pub metrics: PrometheusHandle,
    pub config: Arc<Config>,
    // ... 15 more
}

// AFTER — per-subsystem state, injected into what needs it
#[derive(Clone)]
struct UserState {
    pub db: PgPool,
    pub cache: Arc<RedisPool>,
}

#[derive(Clone)]
struct NotificationState {
    pub email: EmailClient,
    pub config: Arc<NotificationConfig>,
}

// Router composition
let app = Router::new()
    .nest("/users", users_router().with_state(user_state))
    .nest("/notifications", notifications_router().with_state(notification_state));
```

---

## 12. `panic!` in library → typed error variant

```rust
// BEFORE
pub fn divide(a: u32, b: u32) -> u32 {
    if b == 0 { panic!("division by zero"); }
    a / b
}

// AFTER
#[derive(thiserror::Error, Debug)]
#[error("division by zero")]
pub struct DivisionByZero;

pub fn divide(a: u32, b: u32) -> Result<u32, DivisionByZero> {
    if b == 0 { return Err(DivisionByZero); }
    Ok(a / b)
}
```

Exception: invariant violations that can't reach production code (internal asserts on impossible states) are fine as `panic!` / `unreachable!`.

---

## 13. `#[cfg(test)]` helper branch → test-only trait impl

**Symptom:** Production code has `if cfg!(test) { ... } else { ... }` branches.

```rust
// BEFORE
fn current_time() -> Instant {
    if cfg!(test) {
        TEST_CLOCK.now()
    } else {
        Instant::now()
    }
}

// AFTER — inject clock
trait Clock {
    fn now(&self) -> Instant;
}

struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant { Instant::now() }
}

#[cfg(test)]
struct MockClock { now: Instant }
#[cfg(test)]
impl Clock for MockClock {
    fn now(&self) -> Instant { self.now }
}

// Production code takes `clock: &dyn Clock`
fn process(clock: &dyn Clock, /* ... */) { /* ... */ }
```

---

## 14. Global `LazyLock<Mutex<T>>` → injection

**Symptom:** Global mutable state for a service; hard to test with different instances.

```rust
// BEFORE
use std::sync::{LazyLock, Mutex};
static CACHE: LazyLock<Mutex<HashMap<Key, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn lookup(k: &Key) -> Option<Value> {
    CACHE.lock().unwrap().get(k).cloned()
}

// AFTER — inject
pub struct Cache { inner: Arc<Mutex<HashMap<Key, Value>>> }
impl Cache {
    pub fn new() -> Self { Self { inner: Default::default() } }
    pub fn lookup(&self, k: &Key) -> Option<Value> {
        self.inner.lock().unwrap().get(k).cloned()
    }
}

// In main()
let cache = Cache::new();
let service = MyService::new(cache.clone());
```

`LazyLock` is still fine for **immutable** global state (config snapshot, parsed regex, lookup tables).

---

## 15. `HashMap<String, V>` → typed key enum

**Symptom:** Keys are a small, known set; typos at compile time would be valuable.

```rust
// BEFORE
let mut counts: HashMap<String, u32> = HashMap::new();
*counts.entry("success".to_string()).or_insert(0) += 1;
// Typo: counts.entry("sucess"...) — silent bug

// AFTER
#[derive(Hash, PartialEq, Eq, Copy, Clone)]
enum Status { Success, Failure, Timeout }

let mut counts: HashMap<Status, u32> = HashMap::new();
*counts.entry(Status::Success).or_insert(0) += 1;
// counts.entry(Status::Sucess) — compile error
```

---

## 16. Manual polling → `select!`

**Symptom:** Hand-rolled future polling; hard to reason about correctness.

```rust
// BEFORE — complex manual polling
async fn run() {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        if let Ok(msg) = rx.try_recv() {
            handle(msg);
        }
        if shutdown.load(Ordering::Acquire) { break; }
    }
}

// AFTER — clear intent with select!
async fn run() {
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(m) => handle(m),
                    None => break,    // Channel closed
                }
            }
            _ = shutdown.cancelled() => break,
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                heartbeat();
            }
        }
    }
}
```

---

## Phase separation (escape `&self`/`&mut self` collisions)

**Symptom**: borrow checker rejects with *"cannot borrow as immutable
because it is also borrowed as mutable"* inside one method, and the
tempting fix is `RefCell` to silence it.

**Before**:

```rust
fn evaluate(&mut self, input: &str) -> Result<T, E> {
    let intermediate = self.parse(input)?;
    for x in &intermediate {
        if needs_lookup(x) {
            let found = self.lookup(x)?;          // &self
            self.cache.insert(x.clone(), found);   // &mut self — CONFLICT
        }
    }
    self.process(intermediate)
}
```

**After** — gather the side-effect data in the immutable phase, apply
in the mutable phase:

```rust
fn evaluate(&mut self, input: &str) -> Result<T, E> {
    let intermediate = self.parse(input)?;
    let mut to_cache = Vec::new();
    let resolved: Vec<_> = intermediate.into_iter().map(|x| {
        if needs_lookup(&x) {
            let found = self.lookup(&x)?;
            to_cache.push((x.clone(), found.clone()));
            Ok(replaced_with(found))
        } else { Ok(x) }
    }).collect::<Result<_, _>>()?;                  // &self phase ends
    for (k, v) in to_cache { self.cache.insert(k, v); }   // &mut self phase
    self.process(resolved)
}
```

Or split into two methods, one `&self`, one `&mut self`, with the
holder calling them in sequence.

**Production exemplar**: `rust-analyzer` enforces this at crate level —
`hir-expand` (macro expansion) → `hir-def` (lowering + name resolution)
→ `hir-ty` (type inference) are separate crates. See
[rust-implementing/language-patterns.md §"Phase separation"](../rust-implementing/language-patterns.md)
for the full pattern with citations.

---

## Heavy struct → independent Arc fields

**Symptom**: methods that need to mutate two unrelated fields in
parallel won't compile because `&mut self.a` and `&mut self.b` both
implicitly hold `&mut self`. The struct is one borrow unit.

**Before** — `publish_and_record` can't borrow `messages` and
`metrics` mutably at the same time:

```rust
pub struct Broker {
    messages: HashMap<String, Vec<Event>>,
    metrics:  HashMap<String, u64>,
    configs:  HashMap<String, Config>,
}
```

**After** — each logically-independent field gets its own lock:

```rust
pub struct Broker {
    storage: Arc<Mutex<Storage>>,
    metrics: Arc<Metrics>,            // lock-free internally if possible
    configs: Arc<RwLock<Configs>>,    // read-heavy
}

impl Broker {
    pub fn publish_and_record(&self, msg: Message) -> Result<u64> {
        let offset = self.storage.lock().unwrap().append(msg.clone())?;
        self.metrics.record_message(msg.value.len());   // independent access
        Ok(offset)
    }
}
```

**Lock primitive choice**:

| Use | When |
|---|---|
| `std::sync::Mutex<T>` | Default. Any access (read or write) is exclusive. |
| `parking_lot::Mutex<T>` | Existing dep; want unpoisoned locks; benchmarks show contention. |
| `std::sync::RwLock<T>` | Reads dominate writes by ≥10× (config, route tables). |
| `Atomic*` | Scalar-sized state. Lock-free but only for primitives. |
| `dashmap::DashMap<K, V>` | Concurrent map — sharded internally. |

**Reference**: [rust-unofficial/patterns *Compose Structs*](https://rust-unofficial.github.io/patterns/patterns/structural/compose-structs.html)
— *"this design pattern lets you work around limitations in the borrow
checker. And it often produces a better design."*

**Production exemplar**: [Apache Iggy `ProducerCore`](https://github.com/apache/iggy/blob/master/core/sdk/src/clients/producer.rs)
has ~24 fields, each individually wrapped — never a single
`Mutex<Producer>` over the whole thing. See also rust-planning
architecture-patterns.md §"Compose Structs for Independent Borrowing".

**Trade-off**: a small heap allocation per `Arc`, plus lock
acquisition cost. Worth it when independent borrowing is required;
overkill when fields really must mutate together (one lock at the
boundary is correct in that case).

---

## Pointer → index (escape Vec-reallocation use-after-free)

**Symptom**: a struct owns a `Vec<T>` AND has a field of type
`*const T`, `*mut T`, `&T`, or `Option<&T>` referring to elements of
that `Vec`. Miri flags use-after-free; safe-Rust attempts hit
self-referential-struct compile errors.

**Before**:

```rust
struct History {
    states: Vec<State>,
    current: Option<*const State>,   // dangles after reallocation
}
```

**After**:

```rust
struct History {
    states: Vec<State>,
    position: usize,                  // or Option<usize> if "no current" is meaningful
}

impl History {
    fn current(&self) -> Option<&State> {
        self.position.checked_sub(1).and_then(|i| self.states.get(i))
    }
}
```

For graph nodes, prefer the production crate alternatives:
[`orlp/slotmap`](https://github.com/orlp/slotmap) (returns a `Key` on
insert, key stays valid across reallocation),
[`petgraph`](https://github.com/petgraph/petgraph) (`NodeIndex<Ix>`),
or [`fitzgen/generational-arena`](https://github.com/fitzgen/generational-arena)
(`Index { index, generation }` — guards against ABA).

The `std::vec::Vec` documentation states the underlying invariant:
*"Modifying the vector may cause its buffer to be reallocated, which
would also make any pointers to it invalid."*

---

## Related

- [rust-reviewing/SKILL.md §11](SKILL.md#11-refactor-templates-tight-full-treatment-in-refactor-templatesmd) — compact version
- [anti-patterns-catalog.md](anti-patterns-catalog.md) — patterns to flag in review
- [rust-implementing/SKILL.md](../rust-implementing/SKILL.md) — master "which construct?" table for idiomatic choices
