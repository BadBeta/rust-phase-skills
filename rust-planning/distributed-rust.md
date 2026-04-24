# Distributed Rust

Planning-phase decisions for multi-node Rust systems: when distribution is justified, service contracts (gRPC/HTTP), message bus choice, partition handling, idempotency across services.

For microservices architecture patterns, TCP/TLS setup, and resilience code, see [rust-planning/services-architecture.md](services-architecture.md). For the planning-rules summary, see [rust-planning/SKILL.md §5 (Stage 4)](SKILL.md#5-project-layout-decisions).

## Decision 1 — Should this be distributed at all?

**Rule:** don't introduce distribution until single-node is maxed out.

Single node exhausts long before you think:

- `Task::spawn` with thousands of tasks — trivial for Tokio
- `rayon::par_iter` for CPU-bound batches
- Read replicas for scaling reads
- A single Postgres can handle tens of thousands of transactions/second
- An axum server with a tuned Tokio runtime can handle 100K+ RPS on modest hardware

Distribution brings:
- Network partitions (CAP theorem bites)
- Split-brain scenarios
- Eventual consistency
- Distributed tracing complexity
- Operational complexity (deploys, monitoring, debugging)

**Only distribute when you have a specific reason the above can't solve**, e.g.:

- **Geographic distribution** for latency (multi-region deployment)
- **Regulatory isolation** — data must stay in region
- **Independent scaling needs** — one service handles 100x traffic of another
- **Different languages** — your data science is Python
- **Organizational boundaries** — separate teams with separate release cadence

## Decision 2 — Service contract

| Mechanism | When | Ecosystem |
|---|---|---|
| **HTTP + JSON** | Public APIs, simple internal services, easy debugging | reqwest, axum, any HTTP client in any lang |
| **gRPC (tonic)** | Internal service-to-service, strong contracts, streaming | tonic (server + client), protobuf |
| **WebSocket** | Bi-directional streaming with web clients | tokio-tungstenite, axum |
| **Message bus (Kafka/NATS)** | Async event-driven, durable | rdkafka, async-nats |
| **Shared database** | **AVOID.** Couples services tightly | — |

### gRPC with tonic

```rust
// build.rs
fn main() {
    tonic_build::compile_protos("proto/service.proto").unwrap();
}

// service.proto
syntax = "proto3";
service Orders {
  rpc PlaceOrder(PlaceOrderRequest) returns (PlaceOrderResponse);
}

// Server
#[tonic::async_trait]
impl Orders for OrdersService {
    async fn place_order(
        &self,
        request: Request<PlaceOrderRequest>,
    ) -> Result<Response<PlaceOrderResponse>, Status> {
        // ...
    }
}
```

### HTTP + JSON

```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;

let response = client
    .post("http://orders-service/orders")
    .json(&request_body)
    .send()
    .await?
    .error_for_status()?
    .json::<OrderResponse>()
    .await?;
```

## Decision 3 — Message bus choice

| Bus | When |
|---|---|
| **Kafka** | Durable event log, replay, high-throughput streaming. Operational complexity high. |
| **NATS** | Lightweight pub/sub, request-reply; JetStream for durability. Lower ops cost than Kafka. |
| **RabbitMQ** | Classic work queues, routing, dead-letter. Good for job processing. |
| **Redis Streams** | Simpler than Kafka, good for smaller-scale event logs |
| **Postgres LISTEN/NOTIFY** | Tiny-scale pub/sub within one DB cluster — not durable messaging |

## Decision 4 — Cross-service idempotency

Retries across services must not duplicate effects. Design idempotency in:

- **Idempotency-Key header** — client sends unique key per logical operation; server deduplicates
- **Natural idempotency** — PUT with resource ID, deterministic state
- **Outbox pattern** — service writes to local DB + outbox table in one transaction; separate worker publishes to bus

```rust
// Outbox pattern
// In the use case:
let mut tx = pool.begin().await?;
sqlx::query!("INSERT INTO orders ...").execute(&mut *tx).await?;
sqlx::query!(
    "INSERT INTO outbox (topic, payload, idempotency_key) VALUES ($1, $2, $3)",
    "order-placed", payload, idempotency_key,
).execute(&mut *tx).await?;
tx.commit().await?;

// Separate worker process:
loop {
    let events = fetch_unpublished_from_outbox(&pool).await?;
    for event in events {
        bus.publish(&event.topic, &event.payload).await?;
        mark_published(&pool, event.id).await?;
    }
}
```

## Decision 5 — Partition handling

Network partitions are inevitable in distributed systems. For each cross-service call, decide:

- **Timeout** — always set; cascade correctly (outer > middle > inner)
- **Retry strategy** — exponential backoff, jitter, capped attempts
- **Circuit breaker** — after N failures, fail fast instead of wasting resources
- **Fallback** — cached last-good value, degraded mode, user-visible error
- **Bulkhead** — thread/connection pool separation so one failing dep doesn't starve others

See [rust-planning/services-architecture.md](services-architecture.md) for resilience pattern code.

## Decision 6 — Observability across services

Distributed = you can't use a single stack trace. Plan for:

- **Trace propagation** — `traceparent` HTTP header, OpenTelemetry spans across services
- **Correlation IDs** — request_id propagated through every call (include in logs)
- **Centralized logging** — structured JSON logs collected (Loki, Elasticsearch, CloudWatch)
- **Metrics** — Prometheus-compatible metrics exposed; dashboards show per-service latency/error rate
- **Health + readiness endpoints** — `/health` (alive), `/ready` (deps available)

```rust
// OpenTelemetry tracing
use opentelemetry::global;
use tracing_opentelemetry::OpenTelemetryLayer;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;

tracing_subscriber::registry()
    .with(OpenTelemetryLayer::new(tracer))
    .with(tracing_subscriber::fmt::layer().json())
    .init();
```

## Decision 7 — Deployment model

| Model | When |
|---|---|
| Monolith | Stage 0-3 — start here |
| Modular monolith | Stage 3 — Cargo workspace with crate boundaries, deployed as one binary |
| Nanoservices | Stage 3-4 — feature-gated binaries from same workspace |
| Microservices | Stage 4 — separate workspaces, separate deploys, service contracts |
| Service mesh (Istio, Linkerd) | Stage 5 — many services, cross-cutting concerns (mTLS, retry, tracing) |

**The progression is additive.** Start monolithic; split only when the split's specific benefit justifies the cost.

## Decision 8 — Data consistency across services

| Pattern | When | Trade-off |
|---|---|---|
| Strong consistency (distributed TX) | Almost never — operational nightmare | Correct but slow + fragile |
| Saga | Multi-step cross-service operation | Compensating actions on failure |
| Eventual consistency + outbox | Event-driven updates | Consumers see stale state briefly |
| Read-your-own-writes cache | Within a single service, consistency needed for user | Complex cache invalidation |
| Idempotent consumers | Any retryable consumer | Plan idempotency key + dedup |

## Related

- [rust-planning/services-architecture.md](services-architecture.md) — microservices patterns, kernel, resilience (circuit breakers, retries), service discovery
- [rust-planning/data-strategy.md](data-strategy.md) — store choice, migration, caching
- [rust-planning/SKILL.md §5](SKILL.md#5-project-layout-decisions) — Stage 4 (distributed) trigger conditions
- [rust-implementing/web-apis.md](../rust-implementing/web-apis.md) — HTTP server/client implementation
