# Rust WASM Frameworks Subskill

> Quick reference for Leptos, Yew, Dioxus, and Sycamore.

## When to Activate

Activate when user asks about:
- Choosing a Rust WASM framework
- Leptos signals, components, routing
- Yew hooks, agents, components
- Dioxus RSX, state management
- Sycamore reactive primitives
- SSR/hydration with Rust frameworks

## Full Reference

See `rust_wasm_frameworks.md` for complete documentation.

## Framework Comparison

| Framework | Reactivity | SSR | Ecosystem | Learning Curve |
|-----------|------------|-----|-----------|----------------|
| Leptos | Fine-grained signals | Yes | Growing | Medium |
| Yew | Virtual DOM | Yes | Mature | Medium |
| Dioxus | Fine-grained | Yes | Growing | Low (React-like) |
| Sycamore | Fine-grained | Yes | Smaller | Medium |

## Recommended Choice

**Leptos** for new projects in 2025:
- Fine-grained reactivity (best performance)
- Strong SSR support
- Active development
- Growing ecosystem

## Leptos Quick Example

```rust
use leptos::*;

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <button on:click=move |_| set_count.update(|c| *c += 1)>
            "Count: " {count}
        </button>
    }
}
```

## Yew Quick Example

```rust
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let count = use_state(|| 0);
    let onclick = {
        let count = count.clone();
        Callback::from(move |_| count.set(*count + 1))
    };

    html! {
        <button {onclick}>{ format!("Count: {}", *count) }</button>
    }
}
```

## Key Patterns

1. **Use signals/memos** for derived state
2. **Keyed iteration** with `<For>` for lists
3. **Component composition** over prop drilling
4. **Async resources** for data fetching
