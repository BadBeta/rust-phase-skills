# Styling Subskill

> Quick reference for CSS and Tailwind with Rust WASM.

## When to Activate

Activate when user asks about:
- Tailwind CSS with Rust WASM frameworks
- CSS-in-Rust solutions (stylers, stylist)
- Styling Leptos/Yew/Dioxus components
- Dark mode implementation
- Responsive design in WASM apps
- Animations and transitions
- Coordinating styles with LiveView

## Full Reference

See `rust_wasm_styling.md` for complete documentation.

## Tailwind Setup

```javascript
// tailwind.config.js
module.exports = {
    content: [
        "./src/**/*.rs",  // Include Rust files!
        "./index.html",
    ],
    darkMode: 'class',
    theme: { extend: {} },
    plugins: [],
}
```

```css
/* input.css */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

## Tailwind in Leptos

```rust
#[component]
fn Button(
    #[prop(default = "primary")] variant: &'static str,
    children: Children,
) -> impl IntoView {
    let classes = match variant {
        "primary" => "bg-blue-600 text-white hover:bg-blue-700",
        "secondary" => "bg-gray-200 text-gray-900 hover:bg-gray-300",
        _ => "",
    };

    view! {
        <button class=format!(
            "px-4 py-2 rounded-md font-medium transition-colors {}",
            classes
        )>
            {children()}
        </button>
    }
}
```

## CSS-in-Rust (stylers)

```rust
use stylers::style;

#[component]
fn StyledButton() -> impl IntoView {
    let class = style! {
        button {
            background: linear-gradient(135deg, #667eea, #764ba2);
            color: white;
            padding: 0.75rem 1.5rem;
            border-radius: 0.5rem;
        }
        button:hover {
            transform: translateY(-2px);
        }
    };

    view! { <button class=class>"Click"</button> }
}
```

## Dark Mode

```rust
// Toggle dark class on document
create_effect(move |_| {
    let is_dark = dark_mode.get();
    if let Some(root) = document().document_element() {
        let class_list = root.class_list();
        if is_dark {
            let _ = class_list.add_1("dark");
        } else {
            let _ = class_list.remove_1("dark");
        }
    }
});

// Use in components
view! {
    <div class="bg-white dark:bg-gray-800 text-gray-900 dark:text-white">
        "Auto dark mode"
    </div>
}
```

## Class Composition Helper

```rust
pub fn cx(classes: &[(&str, bool)]) -> String {
    classes.iter()
        .filter_map(|(c, cond)| if *cond { Some(*c) } else { None })
        .collect::<Vec<_>>()
        .join(" ")
}

// Usage
let classes = cx(&[
    ("base-class", true),
    ("active", is_active),
    ("disabled", is_disabled),
]);
```

## Key Patterns

1. **Add `.rs` to Tailwind content** - Otherwise classes are purged
2. **Use class composition** - Avoid string concatenation everywhere
3. **Dark mode with `class` strategy** - More control than `media`
4. **Extract common styles** - Create component style constants
5. **Consistent approach** - Don't mix inline, Tailwind, and CSS-in-Rust
