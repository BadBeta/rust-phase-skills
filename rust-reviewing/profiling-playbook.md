# Profiling Playbook — Deep Reference

Tool selection and usage for measuring Rust performance. The SKILL.md hub (§5, §9) has the tight version; this file has commands, interpretation, and trade-offs.

For correctness bugs, see [debugging-playbook.md](debugging-playbook.md). For common perf pitfalls by symptom, see [performance-catalog.md](performance-catalog.md).

## Rules before profiling

1. **Always profile release builds.** `cargo build --release` or `cargo bench`. Debug builds are 10-100x slower and mislead about where time goes.
2. **Measure before guessing.** Intuition is wrong more often than right in Rust — zero-cost abstractions and LLVM optimizations change what's hot.
3. **Fix one thing at a time.** Measure → change → measure. Otherwise you don't know which change helped.
4. **Profile in realistic conditions.** Cold cache vs warm, single-threaded vs contended, real data size vs 10 items.
5. **Stop at diminishing returns.** The first 80% of performance comes from algorithmic fixes and removing obvious waste. Micro-optimization rarely justifies its complexity.

## Tool selection

### CPU time: "Where is the time going?"

| Tool | Platforms | Overhead | When |
|---|---|---|---|
| **cargo flamegraph** | Linux, macOS, Windows (via inferno) | Low (sampling) | First reach — easiest cross-platform |
| **samply** | Linux, macOS | Low (sampling) | No root needed; good alternative to perf |
| **perf** | Linux | Low | Most powerful; hardware counters |
| **Instruments** | macOS | Low | GUI; integrates with Xcode |
| **VTune** | Linux, Windows (Intel) | Low | Detailed microarchitectural analysis |

### Micro-benchmarks: "Which of these two implementations is faster?"

| Tool | Purpose |
|---|---|
| **criterion** | Statistical rigor, comparisons across runs, CI-friendly HTML reports |
| **iai** | Cachegrind-based; deterministic (no system noise); catches cache-behavior changes |
| **divan** | Newer, lighter weight than criterion |

### Heap: "Where are allocations coming from?"

| Tool | Purpose |
|---|---|
| **DHAT** (Valgrind) | Most detailed; per-allocation metadata |
| **heaptrack** (Linux) | Comprehensive; GUI for exploration |
| **dhat-rs** | In-process selective profiling |
| **jemalloc** + **jemalloc-ctl** | Runtime allocator stats |
| **Valgrind memcheck** | Leaks + use-after-free (not performance) |

### Async runtime: "Why are my Tokio tasks slow?"

| Tool | Purpose |
|---|---|
| **tokio-console** | Task states, lock wait times, I/O events, channel fill levels |
| **tracing + EnvFilter** | Custom instrumentation |

### Compile time / binary size

| Tool | Purpose |
|---|---|
| **cargo build --timings** | Shows per-crate compile time as HTML |
| **-Zself-profile** | rustc profile (nightly only) |
| **cargo-llvm-lines** | Which functions monomorphize most; code-gen bloat |
| **cargo-bloat** | Which functions are biggest in the output binary |
| **strip / llvm-strip** | Remove debug info |

---

## Workflow: CPU profiling

### Step-by-step

1. **Run once under a realistic workload to establish baseline.**
   ```sh
   cargo build --release
   time ./target/release/my-app < realistic-input.dat > /dev/null
   ```
2. **Generate a flamegraph.**
   ```sh
   cargo install flamegraph
   cargo flamegraph --bin my-app -- arg1 arg2
   # Opens flamegraph.svg in your browser
   ```
3. **Read the flamegraph top-down.**
   - Wide bars at the top = hot functions
   - Look for "surprising" width — a library function you thought was fast
   - Allocation stacks appear under `alloc::alloc_zeroed` or similar
