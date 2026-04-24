# Async Strategy

Planning-phase decisions for async code: sync vs async, runtime choice, task topology, actor vs channels, graceful shutdown design, when NOT to add channels.

For the implementation-side code patterns (channel primitives, `select!`, `JoinSet`, Tower service, structured concurrency templates), see [rust-implementing/async-patterns.md](../rust-implementing/async-patterns.md). For the planning rules summary, see [rust-planning/SKILL.md §10 Async Strategy](SKILL.md#10-async-strategy) and the inter-component communication decision table in §8.

## Decision 1 — Should this be async at all?

Async Rust costs ergonomic complexity (lifetimes, `Send` bounds, pin projection, runtime selection, fragmented ecosystem). The benefit is concurrent I/O at scale.

| Case | Async? |
|---|---|
| Web server handling many concurrent HTTP requests | **Yes** |
| Database-backed service with many connections | **Yes** |
| Streaming processor with I/O overlap | **Yes** |
| Many long-lived independent tasks (actor-like) | **Yes** |
| NIF / embedded runtime where host is async | **Yes** |
| CLI tool doing one thing | **No** — sync |
| Library with no I/O | **No** — sync |
| Library for sync consumers | **No** — sync (or dual API) |
| Pure computation | **No** — sync |
| Code that will be called from sync context | **Caution** — mixing requires `Runtime::block_on`, awkward |

**Rule of thumb:** if you're doing ≤3 concurrent I/O operations, sync + threads (or blocking I/O) is usually simpler and within perf budget. Async starts paying off at hundreds of concurrent operations.

## Decision 2 — Which runtime?

One runtime per binary. Don't mix. Tokio + async-std in the same process have incompatible reactors.

| Runtime | When to choose | Trade-offs |
|---|---|---|
| **Tokio** (default for headless) | Web services, databases, most async server code | Huge ecosystem; `axum`, `sqlx`, `reqwest`, `tonic` integrate natively. Slight binary size / compile-time overhead. |
| `smol` | Small/embedded, minimal deps, want `async-std`-like API | Smaller footprint, less mature ecosystem. **Upstream recommendation for projects moving off `async-std`.** |
| `async-std` | **Discontinued** as of March 2025 (v1.13.1 was the final release) | Do not use for new projects. Migrate existing code to Tokio or smol. |
| `monoio` / `glommio` | io_uring, single-threaded-per-core, Linux-only HPC | Highest performance for specific workloads; smaller ecosystem |
| `embassy` | Embedded, `no_std`, bare-metal MCU (Cortex-M, RISC-V, ESP32 via esp-hal, RP2040/RP2350, nRF, STM32) | Async-first embedded framework. Architecture selected via `platform-*` features. Chip selected via mutually-exclusive features (e.g., `rp2040` vs `rp235xa`). Async peripherals are THE API — no sync/async toggle. Ecosystem: `critical-section` (concurrency primitive), `portable-atomic` (atomics polyfill), `defmt` (binary logging), `heapless` (no_std collections), `static_cell` (for the executor singleton). See the `rp2040`, `rp2350`, `esp32-c` skills for chip specifics. |
| **Custom runtime integrated with a UI event loop** | GUI apps where async work must coordinate with the main thread | Zed's GPUI uses `async-task` wrapping platform primitives (GCD on macOS); egui/iced drive their own executors; Bevy uses its own task pools. Don't try to force Tokio into a GUI's main thread. |

### Tokio flavor (single-threaded vs multi-threaded)

| Flavor | When |
|---|---|
| Multi-threaded (default: `#[tokio::main]`) | Web servers, any parallelizable async workload |
| Single-threaded (`#[tokio::main(flavor = "current_thread")]`) | Low-overhead services, deterministic testing, NIFs/embedded, resource-constrained |

Multi-threaded gives work stealing and cross-core parallelism at the cost of `Send` bounds everywhere. Single-threaded lets you use `Rc`/`RefCell` freely but can't leverage multiple cores.

## Decision 3 — Task topology

Sketch the task graph before spawning. Questions:

- How many top-level tasks? (Usually 1-5: HTTP server, background worker, metrics flusher, health checker, signal handler.)
- Are they all supervised? By what mechanism?
- How do they coordinate shutdown?

Patterns:

- **Supervisor + workers**: parent task with `JoinSet`, children `set.spawn(...)`, await completion.
- **Fan-out/fan-in**: producer spawns N workers from a bounded channel; each pulls work and produces results back.
- **Per-connection tasks**: web server spawns one task per incoming connection (axum does this automatically; don't hand-roll unless you need specific behavior).
- **Actor**: a task owns some state and receives messages on a channel; other tasks send; reply via oneshot.

### Spawn discipline

**Every `tokio::spawn` must have a traced handle.** Options:

```rust
// 1. JoinSet — group of related tasks, cancellable together
let mut set = tokio::task::JoinSet::new();
for item in inputs {
    set.spawn(async move { process(item).await });
}
while let Some(result) = set.join_next().await {
    handle_result(result?)?;
}

// 2. Store JoinHandle in state for long-lived tasks
struct App {
    background: tokio::task::JoinHandle<()>,
}
impl App {
    async fn shutdown(self) {
        self.background.abort();
        let _ = self.background.await;   // Await to completion/cancellation
    }
}

// 3. Await at top level for fire-and-await patterns
let handle = tokio::spawn(async { ... });
// ... some work ...
let result = handle.await?;
```

**Never fire-and-forget.** A spawned task that panics silently swallows the error and leaks its resources forever.

## Decision 4 — Actor vs channels vs shared state

For communication between concurrent components, pick the right primitive.

| Need | Use | Why |
|---|---|---|
| Serialize writes to a single resource | GenServer-style actor: one task owns the state, others send commands | Single-writer discipline, no lock contention |
| Multiple readers, one writer | `Arc<RwLock<T>>` | Reads parallelize |
| Multiple readers, no writers | `Arc<T>` | Zero synchronization cost |
| Producer → consumer (fan-out by work) | `tokio::sync::mpsc` channel | Natural backpressure if bounded |
| Multiple consumers see every message | `tokio::sync::broadcast` | Pub/sub |
| Multiple consumers see latest state | `tokio::sync::watch` | Config reload, status |
| Request/response | `tokio::sync::oneshot` | Tiny, zero-contention reply channel |

See the decision table in [rust-planning/SKILL.md §8 Inter-Component Communication](SKILL.md#8-inter-component-communication) for the full escalation path.

## Decision 5 — Graceful shutdown

**Design upfront. Retrofitting is painful.** Shutdown pattern:

```
1. Shutdown signal (SIGTERM, Ctrl+C, internal failure)
2. Root CancellationToken cancelled
3. HTTP server stops accepting new connections
4. In-flight requests given a grace period (e.g., 30s) to complete
5. Child tasks receive cancellation via child tokens
6. Each task finishes its current unit of work, drops guards, closes connections
7. Parent awaits all children, then exits with the right code
```

### `tokio_util::sync::CancellationToken`

```rust
use tokio_util::sync::CancellationToken;

let shutdown = CancellationToken::new();

// Child task watches parent token
let child_token = shutdown.child_token();
tokio::spawn(async move {
    loop {
        tokio::select! {
            _ = child_token.cancelled() => {
                // Drain, close, exit
                break;
            }
            work = recv_work() => {
                process(work).await;
            }
        }
    }
});

// On SIGTERM:
shutdown.cancel();
```

### Signal handling

```rust
async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.unwrap(); };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap()
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
```

## Decision 5.5 — Sync wrapper around an async library (tokio-modbus pattern)

A common need: your library's primary API is async (for efficiency, for concurrent I/O), but some users want to call it from sync code without adopting tokio themselves. Pattern: offer sync wrappers as feature-gated variants that internally `block_on` a private runtime.

tokio-modbus exemplifies this:

```toml
[features]
rtu = ["dep:tokio-serial"]
tcp = []
sync = []                  # internal marker
rtu-sync = ["rtu", "sync"]  # sync wrapper around async RTU client
tcp-sync = ["tcp", "sync"]  # sync wrapper around async TCP client
```

The sync API is implemented by spawning a private `tokio::runtime::Runtime`, running the async call via `block_on`, and returning the result. Users who already have a tokio runtime use the async API; users in sync contexts opt into a sync variant via the feature flag.

**When to offer sync wrappers:**

- Your library is clearly async-first (I/O-heavy, concurrent)
- But you have users in sync environments (CLI tools, industrial scripts, PLC communication) who would otherwise write their own block_on wrapper
- The cost is small: a few `_sync` fns calling `runtime.block_on(async_fn())`

**Don't offer sync wrappers when:**

- Your library is not genuinely async in shape (ceremonial `async fn` without awaits)
- Sync consumers could trivially wrap themselves (one-line `pollster::block_on(...)` calls)
- The internal runtime creation would be hidden global state — be explicit about it if you do

## Decision 6 — When NOT to add channels

Most Rust applications don't need channels for internal component communication. Direct function calls through trait-bounded dependencies are the right default.

Adding a channel is justified ONLY when:

- **Work can be deferred.** Logging, metrics, notifications — caller shouldn't wait for the receiver.
- **Producer faster than consumer.** Bounded mpsc provides backpressure.
- **Multiple consumers on the same events.** Broadcast makes this natural.
- **Cross-task decoupling.** Background worker independent of request handlers.

If none of these apply, use a method call. A bounded mpsc between two objects that always work together is just indirection.

## Decision 7 — Timeout budget

Every external call has a timeout. Design the cascade:

```
Request enters HTTP handler      30s budget
  ├── DB query                   5s  (fails fast if slow)
  ├── External HTTP call         3s  (retry budget = 2x3 = 6s)
  └── Response serialization     <1s
Total worst case: 5 + 6 + 1 = 12s < 30s  ✓
```

Outer > middle > inner. If inner timeouts exceed outer, the outer fires first and you get meaningless "handler timed out" instead of "database was slow."

`tokio::time::timeout(Duration::from_secs(N), fut)` wraps any future.

## Decision 8 — Blocking in async

Async runtimes are cooperative. A blocking call (sync I/O, CPU-heavy work, `std::thread::sleep`) holds a worker thread and prevents other tasks from running.

Rules:
- **Never** `std::thread::sleep` in async. Use `tokio::time::sleep`.
- **Never** synchronous file I/O in async. Use `tokio::fs::*` or `spawn_blocking`.
- **Never** CPU-heavy computation in async without `spawn_blocking` or `rayon`.

`tokio::task::spawn_blocking(|| sync_work())` moves the work to Tokio's blocking-thread pool, preserving the reactor for other tasks.

```rust
// Blocking file I/O in async — DO NOT use std::fs here
async fn read_config() -> anyhow::Result<Config> {
    // Option A: async filesystem API
    let s = tokio::fs::read_to_string("config.toml").await?;
    Ok(toml::from_str(&s)?)
    
    // Option B: spawn_blocking if no async alternative
    // let config = tokio::task::spawn_blocking(|| {
    //     let s = std::fs::read_to_string("config.toml")?;
    //     toml::from_str::<Config>(&s).map_err(Into::into)
    // }).await??;
}
```

## Related

- [rust-implementing/async-patterns.md](../rust-implementing/async-patterns.md) — implementation: channel patterns, `select!`, `JoinSet` templates, Tower `Service`, Pin/Unpin internals
- [rust-planning/SKILL.md §10](SKILL.md#10-async-strategy) — planning rules summary
- [rust-reviewing/SKILL.md §7.4](../rust-reviewing/SKILL.md#74-async-correctness) — async correctness review checklist
- [rust-reviewing/debugging-playbook.md](../rust-reviewing/debugging-playbook.md) — async bug patterns (deadlock, stall, missed wake)
