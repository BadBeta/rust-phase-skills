# Security Audit Checklist

Security review for Rust code. The SKILL.md hub (§7.9) has the tight checklist; this file has deeper detail on each category with BAD/GOOD patterns.

For unsafe review specifically, see [SKILL.md §7.6](SKILL.md#76-unsafe-and-ffi). For dependency audit, see §5 here and [rust-planning/workspace-layout.md](../rust-planning/workspace-layout.md).

## How to use this file

Scan the checklists in order. Each category has "check for" items and BAD/GOOD patterns. Flag anything found with severity per [SKILL.md §6](SKILL.md#6-severity-classification). Most security findings are **block**-severity.

## Contents

1. Input validation at system boundaries
2. Injection attacks (SQL, command, log, format)
3. Authentication and authorization
4. Secrets management
5. Crypto primitives
6. Supply chain (dependencies)
7. Unsafe code review
8. Logging discipline
9. HTTP-specific (TLS, CORS, headers)
10. Deserialization

---

## 1. Input validation at system boundaries

### Check for
- [ ] All HTTP request bodies validated (axum extractors with typed structs + `#[derive(Deserialize)]` with `#[serde(deny_unknown_fields)]`)
- [ ] All CLI args validated (clap derive with `#[arg]` constraints or `value_parser`)
- [ ] All config file contents validated (typed `Deserialize` + explicit ranges)
- [ ] All deserialized messages (queue consumers, webhooks) validated before use
- [ ] Numeric inputs bounded (max size, non-negative where required)
- [ ] String inputs length-limited
- [ ] File paths validated to prevent traversal (`../../../etc/passwd`)

### BAD / GOOD

```rust
// BAD — accepts any JSON, uses fields raw
#[derive(Deserialize)]
struct CreateUser {
    name: String,
    age: u32,
    role: String,
}
// Attacker sends {"role": "admin"} → becomes admin

// GOOD — typed role, explicit constraints
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateUser {
    #[serde(deserialize_with = "deserialize_bounded_string")]
    name: String,  // 1-255 chars
    age: Age,  // newtype with validation in constructor
    // role removed from input; set by server based on auth context
}

// BAD — file access with user-controlled path
fn read_file(path: &str) -> Result<Vec<u8>> {
    std::fs::read(path)  // User can pass /etc/passwd
}

// GOOD — canonicalize and verify inside allowed dir
fn read_file(basedir: &Path, filename: &str) -> Result<Vec<u8>> {
    let joined = basedir.join(filename);
    let canonical = joined.canonicalize()?;
    if !canonical.starts_with(basedir) {
        return Err(Error::PathEscape);
    }
    std::fs::read(canonical)
}
```

---

## 2. Injection attacks

### SQL injection

Parameterized queries only. Never interpolate user input into SQL.

```rust
// BAD
let q = format!("SELECT * FROM users WHERE email = '{}'", email);
sqlx::query(&q).fetch_one(&pool).await?;

// GOOD — parameterized
sqlx::query!("SELECT * FROM users WHERE email = $1", email)
    .fetch_one(&pool).await?;
```

`sqlx::query!` (compile-checked) or `sqlx::query_as!` (typed result) is the standard. Diesel's DSL is also safe by construction.

### Command injection

```rust
// BAD — shell interpolation
std::process::Command::new("sh")
    .arg("-c")
    .arg(format!("grep {} file.txt", user_pattern))  // SHELL INJECTION
    .output()?;

// GOOD — direct exec, user input as arg
std::process::Command::new("grep")
    .arg(user_pattern)  // grep receives literal string, not shell-parsed
    .arg("file.txt")
    .output()?;
```

Avoid `sh -c` with user input entirely. Direct `Command::new` with args is safe.

### Format string injection

Rust format strings are compile-time parsed; `format!("{user_input}")` doesn't allow user input to BE the format. But be wary of dynamic format strings:

```rust
// If you're using a runtime format system (uncommon), user input as format string is bad.
// Native Rust format! is safe.

// BAD — log with user input as format string
log::info!(&user_message);  // Could be "{}%%" and log crate might interpret

// GOOD
log::info!("user said: {}", user_message);  // user_message is an arg, not format
```

### Log injection

User input in log messages can inject newlines, ANSI escape codes, or fake log entries.

```rust
// BAD
log::info!("user {} logged in", user_name);  // If user_name = "hacker\n[ERROR] root logged in"

// BETTER
log::info!(user_name = %user_name, "user logged in");  // Structured logging — user_name is a field
// Or sanitize:
log::info!("user {:?} logged in", user_name);  // Debug format escapes newlines
```

### HTML / XSS (if rendering to HTML)

Use templating that escapes by default (askama, tera, minijinja) and mark user content as text. Never `String::push_str` user content into HTML.

---

## 3. Authentication and authorization

### Check for
- [ ] Auth happens at a single layer (middleware, not scattered in handlers)
- [ ] Session tokens are cryptographically random (`rand::rngs::OsRng`)
- [ ] Passwords stored as salted hash (`argon2`, `bcrypt` — never SHA alone)
- [ ] JWT signature verified before use of claims
- [ ] JWT `exp` / `nbf` / `iat` claims validated
- [ ] RBAC / permissions checked on every mutation, not just reads
- [ ] Rate limiting on auth endpoints (login, signup, password reset)
- [ ] Password reset tokens single-use and time-limited
- [ ] No session fixation (new session ID on login)

### Password hashing

```rust
// GOOD — argon2
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

fn hash(pw: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2.hash_password(pw.as_bytes(), &salt)?.to_string())
}

fn verify(pw: &str, hash: &str) -> Result<bool> {
    let parsed = argon2::PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(pw.as_bytes(), &parsed).is_ok())
}
```

### JWT validation

```rust
// BAD — uses claims without verifying signature
let claims: Claims = serde_json::from_slice(&base64_decode(jwt.split('.').nth(1)?))?;

// GOOD — jsonwebtoken crate handles verification
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

let mut validation = Validation::new(Algorithm::HS256);
validation.set_audience(&["my-app"]);
validation.set_issuer(&["my-auth"]);
let token = decode::<Claims>(&jwt, &DecodingKey::from_secret(secret), &validation)?;
```

### Timing-safe comparison

For comparing secrets (tokens, HMACs), use constant-time comparison to prevent timing attacks:

```rust
use subtle::ConstantTimeEq;
if token.as_bytes().ct_eq(expected.as_bytes()).into() {
    // auth OK
}
// NOT: if token == expected { ... }  // Timing leak
```

---

## 4. Secrets management

### Check for
- [ ] No secrets in source code (no hardcoded API keys, passwords)
- [ ] No secrets in logs (use `secrecy::Secret<T>` wrapper)
- [ ] No secrets in error messages propagated to users
- [ ] No secrets in git history (use `git-secrets`, `trufflehog` in CI)
- [ ] Config secrets loaded from env / secret manager, not files in repo
- [ ] `.env` files gitignored
- [ ] Secrets rotated periodically (design rotation-compatible)

### `secrecy` crate

```rust
use secrecy::{Secret, ExposeSecret};

#[derive(Debug)]  // Secret<T> redacts in Debug output
pub struct Config {
    pub db_url: Secret<String>,
    pub jwt_secret: Secret<String>,
}

// Usage
fn connect(config: &Config) -> Result<Pool> {
    Pool::connect(config.db_url.expose_secret())  // Explicit "expose" call
}
```

`Debug` of a `Secret<T>` prints `"[REDACTED]"` — safe to log structures containing them.

---

## 5. Crypto primitives

### Use vetted implementations

| Need | Use |
|---|---|
| TLS | `rustls` (pure Rust) or `native-tls` (system) |
| HTTP client with TLS | `reqwest` with `rustls-tls` feature |
| Password hashing | `argon2` (preferred) or `bcrypt` |
| HMAC | `hmac` crate + `sha2` |
| AES-GCM | `aes-gcm` crate |
| Ed25519 | `ed25519-dalek` |
| X25519 (key exchange) | `x25519-dalek` |
| SHA-2/3 | `sha2`, `sha3` |
| AEAD | `chacha20poly1305` or `aes-gcm` |
| Secure random | `rand::rngs::OsRng` or `getrandom` |
| File encryption | `age` |
| General purpose | RustCrypto suite |

### Never
- Hand-roll crypto primitives
- Use MD5 or SHA-1 for security purposes
- Use `rand::thread_rng()` for cryptographic material (use `OsRng`)
- Use `ring` if you need FIPS compliance (ring is not FIPS-certified; use BoringSSL/wolfSSL)

### Secure randomness

```rust
use rand::rngs::OsRng;
use rand::RngCore;

let mut token = [0u8; 32];
OsRng.fill_bytes(&mut token);
```

---

## 6. Supply chain (dependency hygiene)

### Check for
- [ ] `cargo-audit` runs in CI (fails on known advisories)
- [ ] `cargo-deny` configured (bans / licenses / duplicates)
- [ ] Lockfile committed
- [ ] New deps reviewed for maintenance status (abandoned?)
- [ ] Transitively pulled deps reviewed for unusual access (random crate grabs network? file system?)
- [ ] Pinned version of any crate you'd be unhappy to auto-upgrade
- [ ] `cargo-vet` for org-wide audited deps (Mozilla-style review workflow)

### Example CI

```yaml
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-audit
      - run: cargo audit
  
  deny:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install cargo-deny
      - run: cargo deny check
```

### `deny.toml` skeleton

```toml
[advisories]
vulnerability = "deny"
unsound = "deny"
yanked = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-3.0"]

[bans]
multiple-versions = "warn"
deny = [
    # { name = "openssl", reason = "use rustls" },
]
```

---

## 7. Unsafe code review (security angle)

See [SKILL.md §7.6](SKILL.md#76-unsafe-and-ffi) for the general checklist. Security-specific concerns:

- **FFI with attacker-controlled input** — any unsafe processing untrusted data is a fuzz target
- **`String::from_utf8_unchecked` with user input** — security BLOCK
- **Raw pointer arithmetic in parsers** — audit carefully; use safe slicing
- **`Transmute` between public types** — review for invariant violations
- **`catch_unwind` missing at FFI boundary** — panic crossing = UB
- **`#[repr(C)]` type changes without coordinating with C side** — silent corruption

---

## 8. Logging discipline

- **Never log secrets** — tokens, passwords, API keys, PII
- **Use structured logging** (`tracing`) — easier to audit
- **Log auth failures but not auth successes at INFO** (too noisy; use DEBUG)
- **Avoid logging user-controlled strings as format** — log injection
- **Include request/trace IDs** in logs for correlation

```rust
// GOOD — structured, no secret in format
tracing::info!(
    user_id = %user_id,
    ip = %ip,
    "login succeeded"
);

// BAD — logs session token
tracing::info!("session created: {:?}", session);  // session.token logged!
```

---

## 9. HTTP-specific

### Check for
- [ ] TLS enforced (HTTPS); HTTP redirects to HTTPS
- [ ] HSTS header sent (`strict-transport-security`)
- [ ] CSP header set (prevents XSS scope)
- [ ] CORS configured specifically, not `*`
- [ ] Cookies: `Secure`, `HttpOnly`, `SameSite=Strict` or `Lax`
- [ ] Request body size limits (default axum: 2MB)
- [ ] Rate limiting middleware
- [ ] Timeouts on all external HTTP calls

### Axum body size limit

```rust
use axum::extract::DefaultBodyLimit;

let app = Router::new()
    .route("/upload", post(upload_handler))
    .layer(DefaultBodyLimit::max(10 * 1024 * 1024))  // 10 MB
```

### Headers middleware (axum + tower-http)

```rust
use tower_http::{set_header::SetResponseHeaderLayer, cors::CorsLayer};
use axum::http::{HeaderValue, header};

let app = Router::new()
    .route("/", get(|| async {}))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=63072000; includeSubDomains"),
    ))
    .layer(SetResponseHeaderLayer::if_not_present(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'"),
    ))
    .layer(CorsLayer::new().allow_origin("https://my-app.example".parse::<HeaderValue>().unwrap()));
```

---

## 10. Deserialization

Untrusted deserialization is a significant attack surface. Concerns:

- **Resource exhaustion** — attacker sends a 10GB JSON; parser allocates forever
  - Mitigate: body size limits, serde size limits, streaming parsers
- **Stack overflow** — deeply nested JSON/XML
  - Mitigate: serde-json has a default depth limit; check yours
- **Deserialization into `#[serde(tag)]`-typed enums** — attacker picks the variant; ensure all variants are safe
- **`#[serde(deny_unknown_fields)]`** — catches typos and prevents extra-field smuggling
- **`HashMap<String, ...>` deserialization** — attacker can cause collisions with weak hasher. Use `BTreeMap` or `ahash` for trusted-size caches.

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]   // Reject unknown fields
struct Config {
    name: String,
    // No extra fields accepted
}
```

---

## Related

- [rust-reviewing/SKILL.md §7.9](SKILL.md#79-security) — compact checklist
- [rust-planning/unsafe-strategy.md](../rust-planning/unsafe-strategy.md) — planning unsafe usage
- [rust-planning/workspace-layout.md](../rust-planning/workspace-layout.md) — dependency architecture
- [rust-implementing/web-apis.md](../rust-implementing/web-apis.md) — HTTP implementation patterns

## References

- [Rust Security Advisory Database](https://rustsec.org/)
- [cargo-audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [cargo-deny](https://embarkstudios.github.io/cargo-deny/)
- [cargo-vet](https://mozilla.github.io/cargo-vet/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/) — web security baseline
- [RustCrypto](https://github.com/RustCrypto) — vetted crypto crates