4. **Identify the top 1-3 hotspots.**
5. **Form a hypothesis** (cloning, allocation, mutex, N+1, etc.).
6. **Make the smallest possible fix.**
7. **Re-profile to verify improvement.**
8. **Next hotspot is now different — stop when benefits don't justify effort.**

### samply (alternative, no root)

```sh
cargo install samply
samply record ./target/release/my-app arg1 arg2
# Uploads to the Firefox profiler UI; local HTML report
```

### perf (Linux, most powerful)

```sh
cargo build --release
perf record -g --call-graph dwarf ./target/release/my-app args
perf report
# Or: perf report --stdio > report.txt
```

Hardware counters:
```sh
perf stat -e cache-misses,cache-references,branch-misses,branch-instructions \
  ./target/release/my-app args
```

### Interpreting CPU profiles

- **Top function in self time** — the one burning CPU directly
- **Top function in cumulative time** — the one whose call tree burns CPU (often a top-level fn)
- **`kernel`/`__vdso` time** — system calls; often I/O
- **`jemalloc_usable_size` / `malloc`** — allocation-heavy code path
- **`std::sys::unix::...`** — might be syscalls; narrow down

---

## Workflow: Micro-benchmarking

### criterion setup

```toml
# Cargo.toml
[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "my_bench"
harness = false
```

```rust
// benches/my_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_parse(c: &mut Criterion) {
    let input = std::fs::read_to_string("tests/data.json").unwrap();
    c.bench_function("parse_json", |b| {
        b.iter(|| serde_json::from_str::<MyStruct>(black_box(&input)).unwrap())
    });
}

fn bench_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_by_size");
    for size in [10, 100, 1000, 10_000] {
        let data = generate_input(size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, d| {
            b.iter(|| parse(black_box(d)));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_parse, bench_sizes);
criterion_main!(benches);
```

Run: `cargo bench`. HTML report in `target/criterion/report/index.html`.

### Tips

- **`black_box`** prevents the compiler from optimizing away your work
- **Separate setup from measurement** — use `b.iter_with_setup` for per-iteration setup
- **Compare alternatives** — `group.bench_function("impl-a", ...)` and `group.bench_function("impl-b", ...)`
- **Track regressions in CI** — criterion compares against saved baselines
- **Realistic input size** — 10 items is unrealistic for most code

### iai for deterministic benches

```rust
use iai::black_box;
fn bench_parse() -> MyStruct {
    serde_json::from_str(black_box(INPUT)).unwrap()
}
iai::main!(bench_parse);
```

iai uses Cachegrind; numbers are reproducible across machines. Good for CI perf gates.

---

## Workflow: Heap profiling

### DHAT (Valgrind)

```sh
# Slow but detailed
valgrind --tool=dhat ./target/release/my-app args
# Produces dhat.out.NNN — open in https://nnethercote.github.io/dh_view/dh_view.html
```

Shows per-allocation-site:
- Total bytes allocated
- Peak live bytes
- Number of allocations
- Average lifetime

### heaptrack (Linux GUI)

```sh
heaptrack ./target/release/my-app args
heaptrack_gui heaptrack.my-app.NNNN.zst
```

GUI shows allocation timeline, peak memory, callgraph, flamegraph of allocations.

### dhat-rs (in-process, selective)

```toml
# Cargo.toml
[dev-dependencies]
dhat = "0.3"
```

```rust
fn main() {
    let _profiler = dhat::Profiler::new_heap();
    // ... your code ...
    // Drops the guard, writes dhat-heap.json
}
// View at https://nnethercote.github.io/dh_view/dh_view.html
```

Good for profiling a specific sub-phase without Valgrind overhead everywhere.

### Jemalloc runtime stats

```toml
[dependencies]
tikv-jemallocator = "0.5"
tikv-jemalloc-ctl = "0.5"
```

```rust
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// Query at runtime
use tikv_jemalloc_ctl::{stats, epoch};
epoch::advance().unwrap();
println!("allocated: {}", stats::allocated::read().unwrap());
println!("resident: {}", stats::resident::read().unwrap());
```

