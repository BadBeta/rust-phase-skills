# Error Strategy

Planning-phase decisions for error handling: library vs application error types, `thiserror` vs `anyhow` vs `color-eyre` vs `miette`, hand-rolled `Error + Display + ErrorKind` pattern, multi-layer translation, when NOT to use `Box<dyn Error>`, error recovery strategies.

For implementation-side code (`?` operator chains, `thiserror` derives, `anyhow::Context`, `From` conversions), see [rust-implementing/error-handling.md](../rust-implementing/error-handling.md). For the planning rules summary, see [rust-planning/SKILL.md §7 Error Strategy](SKILL.md#7-error-strategy-planning-level).

## Decision 1 — Library vs application

The two have fundamentally different error-handling needs.

| Concern | Library | Application |
|---|---|---|
| Error type shape | Typed enum (or hand-rolled `impl Error`) | `anyhow::Error` acceptable at main level |
| Callers pattern-match errors? | Yes — variants visible in docs | Usually no — logged/returned to user |
| Error message rendering | Display impl, `# Errors` doc section | Rendered at UI boundary |
| Extensibility | `#[non_exhaustive]` on public enums | N/A |
| Backtrace | Optional (capture via `Backtrace::capture()` in a field) | Usually yes (`anyhow::Error` captures) |
| Example | `sqlx::Error`, `reqwest::Error`, `thiserror`-derived | `anyhow::Result<()>` in `main` |

**Rule:** `anyhow::Error` is for applications and the top of your own stack. Library APIs return typed errors so callers can match on them.

## Decision 2 — Error crate choice

| Crate | When | Pattern |
|---|---|---|
| **thiserror** | Library error types — most common | `#[derive(thiserror::Error, Debug)]` enum |
| **anyhow** | Application error handling, `main`-level glue | `anyhow::Result<T>`, `.context(...)?` |
| **color-eyre** | Like anyhow with nicer reports | Same API as eyre/anyhow |
| **miette** | User-facing diagnostic quality (compiler-style) | `#[derive(miette::Diagnostic)]` |
| **Hand-rolled** | Top-tier libraries where you want full control | `struct Error { kind: ErrorKind }` + `impl Display + impl Error` |

### thiserror pattern (most libraries)

```rust
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]                  // allow adding variants without breaking callers
pub enum ConfigError {
    #[error("io error reading {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },
    
    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),
    
    #[error("missing required key: {0}")]
    Missing(String),
}
```

### anyhow pattern (application)

```rust
fn main() -> anyhow::Result<()> {
    let config = load_config()
        .context("loading application config")?;
    let conn = connect_db(&config)
        .context("connecting to database")?;
    // ...
    Ok(())
}
```

### Hand-rolled pattern (ripgrep, tokio, hyper, serde, polars, rustls)

For top-tier libraries where every aspect of error representation matters:

```rust
pub struct Error(Box<ErrorInner>);  // Box keeps size small

#[non_exhaustive]
enum ErrorKind {
    Io(io::Error),
    Parse { line: u32, col: u32, message: String },
    // ...
}

struct ErrorInner {
    kind: ErrorKind,
    // backtrace, context, etc.
}

impl Error {
    pub fn kind(&self) -> &ErrorKind { &self.0.kind }
}

impl fmt::Display for Error { /* ... */ }
impl fmt::Debug for Error { /* ... */ }
impl std::error::Error for Error { /* ... */ }
impl From<io::Error> for Error { /* ... */ }
```

Benefits: small `Error` size (8 bytes on 64-bit), explicit variants, room for backtraces and context, `#[non_exhaustive]` at both `Error` and `ErrorKind`.

Cost: more boilerplate; usually only worth it for widely-used libraries.

### Advanced variants from polars

For very high-throughput libraries, two polars techniques worth knowing:

```rust
// (1) ErrString(Cow<'static, str>) — allocate only when the error message
//     is dynamic; use static &str for canned messages (no allocation on
//     happy-path error construction)
pub struct ErrString(Cow<'static, str>);
impl From<&'static str> for ErrString { /* ... */ }
impl From<String> for ErrString { /* ... */ }

// (2) Arc<io::Error> — io::Error is NOT Clone. Wrapping it in Arc makes
//     the enclosing error type Clone without losing information. Polars
//     does: IO { error: Arc<io::Error>, msg: Option<ErrString> }.
pub enum PolarsError {
    IO { error: Arc<io::Error>, msg: Option<ErrString> },
    // ... 14 more variants
}

pub type PolarsResult<T> = Result<T, PolarsError>;
```

Also: polars does NOT use `#[non_exhaustive]` on `PolarsError` — this is deliberate, consistent with the "exhaustive matching is a feature for app-internal callers" principle. Published library + exhaustive enum = explicit decision that the variant set is stable.

### Hierarchical enum-of-enums (rustls)

When the error space has many subcategories (TLS is: "bad message format" OR "peer misbehaved" OR "cert invalid" OR …), a flat 30-variant enum becomes unwieldy. rustls uses a **hierarchical pattern**: top-level `Error` has ~20 variants, and several of those variants carry their own dedicated sub-enums which are ALSO `#[non_exhaustive]`.

```rust
#[non_exhaustive]
pub enum Error {
    InvalidMessage(InvalidMessage),        // sub-enum
    PeerIncompatible(PeerIncompatible),    // sub-enum
    PeerMisbehaved(PeerMisbehaved),        // sub-enum
    AlertReceived(AlertDescription),       // sub-enum
    InvalidCertificate(CertificateError),  // sub-enum
    // ... plus structured variants:
    InappropriateMessage { expect_types: Vec<ContentType>, got_type: ContentType },
    // ... plus simple variants:
    NoCertificatesPresented, DecryptError, EncryptError,
    // ... plus escape hatch:
    General(String),
    Other(OtherError),
}

#[non_exhaustive]
pub enum InvalidMessage { /* ~30 sub-variants */ }

#[non_exhaustive]
pub enum PeerMisbehaved { /* ~50 sub-variants */ }
```

Also notable: rustls documents `PeerMisbehaved` as "callers should NOT branch on these variants — report a bug if you see one." The `#[non_exhaustive]` is doing double duty: it permits future expansion AND reminds callers the variants are not a stable API contract.

**When to use hierarchical:**
- Domain has clear subcategories (protocol states, compile phases, file formats)
- Flat enum would exceed ~15 variants
- Some subcategories are "advisory" (user shouldn't branch on them individually, only on the category)

**Lint pair:** rustls enforces the discipline by setting `#![warn(clippy::exhaustive_enums, clippy::exhaustive_structs)]` at the crate root — clippy then demands `#[non_exhaustive]` on every public enum/struct, preventing accidents.

### The CLI-app stack: thiserror + miette

For CLIs and developer tools that produce user-facing diagnostic output (compiler-style with source-code highlighting), derive **both** `thiserror::Error` AND `miette::Diagnostic` on the same enum. This is nushell's pattern:

```rust
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Clone, Error, Diagnostic, PartialEq)]
pub enum ShellError {
    #[error("variable not found: {name}")]
    #[diagnostic(
        code(nu::shell::variable_not_found),
        help("check the spelling or define `{name}` first")
    )]
    VariableNotFound {
        name: String,
        #[label = "not defined in this scope"]
        span: Span,
    },

    #[error(transparent)]
    #[diagnostic(transparent)]
    Io(#[from] IoError),
}
```

Key miette features exercised by nushell:
- `#[diagnostic(code(ns::category::name))]` — machine-readable codes for tooling
- `#[label = "..."]` on a span-bearing field → source-code underline
- `#[help]` attribute / inline `help("...")` in `#[diagnostic(...)]` → suggestions to user
- `#[source_code]` — attach the source for the span to render
- `#[related]` — chain related errors (common compile-error shapes)
- `#[error(transparent)] #[diagnostic(transparent)]` — delegate both traits to an inner error for one-level-deeper variants

This is the modern idiomatic stack for CLI / language / developer tool error reporting. Use it when human users will read the errors.

## Decision 3 — Multi-layer error translation

Each architectural layer has its own error taxonomy. Errors translate at boundaries via `From` conversions.

```
HTTP layer        ApiError (HTTP status, user message)
    ↑ From
Application       AppError (business failures — "order not found")
    ↑ From
Domain            DomainError (invariant violations — "insufficient funds")
    ↑ From          (NEVER know about SQL errors)
Infrastructure    RepoError, GatewayError (technical — "connection timeout")
```

### Example

```rust
// domain/errors.rs
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum OrderError {
    #[error("order not found: {0}")]
    NotFound(OrderId),
    #[error("order cannot be modified in state {0:?}")]
    NotModifiable(OrderState),
}

// infra/errors.rs
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum RepoError {
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    #[error("connection pool: {0}")]
    Pool(#[from] deadpool::managed::PoolError<sqlx::Error>),
}

// app/errors.rs
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum AppError {
    #[error(transparent)]
    Order(#[from] OrderError),
    #[error(transparent)]
    Repo(#[from] RepoError),
}

// api/errors.rs
#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("internal")]
    Internal,
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Order(OrderError::NotFound(_)) => ApiError::NotFound,
            AppError::Order(OrderError::NotModifiable(_)) => ApiError::BadRequest(e.to_string()),
            AppError::Repo(_) => ApiError::Internal,    // Hide infra details from users
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            ApiError::NotFound => StatusCode::NOT_FOUND,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
```

**Domain never sees `sqlx::Error`.** Infra converts `sqlx::Error` → `RepoError`; app converts to `AppError`; api converts to `ApiError`.

## Decision 4 — When NOT to use `Box<dyn Error>`

**Never in library public APIs.** `Box<dyn Error>` loses type information, prevents pattern matching, and forces callers into string-comparison hell.

```rust
// BAD — library API
pub fn parse(s: &str) -> Result<Value, Box<dyn std::error::Error>> { ... }
// Caller can do:
match result {
    Ok(v) => ...,
    Err(e) => {
        // String matching? e.downcast_ref::<MyError>()? Neither is good.
    }
}

// GOOD — typed enum
pub fn parse(s: &str) -> Result<Value, ParseError> { ... }
// Caller pattern-matches:
match result {
    Err(ParseError::InvalidSyntax { line, .. }) => show_error_at(line),
    Err(ParseError::UnexpectedEof) => suggest_more_input(),
    Ok(v) => ...,
}
```

**OK in application code** (behind `anyhow::Error`) because nobody pattern-matches on main-level errors — they're logged or displayed.

## Decision 5 — `String` in `Result` is a smell

```rust
// BAD
fn load() -> Result<Config, String> { ... }

// GOOD
#[derive(thiserror::Error, Debug)]
enum LoadError { ... }
fn load() -> Result<Config, LoadError> { ... }
```

`String` loses:
- Variant information (what failed?)
- Source error chain
- Backtrace
- Structured data (fields)

Only exception: one-off scripts or internal prototypes.

## Decision 6 — Error-value recovery (`SendError<T>`)

Tokio channels return `SendError<T>` on send failure. The `T` inside is the value that failed to send — caller can retry, store, or log it:

```rust
match tx.send(msg).await {
    Ok(()) => {},
    Err(tokio::sync::mpsc::error::SendError(msg)) => {
        // msg was not sent; retry or save
        log::warn!("channel closed, retrying: {msg:?}");
    }
}
```

Design your error types similarly when a failure implies the caller should have the chance to recover the value:

```rust
#[derive(thiserror::Error, Debug)]
pub enum SaveError {
    #[error("database down; retry later")]
    Transient { order: Order },      // Caller can retry with the same Order
    #[error("validation: {0}")]
    Validation(String),              // Permanent; don't retry
}
```

## Decision 7 — Uninhabited error types (`Infallible`, `!`)

When a function "can't fail" in its signature but still returns `Result` for API consistency:

```rust
use std::convert::Infallible;

pub fn never_fails(x: i32) -> Result<i32, Infallible> {
    Ok(x * 2)
}

// Caller
let x = never_fails(5).unwrap();  // Safe — Infallible has no variants
```

Useful for trait impls where the trait requires `Result` but this implementor can't fail.

## Decision 8 — Recovery strategies

For each error variant, plan the recovery:

| Variant | Recovery |
|---|---|
| Transient (network timeout, rate limit) | Retry with backoff + jitter; max attempts |
| Programming error (impossible state) | Panic — this should be caught in tests |
| Input validation | Return to user; no retry |
| External service unavailable | Circuit break; fallback or degraded mode |
| Resource exhaustion (OOM, disk) | Reject new work; alert |
| Permission / auth | Return to user; don't log as error (too noisy) |

The recovery strategy determines:
- Whether retries happen (and where — adapter wrapping the external call)
- Whether to escalate (metric spike, alert, page)
- Whether to fall back (cached value, default, degraded mode)
- Whether to propagate as user-visible

## Decision 9 — Panics

Panics are for invariant violations — things that should never happen if the code is correct.

| Use | Panic? |
|---|---|
| User input is malformed | No — return `Err` |
| External dep returned unexpected data | No — return `Err` |
| An internal invariant is violated (empty stack, unreachable arm) | Yes — `panic!` or `unreachable!` |
| A `#[cfg(test)]` helper failed | Yes — `unwrap`/`expect` fine in tests |
| Library function called with nonsense args | Depends — document in `# Panics` |

In FFI boundaries, panics cross into UB. Wrap with `catch_unwind`:

```rust
#[no_mangle]
pub extern "C" fn my_ffi_fn() -> i32 {
    std::panic::catch_unwind(|| {
        // work that might panic
        0
    }).unwrap_or(-1)  // Convert panic to error return
}
```

## Related

- [rust-implementing/error-handling.md](../rust-implementing/error-handling.md) — implementation: `?` chains, thiserror derives, anyhow context, `From` patterns, error conversion chains
- [rust-planning/SKILL.md §7](SKILL.md#7-error-strategy-planning-level) — planning rules summary, master decision table for error strategy
- [rust-reviewing/SKILL.md §7.5](../rust-reviewing/SKILL.md#75-error-handling) — error handling review checklist
