# GoF Patterns → Rust Translations

A lookup table for "I'm reaching for a GoF pattern from my prior
language background — what does that look like in Rust?"

The table is organized by pattern. For each row:

- **Rust translation**: the idiomatic Rust shape, or "absent — language
  feature replaces it" with the replacement named.
- **Skill section**: where to read the full pattern in the rust-phase
  skills.
- **Production reference**: a major Rust crate (or community-canonical
  guide) that uses the pattern, for cross-checking.

**Five patterns are notably absent in idiomatic Rust** — the language
or standard library replaces them. They are listed at the end with the
replacement called out explicitly.

---

## Creational

| GoF pattern | Rust translation | Skill section | Production reference |
|---|---|---|---|
| **Factory Method** | Associated function on the type. `impl T { pub fn new(...) -> Self }`. Use `impl Into<String>` for ergonomic params. Cascading `from_str` for parsing. | rust-implementing/SKILL.md §"Type choice" rows on validated newtypes; [rust-unofficial/patterns *Constructor*](https://rust-unofficial.github.io/patterns/idioms/ctor.html) | Pervasive. `Vec::new()`, `String::from(...)`, `Box::new(...)`. |
| **Abstract Factory** | Trait with associated types: `trait Factory { type Item: Trait; type Error: std::error::Error; fn create(&self) -> Result<Self::Item, Self::Error>; }`. Compile-time guarantee that families pair correctly. | rust-implementing/type-system.md §"Validated newtypes + Builder" | sqlx connection pools (`PgPool` / `MySqlPool` / `SqlitePool` follow the same trait shape). |
| **Builder** | Each setter takes `mut self` returns `Self`. `build(self)` consumes, validates, returns owned. Use `Option<T>` per field to detect missing in `build()`. Wrap with TypeState for compile-time required-fields. | rust-implementing/type-system.md §"Type State Pattern" + §"Validated newtypes + Builder"; [Rust API Guidelines C-BUILDER](https://rust-lang.github.io/api-guidelines/type-safety.html); [rust-unofficial/patterns *Builder*](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html) | `idanarye/rust-typed-builder` (~6M dl), `elastio/bon` (~3M dl), `tokio::runtime::Builder`, `reqwest::ClientBuilder`. |
| **Singleton** | **Almost never used.** Replace with: `const` (compile-time), `OnceLock<T>` (lazy init, stable Rust 1.70+), `LazyLock<T>` (eager-on-first-access, stable Rust 1.80+), `Arc<T>` (shared ownership), explicit `with_config(...)` constructor. `unsafe`-based singletons with `static mut` are wrong. | rust-planning/SKILL.md rule 14 (no global mutable state for services); [Rust API Guidelines C-OBJECT](https://rust-lang.github.io/api-guidelines/) | `std::sync::OnceLock` / `LazyLock`. anti-patterns-catalog.md A8 catches the misuse. |
| **Prototype** | `#[derive(Clone)]` + `#[derive(Default)]` + `..Default::default()` struct update syntax. `Cow<T>` for clone-on-write. `Arc<T>` for shared template that's not modified per use. | rust-implementing/language-patterns.md §"Cow<T> — Clone on Write" | `std::clone::Clone`, `std::default::Default`. The trait machinery IS the pattern. |

## Structural

| GoF pattern | Rust translation | Skill section | Production reference |
|---|---|---|---|
| **Adapter** | Newtype wrapping a foreign type + impl your local trait on it (orphan-rule workaround). For simple type conversions, use `From` / `Into`. For state-bearing adapters, dedicated struct holding adaptee + config. | rust-implementing/language-patterns.md §"Trait Patterns / Extension Traits" + §"From/Into/AsRef Conversions" | `std::io::BufReader<R>` adapts any `Read` into a buffered `Read`; `tokio_util::compat` adapts std `Read`/`Write` to/from tokio. |
| **Bridge** | `Box<dyn Implementation>` field on the abstraction. Distinguish from Strategy: Bridge = two independent dimensions of variation; Strategy = interchangeable algorithms for one behavior. | rust-planning/architecture-patterns.md §"Architectural Patterns & Principles" | Tower's `Service<Request>` trait — one bridge between request types and service implementations. |
| **Composite** | `Box<dyn Trait>` children for heterogeneous tree. OR enum (closed set) — pick by extensibility need. Trait-object enables Decorator stacking on the tree. | rust-planning/architecture-patterns.md §"Enum-Based Polymorphism (vs `dyn Trait`)" | `syn::Expr` (enum AST nodes); axum `Router` (tree of routers via `nest`). |
| **Decorator** | `Box<dyn Trait>` field + impl same trait. Caching decorator needs `RefCell<Option<T>>` (interior mutability for cache write through `&self`). Compile-time alternative: generic + monomorphization (Tower Layer pattern). | rust-implementing/language-patterns.md §"Middleware / Decorator Pattern" | Tower `Layer<S>` (axum middleware stack); `BufReader<R>` decorating any `Read`. |
| **Facade** | Single struct owning subsystems (no lifetimes — owned subsystems). Or module-level via `pub use` in `lib.rs` / `prelude.rs`. | rust-planning/architecture-patterns.md §"Modules Become Interfaces" + §"Facade Crate Pattern" | apache/iggy `IggyClient` + `prelude.rs`. ripgrep `grep` crate re-exporting `grep-cli`, `grep-matcher`, `grep-printer`, etc. |
| **Flyweight** | **Mostly absent — language features replace.** `const`, `static`, `OnceLock`, `Arc<T>`, `Cow<T>`, `&'static str`. Manual factory rarely needed. | rust-implementing/language-patterns.md §"Cow<T> — Clone on Write" | String interning via `Arc<str>` or `string-interner` crate; `&'static str` literals deduplicated by the linker. |
| **Proxy** | **Mostly absent — language features replace.** `LazyCell` / `LazyLock` for lazy init; `pub` / `pub(crate)` for access control; `Box` / `Rc` / `Arc` for ownership proxying; `Deref` for transparent forwarding (smart pointers only — see anti-patterns A1). Custom proxy needed only for cross-cutting concerns; then it looks like a Decorator. | rust-implementing/language-patterns.md §"Smart Pointers" | `std::sync::OnceLock`, `std::cell::LazyCell`. |

## Behavioural

| GoF pattern | Rust translation | Skill section | Production reference |
|---|---|---|---|
| **Chain of Responsibility** | `Vec<Box<dyn Handler>>`. Handler returns `Option<Result<...>>` for three-way semantics: handled / failed / declined. Order matters (most specific first). | rust-implementing/language-patterns.md §"Pipeline of fallible stages" (related shape) | Tower `ServiceBuilder` middleware chain; axum `Router` nested routes. |
| **Command** | Trait with `execute(&mut self)` (saves prev state) and `undo(&self)` (uses saved state). Dual-stack `history: Vec<Box<dyn Command>>` and `undo_stack`. Each command carries its own saved state. | [rust-unofficial/patterns *Command*](https://rust-unofficial.github.io/patterns/patterns/behavioural/command.html) | Editor commands in `helix-editor`, `zed`. |
| **Iterator** | Built-in `Iterator` trait. Custom iterators for non-standard structures (trees: stack-based DFS). `DoubleEndedIterator` for bidirectional. `IntoIterator` for `for` loop integration. | rust-implementing/SKILL.md §"Iterators & Closures"; rust-implementing/language-patterns.md §"Building Custom Iterators" | `std::iter`, `itertools` (~250M dl). |
| **Mediator** | Components hold `Arc<Mutex<dyn Mediator>>`. Mediator holds `Option<Arc<...>>` for each component. Notify takes event enum. Lock-scope discipline: keep lock duration minimal. | rust-implementing/async-patterns.md §"Actor patterns" (related shape — actor is one common form) | `actix`, `xtra`, `ractor` actor frameworks. |
| **Memento** | Originator + Memento (private constructor) + Caretaker. Store strategy by NAME (not trait object) — recreate via factory on restore. `serde` derives → free serialization persistence. | rust-implementing/serde-patterns.md (serialization shape) | git-style content-addressable storage in `gix` (gitoxide); session save/restore in editors. |
| **Observer** | Trait with `update(&self, event: &Event)` — `&self` not `&mut self` (subject calls notify(&self) and iterates observers immutably). State-needing observers wrap specific fields in `Arc<Mutex<>>` internally. `HashMap<usize, Box<dyn Observer>>` keyed by ID for O(1) detach. Channels (`mpsc`) are the modern alternative. | rust-implementing/async-patterns.md §"Channel patterns" (the modern shape) | `tokio::sync::broadcast` for one-to-many; `tokio::sync::watch` for latest-value. |
| **State** | TypeState (compile-time, see rust-implementing/type-system.md) when transitions known at compile time. Otherwise: `Box<dyn State>` field, transitions returned as `enum InputResult { ChangeMode(Box<dyn State>), … }` data — see rust-implementing/type-system.md §"State transitions returned as data". | rust-implementing/type-system.md §"Type State Pattern" + §"State transitions returned as data" | `embedded-hal` GPIO mode transitions (`pin.into_push_pull_output()`); `idanarye/rust-typed-builder` field tracking. |
| **Strategy** | Trait + `Box<dyn Strategy>` field for runtime swap, OR generic + monomorphization for compile-time. `set_strategy()` for runtime mode change. | rust-planning/architecture-patterns.md §"Trait-Based Dependency Inversion"; [rust-unofficial/patterns *Strategy*](https://rust-unofficial.github.io/patterns/patterns/behavioural/strategy.html) | sqlx `Database` trait with `Postgres` / `MySql` / `Sqlite` impls; `std::collections::HashMap`'s `BuildHasher` parameter. |
| **Template Method** | Trait with default implementation of orchestrating method that calls typed steps; some steps have defaults, others required. Override default with `<Self as Trait>::method(self, args)` syntax to extend not replace. | rust-implementing/type-system.md §"Trait Patterns" | `std::iter::Iterator` itself (default `sum`, `product`, `collect` over `next`); `serde::Serializer` skeleton. |
| **Visitor** | Two traits: `Visitor` (one method per element type) + `Visitable::accept(&mut visitor)`. Double dispatch. Visit children before parent for bottom-up. Optimizer doesn't mutate in place — builds replacement map. | [rust-unofficial/patterns *Visitor*](https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html) | `syn::visit::Visit`, `syn::visit_mut::VisitMut` for AST traversal in proc macros. |

## Patterns absent in idiomatic Rust

The following GoF patterns are usually replaced by language features
or stdlib primitives. Listed for completeness so prior-language
intuition doesn't lead to over-engineering.

| Pattern | Replaced by | Why |
|---|---|---|
| **Singleton** | `OnceLock` / `LazyLock` for lazy init; `const` / `static` for compile-time; `Arc<T>` + explicit construction for shared ownership | Singletons trade testability for global access. Rust prefers explicit injection (rust-planning rule 14). The legitimate "lazy init once" case is `OnceLock<T>`. |
| **Prototype** | `Clone` + `Default` traits, `..Default::default()` struct update syntax, `Cow<T>` | The whole trait machinery handles "create new from existing template" without a separate pattern. |
| **Flyweight** | `Arc<T>`, `Cow<T>`, `&'static str`, `OnceLock<T>`, `string-interner` for interning | Reference counting and string interning are stdlib-level concerns; manual flyweight factories are unnecessary. |
| **Proxy** | `LazyCell` / `LazyLock` for lazy proxy; `pub(crate)` / `pub(super)` for access control; `Box`/`Rc`/`Arc` for ownership proxy; `Deref` for transparent forwarding (smart pointers only) | The four common proxy use cases each have a more specific stdlib mechanism. |
| **Iterator (as a "pattern" requiring a class)** | Built-in `Iterator` trait, `for` loop syntax, `IntoIterator` | Iteration is a language feature, not a pattern users implement from scratch. Custom iterators are still useful (rust-implementing/language-patterns.md §"Building Custom Iterators") but as trait impls, not classes. |

The Singleton case is worth elaborating. The temptation is to write:

```rust
// BAD — see anti-patterns-catalog.md A8
lazy_static! { static ref CONFIG: Mutex<Config> = Mutex::new(...); }
```

The right shape, when you legitimately need a single-instance lazy
value, is:

```rust
use std::sync::OnceLock;
static CONFIG: OnceLock<Config> = OnceLock::new();
let cfg = CONFIG.get_or_init(load_config);
```

For *mutable* shared state, the entire question is wrong — see
rust-planning rules 7b and 14.

---

## How to use this table

1. **Coming from another language and reaching for a known pattern?**
   Look up the pattern here, follow the "Skill section" link to the
   canonical Rust shape.
2. **Reviewing code that "smells like a GoF pattern"?** Look up the
   pattern, check whether the code matches the idiomatic Rust
   translation. Anti-patterns catalog (especially A1 Deref-as-inheritance
   and A2 generics-as-base-class) catches common mis-translations.
3. **Designing a new system?** This table is *not* a recommendation to
   reach for GoF patterns. Many Rust designs work better when modeled
   from data flow + ownership rather than from an OO pattern catalog
   (see rust-planning rule 7a). Use this table to translate when a GoF
   pattern is genuinely the right fit, not as a starting point.

## References

- [rust-unofficial/patterns](https://rust-unofficial.github.io/patterns/) —
  community-curated Rust pattern book. The canonical Rust pattern
  reference; this table cross-references it heavily.
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) —
  C-CTOR, C-CUSTOM-TYPE, C-NEWTYPE, C-BUILDER, C-DEREF, C-OBJECT
  guidelines map directly onto several of the rows above.
- The Gang-of-Four book itself (*Design Patterns*, Gamma et al., 1994)
  — for the original pattern definitions. Most of what you read there
  about C++/Smalltalk-era OO does not translate directly to Rust;
  treat it as historical context for the names and intents, not as a
  Rust implementation guide.