Useful for monitoring long-running services.

---

## Workflow: Async profiling

### tokio-console

```toml
[dependencies]
tokio = { version = "1", features = ["full", "tracing"] }
console-subscriber = "0.4"
```

```rust
fn main() {
    console_subscriber::init();
    // Your normal Tokio app
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { /* ... */ });
}
```

Run your app, then in another terminal:
```sh
cargo install tokio-console
tokio-console http://127.0.0.1:6669
```

Shows:
- **Tasks** — state (Running/Idle/Completed), CPU time, wake count
- **Resources** — mutex wait times, channel fill levels
- **Async ops** — I/O events

Interpret:
- Task stuck in `Idle` with no wakes → forgotten task or missing signal
- Task stuck `Running` → blocking call in async
- Lock with high `contention` → bottleneck; consider channel or sharding

### Custom instrumentation with tracing

```rust
#[tracing::instrument(skip(pool), fields(user_id))]
async fn get_user(pool: &PgPool, id: UserId) -> Result<User, Error> {
    // ... tracing events automatically include user_id
}
```

With `RUST_LOG=my_app=trace`, you see entry/exit and `tracing::info!`/`debug!` events.

---

## Workflow: Compile time

```sh
cargo build --timings
# Open target/cargo-timings/cargo-timing-*.html
```

Shows:
- Which crates take the longest
- Parallelism utilization (are you serialized on one crate?)
- Codegen vs frontend time

### Finding expensive generic instantiations

```sh
cargo install cargo-llvm-lines
cargo llvm-lines --release | head -40
```

Shows functions that generate the most LLVM IR — usually overly-generic code. Reduce with:
- Concrete types where generics don't help
- `#[inline(never)]` on hot-path generics to reduce instantiation
- Move to runtime dispatch (`dyn Trait`) for call sites that don't need monomorphization

### Binary size

```sh
cargo install cargo-bloat
cargo bloat --release --crates | head
cargo bloat --release | head -20  # Per-function
```

Strategies:
- `strip = true` in release profile (removes debug symbols)
- `codegen-units = 1` (allows more inlining / dead-code elimination)
- `panic = "abort"` (removes unwinding tables)
- `opt-level = "z"` (size-optimized) or `"s"`
- Fewer generic instantiations (see llvm-lines above)

---

## Workflow: I/O profiling

### eBPF / bpftrace (Linux)

```sh
# Trace all read() syscalls by your process
sudo bpftrace -e 'tracepoint:syscalls:sys_enter_read /pid == <PID>/ { @[comm] = count(); }'
```

### strace (Linux)

```sh
strace -c ./target/release/my-app args
# Summary of syscall counts and time
```

Shows if you're doing unexpected sync I/O or many small reads.

---

## Common profiling pitfalls

| Pitfall | Symptom | Fix |
|---|---|---|
| Profiling debug build | Everything looks slow | `--release` |
| Profiling with tiny input | Startup dominates; misleading | Realistic data size |
| Cold vs warm cache | First run slow, rest fast | Warm cache first; measure many runs |
| Missing `black_box` in bench | Compiler optimizes away work | Wrap inputs and outputs with `black_box` |
| Single-threaded bench for concurrent code | Doesn't show contention | Benchmark with real thread count |
| `--release` with `debug-assertions = true` | Extra checks inflate numbers | Check profile settings |

---

## Related

- [rust-reviewing/SKILL.md §5, §9](SKILL.md#9-profiling-playbook-tight-full-detail-in-profiling-playbookmd) — tight summary
- [performance-catalog.md](performance-catalog.md) — common pitfalls → root cause → fix
- [rust-implementing/data-structures.md](../rust-implementing/data-structures.md) — benchmarking setup, criterion patterns
- [rust-implementing/observability.md](../rust-implementing/observability.md) — tracing, structured logging, metrics
