# Rust WebAssembly Styling: CSS & Tailwind Integration

> Comprehensive guide to styling Rust WebAssembly applications with CSS and Tailwind CSS.

## Table of Contents

1. [Styling Strategies Overview](#styling-strategies-overview)
2. [CSS Modules with Rust Frameworks](#css-modules-with-rust-frameworks)
3. [Tailwind CSS Integration](#tailwind-css-integration)
4. [Scoped Styles](#scoped-styles)
5. [Dynamic Styling](#dynamic-styling)
6. [CSS-in-Rust Approaches](#css-in-rust-approaches)
7. [Theming & Dark Mode](#theming--dark-mode)
8. [Animations & Transitions](#animations--transitions)
9. [Responsive Design](#responsive-design)
10. [LiveView & WASM Styling](#liveview--wasm-styling)
11. [Patterns & Anti-Patterns](#patterns--anti-patterns)
12. [Common Failures](#common-failures)
13. [Quick Reference](#quick-reference)

---

## Styling Strategies Overview

### Styling Options for Rust WASM

```
┌─────────────────────────────────────────────────────────────┐
│                    Styling Approaches                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  External CSS   │  │   Tailwind CSS  │  │  CSS-in-Rust │ │
│  │  (Traditional)  │  │   (Utility)     │  │  (Inline)    │ │
│  └────────┬────────┘  └────────┬────────┘  └──────┬───────┘ │
│           │                    │                   │         │
│  ┌────────▼────────┐  ┌────────▼────────┐  ┌──────▼───────┐ │
│  │ CSS Modules     │  │ @apply in Rust  │  │ style! macro │ │
│  │ BEM naming      │  │ Class strings   │  │ Typed styles │ │
│  │ Preprocessors   │  │ JIT compilation │  │ Runtime CSS  │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│                                                              │
│  Compile-time ◄────────────────────────────► Runtime        │
│  Separation   ◄────────────────────────────► Co-location    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Framework Support Matrix

| Framework | CSS Modules | Tailwind | Inline Styles | CSS-in-Rust |
|-----------|-------------|----------|---------------|-------------|
| Leptos | ✓ (via Trunk) | ✓ | ✓ | stylers |
| Yew | ✓ (via Trunk) | ✓ | ✓ | stylist |
| Dioxus | ✓ | ✓ | ✓ | manganis |
| Sycamore | ✓ (via Trunk) | ✓ | ✓ | - |

---

## CSS Modules with Rust Frameworks

### Trunk CSS Setup

```toml
# Trunk.toml
[[hooks]]
stage = "build"
command = "sh"
command_arguments = ["-c", "npx tailwindcss -i ./input.css -o ./dist/tailwind.css --minify"]
```

```html
<!-- index.html -->
<!DOCTYPE html>
<html>
<head>
    <link data-trunk rel="css" href="styles/main.css" />
    <link data-trunk rel="css" href="dist/tailwind.css" />
</head>
<body></body>
</html>
```

### CSS Modules in Leptos

```css
/* styles/button.module.css */
.button {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 600;
    transition: all 0.2s ease;
}

.button--primary {
    background-color: #3b82f6;
    color: white;
}

.button--primary:hover {
    background-color: #2563eb;
}

.button--secondary {
    background-color: #e5e7eb;
    color: #374151;
}
```

```rust
use leptos::*;

// Using CSS module classes
#[component]
pub fn Button(
    #[prop(default = "primary")] variant: &'static str,
    children: Children,
) -> impl IntoView {
    let class = match variant {
        "primary" => "button button--primary",
        "secondary" => "button button--secondary",
        _ => "button",
    };

    view! {
        <button class=class>
            {children()}
        </button>
    }
}
```

### CSS Modules in Yew

```rust
use yew::prelude::*;

// Load CSS module at compile time
const STYLES: &str = include_str!("../styles/card.module.css");

#[function_component]
pub fn Card(props: &CardProps) -> Html {
    // Inject styles once
    use_effect_with((), |_| {
        if web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("card-styles"))
            .is_none()
        {
            inject_styles("card-styles", STYLES);
        }
        || ()
    });

    html! {
        <div class="card">
            <div class="card__header">{ &props.title }</div>
            <div class="card__body">{ &props.children }</div>
        </div>
    }
}

fn inject_styles(id: &str, css: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style").unwrap();
    style.set_id(id);
    style.set_inner_html(css);
    document.head().unwrap().append_child(&style).unwrap();
}
```

---

## Tailwind CSS Integration

### Project Setup

```bash
# Initialize Tailwind
npm init -y
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init
```

```javascript
// tailwind.config.js
/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
    ],
    theme: {
        extend: {},
    },
    plugins: [],
}
```

```css
/* input.css */
@tailwind base;
@tailwind components;
@tailwind utilities;

/* Custom utilities for WASM components */
@layer components {
    .wasm-loading {
        @apply animate-pulse bg-gray-200 rounded;
    }

    .wasm-container {
        @apply relative min-h-[200px];
    }
}
```

### Trunk Integration

```toml
# Trunk.toml
[build]
target = "index.html"
dist = "dist"

[[hooks]]
stage = "pre_build"
command = "sh"
command_arguments = ["-c", "npx tailwindcss -i ./input.css -o ./output.css"]
```

```html
<!-- index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link data-trunk rel="rust" data-wasm-opt="z" />
    <link data-trunk rel="css" href="output.css" />
</head>
<body class="bg-gray-100 min-h-screen">
    <div id="app"></div>
</body>
</html>
```

### Tailwind in Leptos

```rust
use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="min-h-screen bg-gray-50">
            <nav class="bg-white shadow-sm border-b">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div class="flex justify-between h-16">
                        <div class="flex items-center">
                            <span class="text-xl font-bold text-gray-900">
                                "WASM Dashboard"
                            </span>
                        </div>
                    </div>
                </div>
            </nav>

            <main class="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    <StatCard title="Users" value="1,234" trend="+12%" />
                    <StatCard title="Revenue" value="$45,678" trend="+8%" />
                    <StatCard title="Orders" value="567" trend="+23%" />
                </div>
            </main>
        </div>
    }
}

#[component]
fn StatCard(
    title: &'static str,
    value: &'static str,
    trend: &'static str,
) -> impl IntoView {
    let trend_color = if trend.starts_with('+') {
        "text-green-600"
    } else {
        "text-red-600"
    };

    view! {
        <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
                <dt class="text-sm font-medium text-gray-500 truncate">
                    {title}
                </dt>
                <dd class="mt-1 text-3xl font-semibold text-gray-900">
                    {value}
                </dd>
                <dd class=format!("mt-2 text-sm {}", trend_color)>
                    {trend}
                </dd>
            </div>
        </div>
    }
}
```

### Tailwind in Yew

```rust
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or_default]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub size: ButtonSize,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    pub children: Children,
}

#[derive(Default, PartialEq, Clone)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Ghost,
}

#[derive(Default, PartialEq, Clone)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[function_component]
pub fn Button(props: &ButtonProps) -> Html {
    let base_classes = "inline-flex items-center justify-center font-medium \
                        rounded-md focus:outline-none focus:ring-2 \
                        focus:ring-offset-2 transition-colors duration-200";

    let variant_classes = match props.variant {
        ButtonVariant::Primary => {
            "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500"
        }
        ButtonVariant::Secondary => {
            "bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-500"
        }
        ButtonVariant::Danger => {
            "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500"
        }
        ButtonVariant::Ghost => {
            "bg-transparent text-gray-700 hover:bg-gray-100 focus:ring-gray-500"
        }
    };

    let size_classes = match props.size {
        ButtonSize::Small => "px-2.5 py-1.5 text-xs",
        ButtonSize::Medium => "px-4 py-2 text-sm",
        ButtonSize::Large => "px-6 py-3 text-base",
    };

    let disabled_classes = if props.disabled {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };

    let classes = format!(
        "{} {} {} {}",
        base_classes, variant_classes, size_classes, disabled_classes
    );

    html! {
        <button
            class={classes}
            disabled={props.disabled}
            onclick={props.onclick.clone()}
        >
            { for props.children.iter() }
        </button>
    }
}
```

### Dynamic Class Composition

```rust
use leptos::*;

/// Utility for composing Tailwind classes
pub fn classes(classes: &[&str]) -> String {
    classes.iter()
        .filter(|c| !c.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Conditional class helper
pub fn class_if(condition: bool, class: &str) -> &str {
    if condition { class } else { "" }
}

#[component]
pub fn Alert(
    #[prop(default = "info")] variant: &'static str,
    #[prop(default = false)] dismissible: bool,
    children: Children,
) -> impl IntoView {
    let (visible, set_visible) = create_signal(true);

    let variant_classes = match variant {
        "success" => "bg-green-50 text-green-800 border-green-200",
        "warning" => "bg-yellow-50 text-yellow-800 border-yellow-200",
        "error" => "bg-red-50 text-red-800 border-red-200",
        _ => "bg-blue-50 text-blue-800 border-blue-200",
    };

    let icon_classes = match variant {
        "success" => "text-green-400",
        "warning" => "text-yellow-400",
        "error" => "text-red-400",
        _ => "text-blue-400",
    };

    view! {
        <Show when=move || visible.get()>
            <div class=format!(
                "rounded-md border p-4 {} {}",
                variant_classes,
                class_if(!visible.get(), "hidden")
            )>
                <div class="flex">
                    <div class=format!("flex-shrink-0 {}", icon_classes)>
                        // Icon here
                    </div>
                    <div class="ml-3 flex-1">
                        {children()}
                    </div>
                    <Show when=move || dismissible>
                        <button
                            class="ml-auto -mx-1.5 -my-1.5 rounded-lg p-1.5 \
                                   hover:bg-black/5 focus:ring-2"
                            on:click=move |_| set_visible.set(false)
                        >
                            <span class="sr-only">"Dismiss"</span>
                            "×"
                        </button>
                    </Show>
                </div>
            </div>
        </Show>
    }
}
```

---

## Scoped Styles

### Using stylers (Leptos)

```toml
[dependencies]
stylers = "1.0"
```

```rust
use leptos::*;
use stylers::style;

#[component]
pub fn ScopedButton() -> impl IntoView {
    let class_name = style! {
        button {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 0.75rem 1.5rem;
            border: none;
            border-radius: 0.5rem;
            font-weight: 600;
            cursor: pointer;
            transition: transform 0.2s, box-shadow 0.2s;
        }

        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
        }

        button:active {
            transform: translateY(0);
        }
    };

    view! {
        <button class=class_name>
            "Styled Button"
        </button>
    }
}
```

### Using stylist (Yew)

```toml
[dependencies]
stylist = { version = "0.13", features = ["yew_integration"] }
```

```rust
use stylist::yew::styled_component;
use stylist::css;
use yew::prelude::*;

#[styled_component]
pub fn StyledCard() -> Html {
    let stylesheet = css!(
        r#"
        :host {
            display: block;
            background: white;
            border-radius: 0.5rem;
            box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
            overflow: hidden;
        }

        .header {
            padding: 1rem;
            background: linear-gradient(to right, #f8fafc, #f1f5f9);
            border-bottom: 1px solid #e2e8f0;
        }

        .body {
            padding: 1rem;
        }

        .footer {
            padding: 1rem;
            background: #f8fafc;
            border-top: 1px solid #e2e8f0;
        }
        "#
    );

    html! {
        <div class={stylesheet}>
            <div class="header">
                <h3>{"Card Title"}</h3>
            </div>
            <div class="body">
                <p>{"Card content goes here..."}</p>
            </div>
            <div class="footer">
                <button>{"Action"}</button>
            </div>
        </div>
    }
}
```

### Dynamic Scoped Styles

```rust
use stylist::{css, Style};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    pub value: f32,  // 0.0 to 1.0
    #[prop_or("blue")]
    pub color: &'static str,
}

#[function_component]
pub fn Progress(props: &ProgressProps) -> Html {
    let color = match props.color {
        "green" => "#22c55e",
        "red" => "#ef4444",
        "yellow" => "#eab308",
        _ => "#3b82f6",
    };

    let width_percent = (props.value * 100.0).min(100.0).max(0.0);

    let style = css!(
        r#"
        .track {
            height: 0.5rem;
            background: #e5e7eb;
            border-radius: 9999px;
            overflow: hidden;
        }

        .bar {
            height: 100%;
            background: ${color};
            width: ${width}%;
            transition: width 0.3s ease;
        }
        "#,
        color = color,
        width = width_percent
    );

    html! {
        <div class={style}>
            <div class="track">
                <div class="bar" />
            </div>
        </div>
    }
}
```

---

## Dynamic Styling

### Reactive Style Updates in Leptos

```rust
use leptos::*;

#[component]
pub fn ColorPicker() -> impl IntoView {
    let (hue, set_hue) = create_signal(200);
    let (saturation, set_saturation) = create_signal(70);
    let (lightness, set_lightness) = create_signal(50);

    // Derived style string
    let background_style = move || {
        format!(
            "background-color: hsl({}, {}%, {}%)",
            hue.get(),
            saturation.get(),
            lightness.get()
        )
    };

    view! {
        <div class="p-6 space-y-4">
            <div
                class="w-full h-32 rounded-lg shadow-inner"
                style=background_style
            />

            <div class="space-y-2">
                <label class="block">
                    <span class="text-sm text-gray-600">"Hue: " {hue}</span>
                    <input
                        type="range"
                        min="0"
                        max="360"
                        class="w-full"
                        prop:value=hue
                        on:input=move |ev| {
                            set_hue.set(event_target_value(&ev).parse().unwrap_or(0))
                        }
                    />
                </label>

                <label class="block">
                    <span class="text-sm text-gray-600">
                        "Saturation: " {saturation} "%"
                    </span>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        class="w-full"
                        prop:value=saturation
                        on:input=move |ev| {
                            set_saturation.set(event_target_value(&ev).parse().unwrap_or(0))
                        }
                    />
                </label>

                <label class="block">
                    <span class="text-sm text-gray-600">
                        "Lightness: " {lightness} "%"
                    </span>
                    <input
                        type="range"
                        min="0"
                        max="100"
                        class="w-full"
                        prop:value=lightness
                        on:input=move |ev| {
                            set_lightness.set(event_target_value(&ev).parse().unwrap_or(0))
                        }
                    />
                </label>
            </div>
        </div>
    }
}
```

### CSS Custom Properties

```rust
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    let (theme, set_theme) = create_signal(Theme::Light);

    // Update CSS custom properties when theme changes
    create_effect(move |_| {
        let theme = theme.get();
        if let Some(document) = web_sys::window()
            .and_then(|w| w.document())
        {
            if let Some(root) = document.document_element() {
                let style = root.unchecked_ref::<web_sys::HtmlElement>().style();

                match theme {
                    Theme::Light => {
                        let _ = style.set_property("--color-bg", "#ffffff");
                        let _ = style.set_property("--color-text", "#1f2937");
                        let _ = style.set_property("--color-primary", "#3b82f6");
                        let _ = style.set_property("--color-surface", "#f3f4f6");
                    }
                    Theme::Dark => {
                        let _ = style.set_property("--color-bg", "#111827");
                        let _ = style.set_property("--color-text", "#f9fafb");
                        let _ = style.set_property("--color-primary", "#60a5fa");
                        let _ = style.set_property("--color-surface", "#1f2937");
                    }
                }
            }
        }
    });

    view! {
        <div
            class="min-h-screen transition-colors duration-300"
            style="background: var(--color-bg); color: var(--color-text);"
        >
            <button
                class="fixed top-4 right-4 p-2 rounded-lg"
                style="background: var(--color-surface);"
                on:click=move |_| {
                    set_theme.update(|t| *t = match t {
                        Theme::Light => Theme::Dark,
                        Theme::Dark => Theme::Light,
                    })
                }
            >
                {move || match theme.get() {
                    Theme::Light => "🌙",
                    Theme::Dark => "☀️",
                }}
            </button>
            {children()}
        </div>
    }
}

#[derive(Clone, Copy)]
enum Theme {
    Light,
    Dark,
}
```

### Inline Styles with Validation

```rust
use std::collections::HashMap;

/// Type-safe style builder
pub struct StyleBuilder {
    properties: HashMap<&'static str, String>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn width(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("width", value.into());
        self
    }

    pub fn height(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("height", value.into());
        self
    }

    pub fn background(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("background", value.into());
        self
    }

    pub fn color(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("color", value.into());
        self
    }

    pub fn transform(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("transform", value.into());
        self
    }

    pub fn transition(mut self, value: impl Into<String>) -> Self {
        self.properties.insert("transition", value.into());
        self
    }

    pub fn custom(mut self, property: &'static str, value: impl Into<String>) -> Self {
        self.properties.insert(property, value.into());
        self
    }

    pub fn build(self) -> String {
        self.properties
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

// Usage
#[component]
pub fn AnimatedBox(progress: f32) -> impl IntoView {
    let style = StyleBuilder::new()
        .width("100px")
        .height("100px")
        .background("#3b82f6")
        .transform(format!("translateX({}px)", progress * 200.0))
        .transition("transform 0.3s ease")
        .build();

    view! {
        <div style=style />
    }
}
```

---

## CSS-in-Rust Approaches

### Compile-Time CSS Generation

```rust
// Build script for CSS extraction
// build.rs
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/");

    // Extract CSS from Rust source files
    let css = extract_styles("src/");
    fs::write("generated.css", css).expect("Failed to write CSS");
}

fn extract_styles(dir: &str) -> String {
    // Parse Rust files for style! macros
    // This is simplified - real implementation would use syn
    let mut css = String::new();

    for entry in walkdir::WalkDir::new(dir) {
        if let Ok(entry) = entry {
            if entry.path().extension().map_or(false, |e| e == "rs") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    // Extract style blocks (simplified)
                    css.push_str(&extract_style_blocks(&content));
                }
            }
        }
    }

    css
}

fn extract_style_blocks(content: &str) -> String {
    // Regex-based extraction (simplified)
    String::new()
}
```

### Runtime Style Injection

```rust
use wasm_bindgen::prelude::*;
use web_sys::{Document, HtmlStyleElement};
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    static INJECTED_STYLES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

pub fn inject_css(id: &str, css: &str) {
    INJECTED_STYLES.with(|styles| {
        let mut styles = styles.borrow_mut();

        if styles.contains(id) {
            return;
        }

        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            let style: HtmlStyleElement = document
                .create_element("style")
                .expect("Failed to create style element")
                .dyn_into()
                .expect("Not a style element");

            style.set_id(id);
            style.set_inner_html(css);

            if let Some(head) = document.head() {
                head.append_child(&style).expect("Failed to append style");
            }

            styles.insert(id.to_string());
        }
    });
}

// Component using runtime injection
#[component]
pub fn Widget() -> impl IntoView {
    const WIDGET_CSS: &str = r#"
        .widget {
            border: 1px solid #e5e7eb;
            border-radius: 0.5rem;
            padding: 1rem;
        }
        .widget__title {
            font-weight: 600;
            margin-bottom: 0.5rem;
        }
    "#;

    inject_css("widget-styles", WIDGET_CSS);

    view! {
        <div class="widget">
            <div class="widget__title">"Widget Title"</div>
            <div class="widget__content">"Content here"</div>
        </div>
    }
}
```

---

## Theming & Dark Mode

### System Preference Detection

```rust
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Copy, PartialEq)]
pub enum ColorScheme {
    Light,
    Dark,
    System,
}

