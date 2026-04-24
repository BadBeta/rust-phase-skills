# Data Strategy

Planning-phase decisions for persistence: which store, which access library, migration strategy, connection pooling, caching strategy, data ownership across contexts.

For implementation-side code (SQLx queries, Diesel schema DSL, MongoDB drivers, connection pool setup, moka/deadpool patterns), see [rust-implementing/database.md](../rust-implementing/database.md). For planning rules summary, see [rust-planning/SKILL.md §7 Data Strategy](SKILL.md#7-data-strategy).

## Decision 1 — Which store?

| Store | Strengths | Weaknesses | When |
|---|---|---|---|
| **PostgreSQL** | Powerful SQL, strong consistency, JSON/JSONB, extensions (PostGIS, pgvector), proven | Operational complexity at scale | **Default for relational.** Unless you know otherwise. |
| **MySQL / MariaDB** | Ubiquitous, good enough for most | Less expressive than Postgres | Legacy or ops-team preference |
| **SQLite** | Embedded, no server, single-file | Single writer, not great for concurrent web | Edge, CLI, desktop, tests |
| **MongoDB** | Document model, rich queries | Eventual consistency subtleties, schema drift | When you truly have document-shaped data |
| **Redis** | Fast, pub/sub, data structures | Not persistent by default, single-region | Cache, queue, session store |
| **sled / redb** | Pure Rust, embedded KV | Smaller ecosystem | Rust-only deployments, tests, state files |
| **Kafka / NATS** | Durable streaming, pub/sub | Operational investment | Event bus, cross-service messaging |
| **DynamoDB / Spanner** | Cloud-scale, managed | Vendor commitment, different query model | Cloud-native, huge scale |

**Rule:** start with PostgreSQL unless you have a specific reason not to. Most projects never need anything else.

## Decision 2 — Access library

| Library | Paradigm | Sync/Async | When |
|---|---|---|---|
| **sqlx** | Compile-checked SQL | Async (Tokio, async-std) | **Default for async Rust.** Write SQL; compile-time type-checks against schema. |
| **diesel** | Type-safe ORM with schema DSL | Sync by default; async via `diesel_async` | Rich migrations, complex relations, team likes ORM |
| **tokio-postgres** | Raw async Postgres client | Async | Maximum control, custom pooling, specialized needs |
| **mongodb** | Official MongoDB driver | Async | MongoDB access |
| **redis / deadpool-redis** | Redis client + pooling | Both | Redis access |
| **fred** | Full-featured Redis client | Async | Redis with advanced features (clustering, sentinels) |
| **sled / redb** | Embedded KV | Both | Embedded KV |

### sqlx vs diesel — quick contrast

sqlx:
- Write SQL strings; `query_as!(User, "SELECT * FROM users WHERE id = $1", id)` — types checked at compile time against the DB
- Async by default
- Simpler migrations (`sqlx-cli`)
- More "just SQL"

diesel:
- Schema DSL: `users.filter(id.eq(user_id)).first::<User>(&conn)`
- Rich migration tooling
- Async via `diesel_async` but historically sync
- More type-safe; also more opinionated

**Most new Rust projects pick sqlx.** Diesel remains strong in specific ecosystems.

## Decision 3 — Migration strategy

Migrations are a deployment concern. Plan upfront.

### Rules

- **Migrations run from a dedicated binary or CLI** — not from app startup in production. App startup should fail fast if migrations are out of date; it should not silently run migrations that could fail halfway and corrupt state.
- **Forward-only** in production. Don't try to write "down" migrations that roll back. Instead, create a new migration that undoes a bad one.
- **Idempotent.** Running a migration twice should be a no-op (or error). `CREATE TABLE IF NOT EXISTS`, `ADD COLUMN IF NOT EXISTS` (PG 15+), etc.
- **Rolling-deploy compatible.** During deploy, old and new code briefly coexist. Migration must be safe for both versions of the app code. A common pattern:
  1. Deploy migration-1: adds new column (nullable).
  2. Deploy app version that writes to both old and new.
  3. Backfill the new column.
  4. Deploy app version that reads from new column.
  5. Deploy migration-2: drops old column.

### Tooling

- **sqlx-cli**: built-in, simple, works with sqlx schema files
- **refinery**: embedded-migrations, supports Postgres/MySQL/SQLite
- **diesel**: bundled CLI with schema-driven migrations
- Custom: a `mix migrate`-style binary in your workspace — `cargo run --bin migrate`

## Decision 4 — Connection pool

- **One pool per process**, passed via `Arc<PgPool>` or `State<PgPool>`.
- Pool size ~ CPU cores × 2-4 for sync workloads; less for async.
- **Don't** create a new pool per request (big overhead).
- **Don't** use a global `LazyLock<PgPool>` in library code — inject via constructor for testability.

```rust
let pool = sqlx::PgPool::connect(&config.database_url).await?;
let app = axum::Router::new()
    .route("/orders", get(list_orders))
    .with_state(Arc::new(pool));
```

## Decision 5 — Caching strategy

Caching is a design decision, not an afterthought. Pick ONE approach per cache:

| Strategy | Description | When |
|---|---|---|
| **TTL-based** | Cache entries expire after N seconds | Simplest; tolerates staleness |
| **Write-through** | Write to DB, then update cache | Consistency over speed |
| **Write-behind** | Write to cache, async flush to DB | Speed over durability; accepts write-loss risk |
| **Invalidate on event** | PubSub or DB triggers notify cache | Multi-writer scenarios; complex |
| **Read-through** | Cache miss triggers DB fetch + cache populate | Natural read-heavy patterns |

### In-memory caching

```rust
use moka::future::Cache;

let cache: Cache<OrderId, Order> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300))
    .build();
```

moka is async-aware (futures), uses TinyLFU eviction, suitable for concurrent access.

### Distributed caching

```rust
use deadpool_redis::{Config, Runtime};

let cfg = Config::from_url(&env::var("REDIS_URL")?);
let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
```

### Cache invalidation is hard

- **Simpler: short TTL.** 30s-5min tolerates staleness and eliminates invalidation complexity.
- **Medium: invalidate on write.** Writer updates DB + deletes cache key.
- **Hardest: multi-writer with event bus.** Each writer publishes an invalidation event; all caches subscribe.

## Decision 6 — Data ownership

**One crate/module owns each entity's persistence.** The `Orders` context owns the `orders` table; nobody else writes to it.

- Reads by other contexts go through the owner's public API, not direct SQL.
- Cross-context joins are a smell — they couple two contexts at the DB level.
- If you need data from two contexts together, consider:
  - Denormalized read model (CQRS)
  - Foreign context's public API
  - Event sourcing / projections

## Decision 7 — Transactions

- **One transaction per use case.** Multiple aggregates in one TX → they're really one aggregate.
- **Never open a transaction from the HTTP layer.** Use-case layer owns transaction scope.
- **Saga for cross-service consistency**, not distributed transactions. Each step is locally committed; compensations on failure.

```rust
// sqlx transaction pattern
let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO orders ...").execute(&mut *tx).await?;
sqlx::query!("INSERT INTO line_items ...").execute(&mut *tx).await?;
tx.commit().await?;
```

## Decision 8 — Testing with real DB

- **`#[sqlx::test]`** (sqlx) — creates a new DB per test, migrations applied, automatic cleanup.
- **Transaction-based isolation** — each test runs in a transaction that rolls back at end. Fast, but can't test code that commits.
- **Docker Compose** — spin up real DB for integration tests. `testcontainers` crate for ergonomics.
- **Mock the repository trait** for unit tests that shouldn't touch DB.

See [rust-planning/test-strategy.md](test-strategy.md) for the planning-level test pyramid.

## Related

- [rust-implementing/database.md](../rust-implementing/database.md) — implementation: sqlx queries, connection pools, migrations, Diesel schema DSL, MongoDB, caching (moka/dashmap/Redis), query composition
- [rust-planning/SKILL.md §7](SKILL.md#7-data-strategy) — planning rules summary
- [rust-planning/domain-patterns.md](domain-patterns.md) — DDD, bounded contexts (consistency with data ownership)
