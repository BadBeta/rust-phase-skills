# Rust WebAssembly Frontend Frameworks

> **Version**: 2025
> **Status**: Complete Reference

## Table of Contents
1. [Framework Overview](#1-framework-overview)
2. [Leptos Deep Dive](#2-leptos-deep-dive)
3. [Yew Deep Dive](#3-yew-deep-dive)
4. [Dioxus Deep Dive](#4-dioxus-deep-dive)
5. [Sycamore Overview](#5-sycamore-overview)
6. [Framework Comparison](#6-framework-comparison)
7. [Routing](#7-routing)
8. [State Management](#8-state-management)
9. [Forms & Validation](#9-forms--validation)
10. [Async & Data Loading](#10-async--data-loading)
11. [Patterns](#11-patterns)
12. [Anti-Patterns](#12-anti-patterns)
13. [Common Failures & Solutions](#13-common-failures--solutions)
14. [Quick Reference](#14-quick-reference)

---

## 1. Framework Overview

### 1.1 Framework Comparison Matrix

| Feature | Leptos | Yew | Dioxus | Sycamore |
|---------|--------|-----|--------|----------|
| **Reactivity** | Fine-grained (signals) | Virtual DOM | Virtual DOM + signals | Fine-grained |
| **SSR Support** | Excellent | Limited | Good | Good |
| **Hydration** | Built-in | Manual | Built-in | Built-in |
| **Desktop** | Via Tauri | No | Native | No |
| **Mobile** | No | No | Experimental | No |
| **GitHub Stars** | 18.5k+ | 30.5k+ | 25k+ | 3.2k+ |
| **Bundle Size** | Smallest | Larger | Medium | Small |
| **Learning Curve** | Moderate | Easy (React-like) | Easy (React-like) | Moderate |

### 1.2 When to Choose Each Framework

**Choose Leptos when:**
- Maximum performance is critical
- SSR with streaming/hydration is needed
- You want fine-grained reactivity without VDOM overhead
- Building full-stack Rust applications

**Choose Yew when:**
- Coming from React background
- Building client-side SPAs
- Large ecosystem/community is important
- Virtual DOM model is preferred

**Choose Dioxus when:**
- Building cross-platform apps (web + desktop + mobile)
- Sharing code between platforms
- React-like DX with better performance
- Native desktop apps are a priority

**Choose Sycamore when:**
- Lightweight solution needed
- Fine-grained reactivity preferred
- Simpler API desired
- Building smaller applications

---

## 2. Leptos Deep Dive

### 2.1 Getting Started

```bash
# Install cargo-leptos
cargo install cargo-leptos

# Create new project (with Axum backend)
cargo leptos new my-app
cd my-app

# Run development server
cargo leptos watch
```

### 2.2 Reactive Primitives

#### Signals
```rust
use leptos::prelude::*;

#[component]
fn Counter() -> impl IntoView {
    // Create a signal (getter, setter pair)
    let (count, set_count) = signal(0);

    // Or use RwSignal for combined read/write
    let count = RwSignal::new(0);

    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>
            "Count: " {count}
        </button>
    }
}
```

#### Signal Methods
```rust
let (value, set_value) = signal(0);

// Reading
value.get()           // Clone and track
value.get_untracked() // Clone without tracking
value.read()          // Borrow and track (returns guard)

// Writing
set_value.set(5)              // Set new value
set_value.update(|v| *v += 1) // Mutate in place
set_value.write()             // Get mutable reference (guard)
```

#### Memos (Derived Computations)
```rust
#[component]
fn DoubleCounter() -> impl IntoView {
    let (count, set_count) = signal(0);

    // Memo only recomputes when dependencies change
    let doubled = Memo::new(move |_| count.get() * 2);

    // For simple derivations, use derived signals instead
    let tripled = move || count.get() * 3;

    view! {
        <p>"Count: " {count}</p>
        <p>"Doubled (memo): " {doubled}</p>
        <p>"Tripled (derived): " {tripled}</p>
    }
}
```

#### Effects
```rust
#[component]
fn EffectExample() -> impl IntoView {
    let (count, set_count) = signal(0);

    // Effect runs when dependencies change
    Effect::new(move |_| {
        // This runs on every count change
        log::info!("Count changed to: {}", count.get());
    });

    // Effect with cleanup
    Effect::new(move |_| {
        let interval = set_interval(/* ... */);

        // Return cleanup function
        on_cleanup(move || {
            clear_interval(interval);
        });
    });

    view! { /* ... */ }
}
```

### 2.3 Components

#### Basic Component with Props
```rust
#[component]
fn Greeting(
    name: String,
    #[prop(optional)] greeting: Option<String>,
    #[prop(default = "!".to_string())] punctuation: String,
) -> impl IntoView {
    let greeting = greeting.unwrap_or_else(|| "Hello".to_string());

    view! {
        <p>{greeting} ", " {name} {punctuation}</p>
    }
}

// Usage
view! {
    <Greeting name="World"/>
    <Greeting name="Rust" greeting="Greetings".to_string()/>
}
```

#### Generic Components
```rust
#[component]
fn List<T>(
    items: Vec<T>,
    render_item: impl Fn(T) -> impl IntoView + 'static,
) -> impl IntoView
where
    T: Clone + 'static,
{
    view! {
        <ul>
            {items.into_iter().map(render_item).collect_view()}
        </ul>
    }
}
```

#### Children
```rust
#[component]
fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="card">
            <div class="card-body">
                {children()}
            </div>
        </div>
    }
}

// For callable children
#[component]
fn Wrapper(children: ChildrenFn) -> impl IntoView {
    view! {
        <div>{children()}</div>
        <div>{children()}</div> // Can call multiple times
    }
}
```

### 2.4 Parent-Child Communication

```rust
// Method 1: Pass WriteSignal as prop
#[component]
fn Child1(setter: WriteSignal<i32>) -> impl IntoView {
    view! {
        <button on:click=move |_| setter.update(|n| *n += 1)>
            "Increment"
        </button>
    }
}

// Method 2: Pass callback
#[component]
fn Child2(on_click: impl Fn() + 'static) -> impl IntoView {
    view! {
        <button on:click=move |_| on_click()>
            "Click me"
        </button>
    }
}

// Method 3: Context
#[component]
fn Parent() -> impl IntoView {
    let (count, set_count) = signal(0);
    provide_context(set_count);

    view! {
        <Child3/>
        <p>"Count: " {count}</p>
    }
}

#[component]
fn Child3() -> impl IntoView {
    let set_count = expect_context::<WriteSignal<i32>>();

    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>
            "Increment via context"
        </button>
    }
}
```

### 2.5 Error Handling

```rust
use leptos::prelude::*;

#[component]
fn ValidatedInput() -> impl IntoView {
    let (value, set_value) = signal(Ok(0i32));

    view! {
        <input
            type="text"
            on:input:target=move |ev| {
                set_value.set(ev.target().value().parse::<i32>())
            }
        />

        <ErrorBoundary
            fallback=|errors| view! {
                <div class="error">
                    <p>"Errors:"</p>
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                        }
                    </ul>
                </div>
            }
        >
            <p>"Valid number: " {value}</p>
        </ErrorBoundary>
    }
}
```

---

## 3. Yew Deep Dive

### 3.1 Getting Started

```bash
# Add wasm target
rustup target add wasm32-unknown-unknown

# Install Trunk
cargo install trunk

# Create project
cargo new yew-app
cd yew-app

# Add dependencies to Cargo.toml
# [dependencies]
# yew = { version = "0.21", features = ["csr"] }

# Create index.html and run
trunk serve
```

### 3.2 Function Components with Hooks

#### use_state
```rust
use yew::prelude::*;

#[function_component]
fn Counter() -> Html {
    let counter = use_state(|| 0);

    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| counter.set(*counter + 1))
    };

    html! {
        <div>
            <button {onclick}>{ "Increment" }</button>
            <p>{ format!("Count: {}", *counter) }</p>
        </div>
    }
}
```

#### use_effect
```rust
#[function_component]
fn EffectDemo() -> Html {
    let counter = use_state(|| 0);

    // Run on every render
    use_effect(|| {
        log::info!("Component rendered");
        || () // Cleanup (optional)
    });

    // Run only when dependencies change
    {
        let counter = counter.clone();
        use_effect_with(*counter, move |count| {
            log::info!("Counter changed to: {}", count);
            || ()
        });
    }

    html! { /* ... */ }
}
```

#### use_reducer
```rust
use std::rc::Rc;

enum CounterAction {
    Increment,
    Decrement,
    Reset,
}

#[derive(Default, Clone, PartialEq)]
struct CounterState {
    count: i32,
}

impl Reducible for CounterState {
    type Action = CounterAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            CounterAction::Increment => Self { count: self.count + 1 }.into(),
            CounterAction::Decrement => Self { count: self.count - 1 }.into(),
            CounterAction::Reset => Self::default().into(),
        }
    }
}

#[function_component]
fn ReducerCounter() -> Html {
    let state = use_reducer(CounterState::default);

    let increment = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(CounterAction::Increment))
    };

    html! {
        <div>
            <p>{ state.count }</p>
            <button onclick={increment}>{ "+1" }</button>
        </div>
    }
}
```

#### use_context
```rust
#[derive(Clone, PartialEq)]
struct Theme {
    dark: bool,
}

#[function_component]
fn App() -> Html {
    let theme = use_state(|| Theme { dark: false });

    html! {
        <ContextProvider<Theme> context={(*theme).clone()}>
            <ThemedButton/>
        </ContextProvider<Theme>>
    }
}

#[function_component]
fn ThemedButton() -> Html {
    let theme = use_context::<Theme>().expect("Theme context not found");

    let class = if theme.dark { "btn-dark" } else { "btn-light" };

    html! {
        <button class={class}>{ "Themed Button" }</button>
    }
}
```

### 3.3 Components with Properties

```rust
#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub label: String,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or(Callback::noop())]
    pub onclick: Callback<MouseEvent>,
}

#[function_component]
fn Button(props: &ButtonProps) -> Html {
    html! {
        <button
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { &props.label }
        </button>
    }
}

// Usage
html! {
    <Button
        label="Click me"
        onclick={Callback::from(|_| log::info!("Clicked!"))}
    />
}
```

### 3.4 Agents (Web Workers)

```rust
use yew_agent::prelude::*;

#[derive(Debug, PartialEq, Eq)]
pub struct WorkerInput(pub u64);

#[derive(Debug, PartialEq, Eq)]
pub struct WorkerOutput(pub u64);

#[reactor]
pub async fn FibonacciWorker(mut scope: ReactorScope<WorkerInput, WorkerOutput>) {
    while let Some(input) = scope.next().await {
        fn fib(n: u64) -> u64 {
            if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
        }

        let result = fib(input.0);
        scope.send(WorkerOutput(result));
    }
}

// In component
#[function_component]
fn FibCalculator() -> Html {
    let result = use_state(|| None);
    let worker = use_reactor::<FibonacciWorker, _>({
        let result = result.clone();
        move |output| result.set(Some(output.0))
    });

    html! {
        <div>
            <button onclick={move |_| worker.send(WorkerInput(40))}>
                { "Calculate Fib(40)" }
            </button>
            if let Some(r) = *result {
                <p>{ format!("Result: {}", r) }</p>
            }
        </div>
    }
}
```

---

## 4. Dioxus Deep Dive

### 4.1 Getting Started

```bash
# Install dx CLI
cargo install dioxus-cli

# Create new project
dx new my-app
cd my-app

# Serve (web)
dx serve

# Build for desktop
dx build --platform desktop
```

### 4.2 RSX Syntax

```rust
use dioxus::prelude::*;

fn App() -> Element {
    rsx! {
        div {
            class: "container",
            h1 { "Hello Dioxus!" }

            // Conditional rendering
            if true {
                p { "This is shown" }
            }

            // Iteration
            for i in 0..5 {
                p { "Item {i}" }
            }

            // Components
            Button { label: "Click me" }
        }
    }
}

#[component]
fn Button(label: String) -> Element {
    rsx! {
        button { "{label}" }
    }
}
```

### 4.3 State with Signals

```rust
fn Counter() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| count += 1,
            "Count: {count}"
        }
    }
}
```

### 4.4 Hooks

```rust
fn HooksDemo() -> Element {
    // State
    let mut name = use_signal(|| String::new());

    // Memoization
    let greeting = use_memo(move || format!("Hello, {}!", name()));

    // Effect
    use_effect(move || {
        log::info!("Name changed to: {}", name());
    });

    // Future/Resource
    let user = use_resource(move || async move {
        fetch_user(name()).await
    });

    rsx! {
        input {
            value: "{name}",
            oninput: move |e| name.set(e.value()),
        }
        p { "{greeting}" }
    }
}
```

### 4.5 Cross-Platform

```rust
// Shared component works on all platforms
#[component]
fn SharedComponent() -> Element {
    rsx! {
        div { "Works everywhere!" }
    }
}

// Platform-specific code
#[cfg(feature = "web")]
fn platform_init() {
    // Web-specific initialization
}

#[cfg(feature = "desktop")]
fn platform_init() {
    // Desktop-specific initialization
}
```

---

## 5. Sycamore Overview

### 5.1 Basic Usage

```rust
use sycamore::prelude::*;

fn main() {
    sycamore::render(App);
}

#[component]
fn App() -> View {
    let count = create_signal(0);

    view! {
        button(on:click=move |_| count.set(*count.get() + 1)) {
            "Count: " (count.get())
        }
    }
}
```

### 5.2 Reactivity

```rust
#[component]
fn ReactiveDemo() -> View {
    let name = create_signal(String::new());

    // Derived value
    let greeting = create_memo(move || {
        format!("Hello, {}!", name.get())
    });

    // Effect
    create_effect(move || {
        log::info!("Name is now: {}", name.get());
    });

    view! {
        input(bind:value=name)
        p { (greeting.get()) }
    }
}
```

### 5.3 Two-Way Binding

```rust
#[component]
fn FormDemo() -> View {
    let text = create_signal(String::new());
    let checked = create_signal(false);

    view! {
        // Two-way binding with bind:
        input(bind:value=text)
        input(type="checkbox", bind:checked=checked)

        p { "Text: " (text.get()) }
        p { "Checked: " (if *checked.get() { "Yes" } else { "No" }) }
    }
}
```

---

## 6. Framework Comparison

### 6.1 Syntax Comparison

**Counter Example Across Frameworks:**

```rust
// LEPTOS
#[component]
fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0);
    view! {
        <button on:click=move |_| set_count.update(|n| *n += 1)>
            {count}
        </button>
    }
}

// YEW
#[function_component]
fn Counter() -> Html {
    let count = use_state(|| 0);
    let onclick = {
        let count = count.clone();
        Callback::from(move |_| count.set(*count + 1))
    };
    html! {
        <button {onclick}>{ *count }</button>
    }
}

// DIOXUS
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        button { onclick: move |_| count += 1, "{count}" }
    }
}

// SYCAMORE
#[component]
fn Counter() -> View {
    let count = create_signal(0);
    view! {
        button(on:click=move |_| count.set(*count.get() + 1)) {
            (count.get())
        }
    }
}
```

### 6.2 Performance Benchmarks (js-framework-benchmark)

| Metric | Leptos | Dioxus | Yew | Sycamore | React |
|--------|--------|--------|-----|----------|-------|
| Create 1k rows | 1.05x | 1.08x | 1.35x | 1.12x | 1.50x |
| Update 1k rows | 1.02x | 1.06x | 1.28x | 1.08x | 1.40x |
| Partial update | 1.01x | 1.04x | 1.45x | 1.05x | 1.60x |
| Select row | 1.00x | 1.02x | 1.20x | 1.03x | 1.30x |
| Memory (MB) | 2.8 | 3.2 | 4.5 | 2.9 | 5.2 |

*Note: Lower is better. 1.00x = vanilla JS baseline*

---

## 7. Routing

### 7.1 Leptos Router

```rust
use leptos::prelude::*;
use leptos_router::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <nav>
                <A href="/">"Home"</A>
                <A href="/about">"About"</A>
                <A href="/users/1">"User 1"</A>
            </nav>
            <main>
                <Routes fallback=|| "Not found">
                    <Route path="/" view=Home/>
                    <Route path="/about" view=About/>
                    <Route path="/users/:id" view=UserProfile/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.with(|p| p.get("id").unwrap_or_default());

    view! {
        <h1>"User: " {id}</h1>
    }
}
```

### 7.2 Yew Router

```rust
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/users/:id")]
    User { id: u64 },
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Home/> },
        Route::About => html! { <About/> },
        Route::User { id } => html! { <User {id}/> },
        Route::NotFound => html! { <h1>{"404"}</h1> },
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch}/>
        </BrowserRouter>
    }
}
```

### 7.3 Dioxus Router

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
    #[route("/users/:id")]
    User { id: u64 },
}

fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Home() -> Element {
    rsx! { h1 { "Home" } }
}

#[component]
fn User(id: u64) -> Element {
    rsx! { h1 { "User {id}" } }
}
```

---

## 8. State Management

### 8.1 Leptos Context

```rust
#[derive(Clone)]
struct AppState {
    user: RwSignal<Option<User>>,
    theme: RwSignal<Theme>,
}

#[component]
fn App() -> impl IntoView {
    let state = AppState {
        user: RwSignal::new(None),
        theme: RwSignal::new(Theme::Light),
    };

    provide_context(state);

    view! {
        <Router>/* ... */</Router>
    }
}

#[component]
fn Profile() -> impl IntoView {
    let state = expect_context::<AppState>();
    let user = state.user;

    view! {
        {move || user.get().map(|u| view! { <p>{u.name}</p> })}
    }
}
```

### 8.2 Yewdux (External State)

```rust
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Store)]
struct AppStore {
    count: i32,
    user: Option<User>,
}

#[function_component]
fn Counter() -> Html {
    let (store, dispatch) = use_store::<AppStore>();

    let increment = dispatch.reduce_mut_callback(|store| store.count += 1);

    html! {
        <button onclick={increment}>
            { store.count }
        </button>
    }
}
```

---

## 9. Forms & Validation

### 9.1 Leptos Forms

```rust
#[component]
fn LoginForm() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(None::<String>);

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if email.get().is_empty() {
            set_error.set(Some("Email required".into()));
            return;
        }

        // Submit logic...
    };

    view! {
        <form on:submit=on_submit>
            <input
                type="email"
                prop:value=email
                on:input:target=move |ev| set_email.set(ev.target().value())
            />
            <input
                type="password"
                prop:value=password
                on:input:target=move |ev| set_password.set(ev.target().value())
            />

            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}

            <button type="submit">"Login"</button>
        </form>
    }
}
```

### 9.2 Controlled Inputs Pattern

```rust
#[component]
fn ControlledInput(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: Option<String>,
) -> impl IntoView {
    view! {
        <input
            type="text"
            prop:value=value
            placeholder=placeholder.unwrap_or_default()
            on:input:target=move |ev| value.set(ev.target().value())
        />
    }
}
```

---

## 10. Async & Data Loading

### 10.1 Leptos Resources & Suspense

```rust
#[component]
fn UserList() -> impl IntoView {
    let users = Resource::new(
        || (),
        |_| async move {
            fetch_users().await
        }
    );

    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || users.get().map(|data| {
                data.into_iter()
                    .map(|user| view! { <p>{user.name}</p> })
                    .collect_view()
            })}
        </Suspense>
    }
}
```

### 10.2 Leptos Server Functions

```rust
#[server(GetUser)]
pub async fn get_user(id: u64) -> Result<User, ServerFnError> {
    // This runs on the server
    let user = db::get_user(id).await?;
    Ok(user)
}

#[component]
fn UserProfile() -> impl IntoView {
    let params = use_params_map();

    let user = Resource::new(
        move || params.get().get("id").map(|s| s.parse::<u64>().ok()),
        |id| async move {
            match id.flatten() {
                Some(id) => get_user(id).await.ok(),
                None => None,
            }
        }
    );

    view! {
        <Suspense fallback=|| "Loading...">
            {move || user.get().flatten().map(|u| view! { <h1>{u.name}</h1> })}
        </Suspense>
    }
}
```

### 10.3 Yew Async

```rust
use gloo_net::http::Request;

#[function_component]
fn DataLoader() -> Html {
    let data = use_state(|| None);
    let loading = use_state(|| true);

    {
        let data = data.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let response = Request::get("/api/data")
                    .send()
                    .await
                    .unwrap()
                    .json::<Data>()
                    .await
                    .unwrap();

                data.set(Some(response));
                loading.set(false);
            });
            || ()
        });
    }

    if *loading {
        html! { <p>{"Loading..."}</p> }
    } else {
        html! { <pre>{ format!("{:?}", *data) }</pre> }
    }
}
```

---

## 11. Patterns

### Pattern 1: Component Composition

```rust
// Leptos - Slots pattern
#[slot]
struct CardHeader {
    children: Children,
}

#[slot]
struct CardBody {
    children: Children,
}

#[component]
fn Card(
    card_header: CardHeader,
    card_body: CardBody,
) -> impl IntoView {
    view! {
        <div class="card">
            <div class="card-header">{card_header.children()}</div>
            <div class="card-body">{card_body.children()}</div>
        </div>
    }
}

// Usage
view! {
    <Card>
        <CardHeader slot>"Title"</CardHeader>
        <CardBody slot><p>"Content"</p></CardBody>
    </Card>
}
```

### Pattern 2: Render Props

```rust
#[component]
fn DataProvider<T, F, V>(
    fetch: impl Fn() -> Resource<(), T> + 'static,
    children: F,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(T) -> V + 'static,
    V: IntoView,
{
    let data = fetch();

    view! {
        <Suspense fallback=|| "Loading...">
            {move || data.get().map(|d| children(d))}
        </Suspense>
    }
}
```

### Pattern 3: Higher-Order Components

```rust
// Leptos HOC for authentication
fn with_auth<F, V>(component: F) -> impl Fn() -> impl IntoView
where
    F: Fn() -> V + Clone + 'static,
    V: IntoView,
{
    move || {
        let auth = expect_context::<AuthState>();

        view! {
            <Show
                when=move || auth.is_authenticated.get()
                fallback=|| view! { <Redirect path="/login"/> }
            >
                {component()}
            </Show>
        }
    }
}
```

### Pattern 4: Custom Hooks (Yew)

```rust
#[hook]
fn use_local_storage<T: Serialize + DeserializeOwned + Clone + 'static>(
    key: &'static str,
    default: T,
) -> (UseStateHandle<T>, Callback<T>) {
    let state = use_state(|| {
        window()
            .local_storage()
            .ok()
            .flatten()
            .and_then(|storage| storage.get_item(key).ok().flatten())
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or(default)
    });

    let set_value = {
        let state = state.clone();
        Callback::from(move |value: T| {
            if let Ok(json) = serde_json::to_string(&value) {
                let _ = window()
                    .local_storage()
                    .ok()
                    .flatten()
                    .map(|s| s.set_item(key, &json));
            }
            state.set(value);
        })
    };

    (state, set_value)
}
```

### Pattern 5: Optimistic Updates

```rust
#[component]
fn TodoItem(todo: Todo, on_toggle: Callback<u64>) -> impl IntoView {
    let (optimistic_done, set_optimistic) = signal(todo.done);

    let toggle = move |_| {
        // Optimistically update UI
        set_optimistic.update(|d| *d = !*d);

        // Trigger actual update
        on_toggle.call(todo.id);
    };

    view! {
        <li class:done=optimistic_done on:click=toggle>
            {todo.title}
        </li>
    }
}
```

---

## 12. Anti-Patterns

### Anti-Pattern 1: Excessive Cloning

```rust
// BAD: Cloning signal handle unnecessarily
let onclick = {
    let count = count.clone(); // Unnecessary!
    let count2 = count.clone();
    move |_| { /* ... */ }
};

// GOOD: Signals are Copy, no clone needed
let onclick = move |_| {
    count.update(|n| *n += 1);
};
```

### Anti-Pattern 2: Blocking Main Thread

```rust
// BAD: Blocking computation in component
#[component]
fn BadComponent() -> impl IntoView {
    let result = expensive_computation(); // Blocks render!
    view! { <p>{result}</p> }
}

// GOOD: Use Resource for async work
#[component]
fn GoodComponent() -> impl IntoView {
    let result = Resource::new(|| (), |_| async {
        expensive_computation_async().await
    });

    view! {
        <Suspense fallback=|| "Computing...">
            {move || result.get()}
        </Suspense>
    }
}
```

### Anti-Pattern 3: Over-Memoization

```rust
// BAD: Memo for simple derivation
let doubled = Memo::new(move |_| count.get() * 2);

// GOOD: Use derived signal for simple cases
let doubled = move || count.get() * 2;

// Memos are only needed when:
// 1. Computation is expensive
// 2. Result is used in multiple places
// 3. You need reference equality
```

### Anti-Pattern 4: Fighting the Framework

```rust
// BAD: Manual DOM manipulation
use web_sys::window;
let doc = window().unwrap().document().unwrap();
doc.get_element_by_id("my-el").unwrap()
    .set_inner_html("Hello");

// GOOD: Let the framework manage DOM
let (text, set_text) = signal("Hello");
view! { <div id="my-el">{text}</div> }
```

### Anti-Pattern 5: Prop Drilling

```rust
// BAD: Passing props through many layers
<A data=data>
    <B data=data>
        <C data=data>
            <D data=data/>  // Finally used here!
        </C>
    </B>
</A>

// GOOD: Use context
provide_context(data);
// In D:
let data = expect_context::<Data>();
```

---

## 13. Common Failures & Solutions

### Failure 1: Hydration Mismatch

```
Error: Hydration mismatch at element <div>
```

**Causes:**
- Server renders different HTML than client
- Using `cfg!(target_arch = "wasm32")` for conditional rendering
- Browser inserting elements (e.g., `<tbody>`)

**Solutions:**
```rust
// Use Effect for client-only code
Effect::new(move |_| {
    // This only runs on client
});

// Or use LocalResource
let data = LocalResource::new(|| async {
    // Client-only async work
});
```

### Failure 2: Signal Disposed

```
Attempted to access disposed signal
```

**Cause:** Accessing a signal after its owning scope is destroyed.

**Solution:**
```rust
// Store signal in longer-lived scope
let (global_count, set_global_count) = signal(0);
provide_context(set_global_count);

// Or check if valid
if let Some(value) = count.try_get() {
    // Use value
}
```

### Failure 3: Yew Re-render Loop

```
Component re-rendering infinitely
```

**Cause:** Updating state inside render without conditions.

**Solution:**
```rust
// BAD
let count = use_state(|| 0);
count.set(*count + 1); // Infinite loop!

// GOOD: Use effect with dependencies
use_effect_with(some_dep, move |_| {
    count.set(*count + 1);
    || ()
});
```

### Failure 4: Missing PartialEq

```
error: the trait bound `MyType: PartialEq` is not satisfied
```

**Solution:**
```rust
#[derive(Clone, PartialEq)]
struct MyType {
    // fields...
}

// For complex types, implement manually
impl PartialEq for MyType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
```

### Failure 5: SSR Feature Flag Issues

```
the crate `mio` cannot be compiled for `wasm32-unknown-unknown`
```

**Solution:**
```toml
# Cargo.toml
[features]
ssr = ["dep:tokio", "dep:sqlx"]

[dependencies]
tokio = { version = "1", optional = true }
sqlx = { version = "0.7", optional = true }
```

---

## 14. Quick Reference

### Leptos Cheatsheet

```rust
// Signals
let (get, set) = signal(initial);
let rw = RwSignal::new(initial);

// Memo
let derived = Memo::new(move |_| /* computation */);

// Effect
Effect::new(move |_| { /* side effect */ });

// Resource
let data = Resource::new(source, fetcher);

// Component
#[component]
fn MyComp(#[prop(optional)] name: Option<String>) -> impl IntoView { }

// Context
provide_context(value);
let ctx = expect_context::<T>();
```

### Yew Cheatsheet

```rust
// State
let state = use_state(|| initial);
let reducer = use_reducer(Reducer::default);

// Effect
use_effect(|| { /* cleanup */ || () });
use_effect_with(deps, |deps| { /* cleanup */ || () });

// Context
use_context::<T>();

// Callback
Callback::from(|e| { });
```

### Dioxus Cheatsheet

```rust
// Signal
let mut sig = use_signal(|| initial);
sig += 1;  // Direct mutation

// Memo
let memo = use_memo(move || /* computation */);

// Effect
use_effect(move || { /* effect */ });

// Resource
let data = use_resource(move || async { });
```

---

## Sources

- [Leptos Book](https://book.leptos.dev/)
- [Leptos Docs](https://docs.rs/leptos/latest/leptos/)
- [Yew Documentation](https://yew.rs/)
- [Yew Hooks](https://yew.rs/docs/concepts/function-components/pre-defined-hooks)
- [Dioxus Documentation](https://dioxuslabs.com/learn/0.7/)
- [Sycamore Documentation](https://sycamore.dev/)
- [Leptos Hydration Bugs](https://book.leptos.dev/ssr/24_hydration_bugs.html)
- [Framework Comparison](https://github.com/flosse/rust-web-framework-comparison)

---

*Document completed: Step 2 of Rust WebAssembly Skill Research*