#[component]
pub fn ThemeContext(children: Children) -> impl IntoView {
    let (scheme, set_scheme) = create_signal(ColorScheme::System);

    // Detect system preference
    let system_prefers_dark = create_memo(move |_| {
        web_sys::window()
            .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok())
            .flatten()
            .map(|mq| mq.matches())
            .unwrap_or(false)
    });

    // Resolved theme
    let is_dark = create_memo(move |_| {
        match scheme.get() {
            ColorScheme::Light => false,
            ColorScheme::Dark => true,
            ColorScheme::System => system_prefers_dark.get(),
        }
    });

    // Apply theme class to document
    create_effect(move |_| {
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(root) = document.document_element() {
                let class_list = root.class_list();
                if is_dark.get() {
                    let _ = class_list.add_1("dark");
                } else {
                    let _ = class_list.remove_1("dark");
                }
            }
        }
    });

    // Listen for system preference changes
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(Some(mq)) = window.match_media("(prefers-color-scheme: dark)") {
                let callback = Closure::wrap(Box::new(move |_: web_sys::Event| {
                    // Force re-evaluation
                    set_scheme.update(|s| *s = *s);
                }) as Box<dyn Fn(_)>);

                let _ = mq.add_event_listener_with_callback(
                    "change",
                    callback.as_ref().unchecked_ref()
                );

                callback.forget();  // Keep alive
            }
        }
    });

    provide_context(scheme);
    provide_context(set_scheme);

    children()
}
```

### Tailwind Dark Mode

```javascript
// tailwind.config.js
module.exports = {
    darkMode: 'class',  // or 'media' for system preference
    content: ["./src/**/*.rs"],
    theme: {
        extend: {
            colors: {
                surface: {
                    light: '#ffffff',
                    dark: '#1f2937',
                },
            },
        },
    },
}
```

```rust
#[component]
pub fn DarkModeCard() -> impl IntoView {
    view! {
        <div class="
            bg-white dark:bg-gray-800
            text-gray-900 dark:text-gray-100
            border border-gray-200 dark:border-gray-700
            rounded-lg shadow-sm
            p-6
            transition-colors duration-200
        ">
            <h3 class="text-lg font-semibold mb-2">
                "Dark Mode Support"
            </h3>
            <p class="text-gray-600 dark:text-gray-400">
                "This card automatically adapts to dark mode."
            </p>
        </div>
    }
}
```

---

## Animations & Transitions

### CSS Keyframe Animations

```rust
use leptos::*;

const ANIMATIONS_CSS: &str = r#"
@keyframes fade-in {
    from {
        opacity: 0;
        transform: translateY(-10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

.animate-fade-in {
    animation: fade-in 0.3s ease-out;
}

.animate-spin-slow {
    animation: spin 3s linear infinite;
}

.animate-pulse-custom {
    animation: pulse 2s ease-in-out infinite;
}
"#;

#[component]
pub fn AnimatedList(items: Vec<String>) -> impl IntoView {
    view! {
        <ul class="space-y-2">
            <For
                each=move || items.clone().into_iter().enumerate()
                key=|(i, _)| *i
                children=|(i, item)| {
                    let delay = format!("animation-delay: {}ms", i * 100);
                    view! {
                        <li
                            class="animate-fade-in p-4 bg-white rounded shadow"
                            style=delay
                        >
                            {item}
                        </li>
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center">
            <div class="
                w-8 h-8 border-4 border-blue-200
                border-t-blue-600 rounded-full
                animate-spin
            " />
        </div>
    }
}
```

### Transition Groups

```rust
use leptos::*;

#[component]
pub fn Modal(
    show: ReadSignal<bool>,
    on_close: impl Fn() + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            // Backdrop
            <div
                class="
                    fixed inset-0 bg-black/50
                    transition-opacity duration-300
                "
                class:opacity-0=move || !show.get()
                on:click=move |_| on_close()
            />

            // Modal panel
            <div
                class="
                    fixed inset-0 flex items-center justify-center p-4
                    transition-all duration-300
                "
                class:opacity-0=move || !show.get()
                class:scale-95=move || !show.get()
            >
                <div
                    class="
                        bg-white rounded-lg shadow-xl
                        max-w-md w-full p-6
                        transform transition-all
                    "
                    on:click=|e| e.stop_propagation()
                >
                    {children()}
                </div>
            </div>
        </Show>
    }
}
```

### Web Animations API

```rust
use wasm_bindgen::prelude::*;
use web_sys::{Element, KeyframeEffect, Animation};
use js_sys::{Object, Array, Reflect};

pub fn animate_element(
    element: &Element,
    keyframes: &[(&str, &str, &str)],  // (property, from, to)
    duration_ms: f64,
) -> Result<Animation, JsValue> {
    let keyframes_array = Array::new();

    // From keyframe
    let from = Object::new();
    for (prop, from_val, _) in keyframes {
        Reflect::set(&from, &(*prop).into(), &(*from_val).into())?;
    }
    keyframes_array.push(&from);

    // To keyframe
    let to = Object::new();
    for (prop, _, to_val) in keyframes {
        Reflect::set(&to, &(*prop).into(), &(*to_val).into())?;
    }
    keyframes_array.push(&to);

    let options = Object::new();
    Reflect::set(&options, &"duration".into(), &duration_ms.into())?;
    Reflect::set(&options, &"easing".into(), &"ease-out".into())?;
    Reflect::set(&options, &"fill".into(), &"forwards".into())?;

    element.animate_with_keyframe_animation_options(
        Some(&keyframes_array),
        &options.into()
    )
}

// Usage
#[component]
pub fn AnimatedButton() -> impl IntoView {
    let button_ref = create_node_ref::<html::Button>();

    let on_click = move |_| {
        if let Some(el) = button_ref.get() {
            let _ = animate_element(
                &el,
                &[
                    ("transform", "scale(1)", "scale(0.95)"),
                    ("opacity", "1", "0.8"),
                ],
                100.0,
            );
        }
    };

    view! {
        <button
            node_ref=button_ref
            class="px-4 py-2 bg-blue-600 text-white rounded"
            on:click=on_click
        >
            "Click me"
        </button>
    }
}
```

---

## Responsive Design

### Responsive Utilities

```rust
use leptos::*;

/// Breakpoint detection hook
pub fn use_breakpoint() -> Memo<Breakpoint> {
    let (width, set_width) = create_signal(get_window_width());

    // Listen for resize
    create_effect(move |_| {
        if let Some(window) = web_sys::window() {
            let callback = Closure::wrap(Box::new(move || {
                set_width.set(get_window_width());
            }) as Box<dyn Fn()>);

            let _ = window.add_event_listener_with_callback(
                "resize",
                callback.as_ref().unchecked_ref()
            );

            callback.forget();
        }
    });

    create_memo(move |_| {
        let w = width.get();
        if w < 640 { Breakpoint::Xs }
        else if w < 768 { Breakpoint::Sm }
        else if w < 1024 { Breakpoint::Md }
        else if w < 1280 { Breakpoint::Lg }
        else if w < 1536 { Breakpoint::Xl }
        else { Breakpoint::Xxl }
    })
}

fn get_window_width() -> u32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|w| w.as_f64())
        .map(|w| w as u32)
        .unwrap_or(1024)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    Xs,   // < 640px
    Sm,   // >= 640px
    Md,   // >= 768px
    Lg,   // >= 1024px
    Xl,   // >= 1280px
    Xxl,  // >= 1536px
}

// Usage
#[component]
pub fn ResponsiveLayout() -> impl IntoView {
    let breakpoint = use_breakpoint();

    view! {
        <div class="container mx-auto px-4">
            {move || match breakpoint.get() {
                Breakpoint::Xs | Breakpoint::Sm => view! {
                    <MobileLayout />
                }.into_view(),
                Breakpoint::Md => view! {
                    <TabletLayout />
                }.into_view(),
                _ => view! {
                    <DesktopLayout />
                }.into_view(),
            }}
        </div>
    }
}
```

### Container Queries (Modern CSS)

```css
/* Container queries in CSS */
.card-container {
    container-type: inline-size;
    container-name: card;
}

@container card (min-width: 400px) {
    .card {
        flex-direction: row;
    }
}

@container card (max-width: 399px) {
    .card {
        flex-direction: column;
    }
}
```

```rust
#[component]
pub fn ResponsiveCard() -> impl IntoView {
    view! {
        <div class="card-container">
            <div class="card flex gap-4 p-4 bg-white rounded-lg shadow">
                <img
                    src="/image.jpg"
                    class="w-24 h-24 object-cover rounded"
                />
                <div class="flex-1">
                    <h3 class="font-bold">"Card Title"</h3>
                    <p class="text-gray-600">"Description text..."</p>
                </div>
            </div>
        </div>
    }
}
```

---

## LiveView & WASM Styling

### Coordinating LiveView and WASM Styles

```css
/* Shared design tokens */
:root {
    --wasm-primary: #3b82f6;
    --wasm-secondary: #6b7280;
    --wasm-success: #22c55e;
    --wasm-error: #ef4444;
    --wasm-transition: 200ms ease;
}

/* LiveView styles that WASM components inherit */
.lv-wasm-container {
    position: relative;
    min-height: 100px;
}

.lv-wasm-container[phx-update="ignore"] {
    /* Ensure WASM-controlled DOM isn't disrupted */
    contain: layout style;
}

/* Loading state while WASM initializes */
.lv-wasm-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    background: #f3f4f6;
}

.lv-wasm-loading::after {
    content: '';
    width: 2rem;
    height: 2rem;
    border: 3px solid #e5e7eb;
    border-top-color: var(--wasm-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}
```

```elixir
# LiveView component with WASM styling coordination
defmodule MyAppWeb.WasmEditorLive do
  use MyAppWeb, :live_view

  def render(assigns) do
    ~H"""
    <div class="wasm-editor-wrapper" phx-hook="WasmEditor" id="editor">
      <div
        class="lv-wasm-container lv-wasm-loading"
        phx-update="ignore"
        id="wasm-mount"
        data-theme={@theme}
        data-accent-color={@accent_color}
      >
        <%!-- WASM renders here --%>
      </div>

      <div class="editor-controls flex gap-2 mt-4">
        <button
          phx-click="toggle_theme"
          class="px-4 py-2 rounded bg-gray-200 hover:bg-gray-300"
        >
          Toggle Theme
        </button>
      </div>
    </div>
    """
  end
end
```

```javascript
// Hook that bridges LiveView and WASM styling
const WasmEditor = {
    mounted() {
        this.container = this.el.querySelector('#wasm-mount');
        this.theme = this.container.dataset.theme;
        this.accentColor = this.container.dataset.accentColor;

        // Initialize WASM with current theme
        import('./pkg/editor.js').then(async (wasm) => {
            await wasm.default();

            this.editor = wasm.Editor.new(this.container, {
                theme: this.theme,
                accentColor: this.accentColor,
            });

            // Remove loading state
            this.container.classList.remove('lv-wasm-loading');
        });

        // Listen for LiveView theme changes
        this.handleEvent('theme_changed', ({ theme }) => {
            this.editor?.setTheme(theme);
        });
    },

    updated() {
        // Handle data attribute changes
        const newTheme = this.container.dataset.theme;
        if (newTheme !== this.theme) {
            this.theme = newTheme;
            this.editor?.setTheme(newTheme);
        }
    },

    destroyed() {
        this.editor?.destroy();
    }
};
```

### WASM Component Styling for LiveView

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Editor {
    container: web_sys::Element,
    theme: String,
    style_element: web_sys::HtmlStyleElement,
}

#[wasm_bindgen]
impl Editor {
    #[wasm_bindgen(constructor)]
    pub fn new(container: web_sys::Element, options: JsValue) -> Result<Editor, JsValue> {
        let options: EditorOptions = serde_wasm_bindgen::from_value(options)?;

        // Create scoped styles
        let style_element = Self::create_styles(&options.theme)?;

        // Build initial UI
        container.set_inner_html(&Self::render_html(&options));

        Ok(Editor {
            container,
            theme: options.theme,
            style_element,
        })
    }

    fn create_styles(theme: &str) -> Result<web_sys::HtmlStyleElement, JsValue> {
        let document = web_sys::window().unwrap().document().unwrap();
        let style = document.create_element("style")?
            .dyn_into::<web_sys::HtmlStyleElement>()?;

        let css = Self::generate_theme_css(theme);
        style.set_inner_html(&css);
        document.head().unwrap().append_child(&style)?;

        Ok(style)
    }

    fn generate_theme_css(theme: &str) -> String {
        let (bg, text, border) = match theme {
            "dark" => ("#1f2937", "#f9fafb", "#374151"),
            _ => ("#ffffff", "#1f2937", "#e5e7eb"),
        };

        format!(r#"
            .wasm-editor {{
                background: {bg};
                color: {text};
                border: 1px solid {border};
                border-radius: 0.5rem;
                padding: 1rem;
                font-family: ui-monospace, monospace;
            }}

            .wasm-editor-line {{
                padding: 0.125rem 0;
            }}

            .wasm-editor-line:hover {{
                background: rgba(128, 128, 128, 0.1);
            }}
        "#, bg = bg, text = text, border = border)
    }

    #[wasm_bindgen(js_name = setTheme)]
    pub fn set_theme(&mut self, theme: String) {
        self.theme = theme.clone();
        let css = Self::generate_theme_css(&theme);
        self.style_element.set_inner_html(&css);
    }

    fn render_html(options: &EditorOptions) -> String {
        format!(r#"
            <div class="wasm-editor">
                <div class="wasm-editor-content">
                    <div class="wasm-editor-line">// Ready for editing</div>
                </div>
            </div>
        "#)
    }

    #[wasm_bindgen]
    pub fn destroy(&self) {
        if let Some(parent) = self.style_element.parent_node() {
            let _ = parent.remove_child(&self.style_element);
        }
        self.container.set_inner_html("");
    }
}

#[derive(serde::Deserialize)]
struct EditorOptions {
    theme: String,
    #[serde(rename = "accentColor")]
    accent_color: String,
}
```

---

## Patterns & Anti-Patterns

### Pattern 1: Design Token System

```rust
// Centralized design tokens
pub mod tokens {
    pub mod colors {
        pub const PRIMARY: &str = "#3b82f6";
        pub const PRIMARY_HOVER: &str = "#2563eb";
        pub const SECONDARY: &str = "#6b7280";
        pub const SUCCESS: &str = "#22c55e";
        pub const ERROR: &str = "#ef4444";
        pub const WARNING: &str = "#f59e0b";
    }

    pub mod spacing {
        pub const XS: &str = "0.25rem";
        pub const SM: &str = "0.5rem";
        pub const MD: &str = "1rem";
        pub const LG: &str = "1.5rem";
        pub const XL: &str = "2rem";
    }

    pub mod radius {
        pub const SM: &str = "0.25rem";
        pub const MD: &str = "0.375rem";
        pub const LG: &str = "0.5rem";
        pub const FULL: &str = "9999px";
    }
}

// Usage with Tailwind arbitrary values
view! {
    <button class=format!(
        "bg-[{}] hover:bg-[{}] px-[{}] py-[{}] rounded-[{}]",
        tokens::colors::PRIMARY,
        tokens::colors::PRIMARY_HOVER,
        tokens::spacing::MD,
        tokens::spacing::SM,
        tokens::radius::MD
    )>
        "Styled Button"
    </button>
}
```

### Pattern 2: Component Variants with Tailwind

```rust
/// Reusable button with variants
pub fn button_classes(variant: &str, size: &str, disabled: bool) -> String {
    let base = "inline-flex items-center justify-center font-medium \
                rounded-md transition-colors focus:outline-none focus:ring-2";

    let variant_class = match variant {
        "primary" => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
        "secondary" => "bg-gray-200 text-gray-900 hover:bg-gray-300 focus:ring-gray-500",
        "ghost" => "bg-transparent hover:bg-gray-100 focus:ring-gray-500",
        "danger" => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        _ => "",
    };

    let size_class = match size {
        "sm" => "px-3 py-1.5 text-sm",
        "lg" => "px-6 py-3 text-lg",
        _ => "px-4 py-2 text-base",
    };

    let state_class = if disabled {
        "opacity-50 cursor-not-allowed pointer-events-none"
    } else {
        ""
    };

    format!("{} {} {} {}", base, variant_class, size_class, state_class)
}
```

### Pattern 3: Semantic Class Composition

```rust
/// Group related styles semantically
pub struct Styles;

impl Styles {
    pub fn card() -> &'static str {
        "bg-white dark:bg-gray-800 rounded-lg shadow-sm border \
         border-gray-200 dark:border-gray-700"
    }

    pub fn card_header() -> &'static str {
        "px-4 py-3 border-b border-gray-200 dark:border-gray-700 \
         font-semibold text-gray-900 dark:text-white"
    }

    pub fn card_body() -> &'static str {
        "p-4 text-gray-600 dark:text-gray-300"
    }

    pub fn input() -> &'static str {
        "w-full px-3 py-2 border border-gray-300 dark:border-gray-600 \
         rounded-md shadow-sm focus:ring-2 focus:ring-blue-500 \
         focus:border-blue-500 dark:bg-gray-700 dark:text-white"
    }

    pub fn label() -> &'static str {
        "block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
    }
}

// Usage
view! {
    <div class=Styles::card()>
        <div class=Styles::card_header()>"Form"</div>
        <div class=Styles::card_body()>
            <label class=Styles::label()>"Email"</label>
            <input type="email" class=Styles::input() />
        </div>
    </div>
}
```

### Pattern 4: Conditional Styling

```rust
/// Clean conditional class application
pub fn cx(classes: &[(&str, bool)]) -> String {
    classes
        .iter()
        .filter_map(|(class, condition)| {
            if *condition { Some(*class) } else { None }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Usage
#[component]
fn Tab(active: bool, label: &'static str) -> impl IntoView {
    let classes = cx(&[
        ("px-4 py-2 font-medium border-b-2 transition-colors", true),
        ("border-blue-500 text-blue-600", active),
        ("border-transparent text-gray-500 hover:text-gray-700", !active),
    ]);

    view! {
        <button class=classes>{label}</button>
    }
}
```

### Pattern 5: Style Extraction for SSR

```rust
// Collect styles during SSR for critical CSS
thread_local! {
    static CRITICAL_CSS: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn register_critical_style(css: &str) {
    CRITICAL_CSS.with(|styles| {
        styles.borrow_mut().push(css.to_string());
    });
}

pub fn extract_critical_css() -> String {
    CRITICAL_CSS.with(|styles| {
        styles.borrow().join("\n")
    })
}
```

### Anti-Pattern 1: Inline Style Soup

```rust
// ANTI-PATTERN: Unreadable inline styles
view! {
    <div style="display: flex; align-items: center; justify-content: space-between; padding: 16px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
        // ...
    </div>
}

// CORRECT: Use Tailwind or extract to CSS
view! {
    <div class="flex items-center justify-between p-4 rounded-lg shadow-md bg-gradient-to-br from-indigo-500 to-purple-600">
        // ...
    </div>
}
```

### Anti-Pattern 2: Duplicated Magic Strings

```rust
// ANTI-PATTERN: Same classes repeated everywhere
view! {
    <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
        "Save"
    </button>
    <button class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700">
        "Submit"
    </button>
}

// CORRECT: Extract to component or constant
const BTN_PRIMARY: &str = "px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700";

view! {
    <button class=BTN_PRIMARY>"Save"</button>
    <button class=BTN_PRIMARY>"Submit"</button>
}

// Or use a component
<Button variant="primary">"Save"</Button>
```

### Anti-Pattern 3: Fighting Framework Styles

```css
/* ANTI-PATTERN: Overriding everything with !important */
.my-component .framework-class {
    margin: 0 !important;
    padding: 0 !important;
    border: none !important;
}

/* CORRECT: Work with the framework */
.my-component {
    /* Custom styles that don't conflict */
}
```

### Anti-Pattern 4: Mixing Styling Approaches

```rust
// ANTI-PATTERN: Inconsistent styling in same component
view! {
    <div class="p-4 bg-white">  // Tailwind
        <h1 style="color: red; font-size: 24px;">  // Inline
            "Title"
        </h1>
        <p class={css!("margin-top: 1rem;")}>  // CSS-in-Rust
            "Text"
        </p>
    </div>
}

// CORRECT: Pick one approach and stick with it
view! {
    <div class="p-4 bg-white">
        <h1 class="text-red-600 text-2xl">"Title"</h1>
        <p class="mt-4">"Text"</p>
    </div>
}
```

### Anti-Pattern 5: Styling in Wrong Layer

```rust
// ANTI-PATTERN: Business logic components with hardcoded styles
fn calculate_total(items: &[Item]) -> f64 {
    let total = items.iter().sum();
    // Don't put UI concerns in logic
    console_log!("Total styled in red");  // Wrong!
    total
}

// CORRECT: Keep styling in presentation layer
#[component]
fn TotalDisplay(total: f64) -> impl IntoView {
    let class = if total < 0.0 { "text-red-600" } else { "text-green-600" };
    view! {
        <span class=class>{format!("${:.2}", total)}</span>
    }
}
```

---

## Common Failures

### Failure 1: Tailwind Not Scanning Rust Files

```javascript
// FAILURE: Content paths don't include .rs files
module.exports = {
    content: [
        "./index.html",
        "./src/**/*.js",  // Missing .rs!
    ],
}

// FIX: Add Rust files to content
module.exports = {
    content: [
        "./index.html",
        "./src/**/*.rs",
        "./src/**/*.js",
    ],
}
```

### Failure 2: CSS Not Updating in Dev

```bash
# FAILURE: CSS built once, not watching
npx tailwindcss -i input.css -o output.css

# FIX: Use watch mode
npx tailwindcss -i input.css -o output.css --watch

# Or configure Trunk to rebuild
```

### Failure 3: FOUC (Flash of Unstyled Content)

```html
<!-- FAILURE: CSS loads after HTML renders -->
<body>
    <div id="app"></div>
    <link rel="stylesheet" href="styles.css">
</body>

<!-- FIX: CSS in head, critical CSS inlined -->
<head>
    <style>/* Critical CSS */</style>
    <link rel="stylesheet" href="styles.css">
</head>
```

### Failure 4: Dark Mode Flicker

```rust
// FAILURE: Theme set after render
create_effect(move |_| {
    // DOM already rendered before this runs
    set_dark_mode();
});

// FIX: Set theme before hydration or use SSR
// index.html
<script>
    (function() {
        const theme = localStorage.getItem('theme') ||
            (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
        document.documentElement.classList.toggle('dark', theme === 'dark');
    })();
</script>
```

### Failure 5: Z-Index Wars

```css
/* FAILURE: Arbitrary z-index values */
.modal { z-index: 9999; }
.dropdown { z-index: 10000; }
.tooltip { z-index: 99999; }

/* FIX: Defined z-index scale */
:root {
    --z-dropdown: 100;
    --z-modal: 200;
    --z-popover: 300;
    --z-tooltip: 400;
}
```

---

## Quick Reference

### Tailwind + Rust Setup Checklist

```
□ npm init & install tailwindcss
□ Create tailwind.config.js with .rs in content
□ Create input.css with @tailwind directives
□ Configure Trunk.toml build hooks
□ Add CSS link to index.html
□ Test class detection in Rust files
```

### Common Tailwind Classes

```rust
// Layout
"flex items-center justify-between"
"grid grid-cols-3 gap-4"
"container mx-auto px-4"

// Spacing
"p-4 m-2 space-y-4"
"px-4 py-2 mt-4 mb-2"

// Typography
"text-lg font-bold text-gray-900"
"text-sm text-gray-500 leading-relaxed"

// Colors
"bg-white dark:bg-gray-800"
"text-blue-600 hover:text-blue-800"
"border-gray-200 dark:border-gray-700"

// Effects
"shadow-sm rounded-lg"
"transition-colors duration-200"
"opacity-50 cursor-not-allowed"
```

### CSS Custom Properties for WASM

```css
:root {
    /* Colors */
    --wasm-bg: #ffffff;
    --wasm-text: #1f2937;
    --wasm-primary: #3b82f6;

    /* Spacing */
    --wasm-space-sm: 0.5rem;
    --wasm-space-md: 1rem;

    /* Transitions */
    --wasm-transition: 200ms ease;
}

.dark {
    --wasm-bg: #1f2937;
    --wasm-text: #f9fafb;
}
```

### Build Commands

```bash
# Development
trunk serve
npx tailwindcss -i input.css -o output.css --watch

# Production
trunk build --release
npx tailwindcss -i input.css -o output.css --minify
```

---

## Sources

- [Tailwind CSS Documentation](https://tailwindcss.com/docs)
- [stylers Crate](https://docs.rs/stylers)
- [stylist Crate](https://docs.rs/stylist)
- [Leptos Styling Guide](https://leptos.dev/docs/view/styling)
- [Yew Styling](https://yew.rs/docs/more/css)
- [Trunk Asset Handling](https://trunkrs.dev/assets/)
- [CSS Custom Properties](https://developer.mozilla.org/en-US/docs/Web/CSS/Using_CSS_custom_properties)
