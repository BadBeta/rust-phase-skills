# Rust WebAssembly JavaScript Interoperability

> **Version**: 2025
> **Status**: Complete Reference

## Table of Contents
1. [Overview](#1-overview)
2. [wasm-bindgen Fundamentals](#2-wasm-bindgen-fundamentals)
3. [Exporting Rust to JavaScript](#3-exporting-rust-to-javascript)
4. [Importing JavaScript into Rust](#4-importing-javascript-into-rust)
5. [Type Conversions](#5-type-conversions)
6. [web-sys: DOM & Web APIs](#6-web-sys-dom--web-apis)
7. [js-sys: JavaScript Standard Library](#7-js-sys-javascript-standard-library)
8. [Closures & Callbacks](#8-closures--callbacks)
9. [Async & Promises](#9-async--promises)
10. [Error Handling](#10-error-handling)
11. [Memory Management](#11-memory-management)
12. [Raw WASM Exports](#12-raw-wasm-exports)
13. [Patterns](#13-patterns)
14. [Anti-Patterns](#14-anti-patterns)
15. [Common Failures & Solutions](#15-common-failures--solutions)
16. [Quick Reference](#16-quick-reference)

---

## 1. Overview

### 1.1 The Interop Stack

```
┌─────────────────────────────────────────────────────────────┐
│                      JavaScript                              │
├─────────────────────────────────────────────────────────────┤
│                 wasm-bindgen Glue Code                       │
│           (Auto-generated JS for type conversion)            │
├─────────────────────────────────────────────────────────────┤
│                    WebAssembly Module                        │
│                      (Rust → WASM)                           │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Key Crates

| Crate | Purpose |
|-------|---------|
| `wasm-bindgen` | Core interop layer, attribute macros |
| `js-sys` | Bindings to JS standard library (Array, Object, Promise, etc.) |
| `web-sys` | Bindings to Web APIs (DOM, fetch, WebGL, etc.) |
| `wasm-bindgen-futures` | Async/Promise interop |

### 1.3 Cargo.toml Setup

```toml
[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
wasm-bindgen-futures = "0.4"

[dependencies.web-sys]
version = "0.3"
features = [
    "console",
    "Window",
    "Document",
    "Element",
    "HtmlElement",
    "Node",
    "Event",
    "MouseEvent",
    "KeyboardEvent",
]
```

---

## 2. wasm-bindgen Fundamentals

### 2.1 The #[wasm_bindgen] Attribute

The `#[wasm_bindgen]` attribute is the core of interop:

```rust
use wasm_bindgen::prelude::*;

// Export a function to JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// Export a struct
#[wasm_bindgen]
pub struct Counter {
    value: i32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Counter { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}
```

### 2.2 Initialization Function

```rust
// Called automatically when WASM module loads
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();

    web_sys::console::log_1(&"WASM initialized!".into());
}
```

---

## 3. Exporting Rust to JavaScript

### 3.1 Functions

```rust
// Basic function
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// With custom JS name
#[wasm_bindgen(js_name = calculateSum)]
pub fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

// Returning Option
#[wasm_bindgen]
pub fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 { None } else { Some(a / b) }
}

// Returning Result
#[wasm_bindgen]
pub fn parse_number(s: &str) -> Result<i32, JsError> {
    s.parse().map_err(|e| JsError::new(&format!("{}", e)))
}
```

### 3.2 Structs with Methods

```rust
#[wasm_bindgen]
pub struct Rectangle {
    width: f64,
    height: f64,
}

#[wasm_bindgen]
impl Rectangle {
    // Constructor
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64) -> Self {
        Rectangle { width, height }
    }

    // Getters (become JS properties)
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> f64 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f64 {
        self.height
    }

    // Setters
    #[wasm_bindgen(setter)]
    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    // Regular methods
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    // Static method
    pub fn square(size: f64) -> Rectangle {
        Rectangle { width: size, height: size }
    }
}
```

**JavaScript Usage:**
```javascript
import { Rectangle } from './pkg/my_wasm.js';

const rect = new Rectangle(10, 20);
console.log(rect.width);      // 10 (getter)
rect.width = 15;              // setter
console.log(rect.area());     // 300

const square = Rectangle.square(5);
```

### 3.3 Enums

```rust
#[wasm_bindgen]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[wasm_bindgen]
pub fn color_to_hex(color: Color) -> String {
    match color {
        Color::Red => "#FF0000".to_string(),
        Color::Green => "#00FF00".to_string(),
        Color::Blue => "#0000FF".to_string(),
    }
}
```

**Note:** Only C-style enums (no data) are supported directly. For complex enums, use structs with type discriminators.

---

## 4. Importing JavaScript into Rust

### 4.1 Global Functions

```rust
#[wasm_bindgen]
extern "C" {
    // Import console.log
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // Import with different Rust name
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);

    // Multiple signatures for polymorphic functions
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(n: u32);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

// Usage
pub fn example() {
    log("Hello from Rust!");
    console_log("Using custom name");
    log_u32(42);
}
```

### 4.2 Nested Namespaces

```rust
#[wasm_bindgen]
extern "C" {
    // Import window.document.write
    #[wasm_bindgen(js_namespace = ["window", "document"])]
    fn write(s: &str);

    // Import from deep namespace
    #[wasm_bindgen(js_namespace = ["my", "app", "utils"])]
    fn helper_function() -> String;
}
```

### 4.3 JavaScript Classes

```rust
#[wasm_bindgen]
extern "C" {
    // Declare the type
    pub type MyJsClass;

    // Constructor
    #[wasm_bindgen(constructor)]
    fn new() -> MyJsClass;

    // Instance method
    #[wasm_bindgen(method)]
    fn do_something(this: &MyJsClass) -> String;

    // Method with custom JS name
    #[wasm_bindgen(method, js_name = getValue)]
    fn get_value(this: &MyJsClass) -> i32;

    // Getter
    #[wasm_bindgen(method, getter)]
    fn name(this: &MyJsClass) -> String;

    // Setter
    #[wasm_bindgen(method, setter)]
    fn set_name(this: &MyJsClass, name: &str);

    // Static method
    #[wasm_bindgen(static_method_of = MyJsClass)]
    fn create() -> MyJsClass;
}
```

### 4.4 Structural vs Final Imports

```rust
#[wasm_bindgen]
extern "C" {
    pub type Duck;

    // Structural (default): duck-typed, works with any object with this method
    #[wasm_bindgen(method, structural)]
    fn quack(this: &Duck);

    // Final: uses prototype chain, more efficient
    #[wasm_bindgen(method, final)]
    fn quack_final(this: &Duck);
}
```

---

## 5. Type Conversions

### 5.1 Primitive Type Mapping

| JavaScript | Rust | Notes |
|------------|------|-------|
| `number` | `i32`, `u32`, `i64`, `u64`, `f32`, `f64` | Direct mapping |
| `boolean` | `bool` | Direct mapping |
| `string` | `String`, `&str` | Copied (not zero-copy) |
| `undefined` | `()` | Unit type |
| `null` | `Option<T>` where `T: FromWasmAbi` | None for null |
| `BigInt` | `i64`, `u64`, `i128`, `u128` | For large integers |

### 5.2 JsValue - The Universal Type

```rust
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn accepts_any(val: JsValue) -> JsValue {
    // JsValue can hold any JS value
    if val.is_string() {
        JsValue::from_str("Got a string!")
    } else if val.is_object() {
        JsValue::from_str("Got an object!")
    } else {
        val
    }
}
```

### 5.3 JsCast - Type Casting

```rust
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, Element, HtmlInputElement};

fn example(element: Element) {
    // Checked cast (returns Result)
    match element.dyn_into::<HtmlInputElement>() {
        Ok(input) => {
            let value = input.value();
            // ...
        }
        Err(el) => {
            // Not an input, el is returned back
        }
    }

    // Checked reference cast (returns Option)
    if let Some(html_el) = element.dyn_ref::<HtmlElement>() {
        html_el.set_inner_text("Hello");
    }

    // Unchecked cast (zero-cost but unsafe if wrong type)
    let html_el: HtmlElement = element.unchecked_into();
}
```

### 5.4 Arrays and Typed Arrays

```rust
use js_sys::{Array, Uint8Array, Float32Array};

#[wasm_bindgen]
pub fn process_array(arr: Array) -> Array {
    let result = Array::new();
    for i in 0..arr.length() {
        let val = arr.get(i);
        result.push(&val);
    }
    result
}

#[wasm_bindgen]
pub fn process_bytes(data: &[u8]) -> Vec<u8> {
    // &[u8] is automatically converted from Uint8Array
    data.iter().map(|b| b.wrapping_add(1)).collect()
}

// For zero-copy access to typed arrays
#[wasm_bindgen]
pub fn sum_float32(arr: Float32Array) -> f32 {
    let mut sum = 0.0;
    arr.for_each(&mut |val, _, _| sum += val);
    sum
}
```

### 5.5 Objects and Serde

```rust
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct Person {
    name: String,
    age: u32,
}

#[wasm_bindgen]
pub fn parse_person(val: JsValue) -> Result<JsValue, JsError> {
    let person: Person = serde_wasm_bindgen::from_value(val)?;
    let modified = Person {
        name: person.name.to_uppercase(),
        age: person.age + 1,
    };
    Ok(serde_wasm_bindgen::to_value(&modified)?)
}
```

---

## 6. web-sys: DOM & Web APIs

### 6.1 Accessing the DOM

```rust
use wasm_bindgen::prelude::*;
use web_sys::{window, Document, Element, HtmlElement};

#[wasm_bindgen]
pub fn manipulate_dom() -> Result<(), JsValue> {
    // Get window and document
    let window = window().expect("no global window");
    let document = window.document().expect("no document");

    // Query elements
    let element = document
        .get_element_by_id("my-element")
        .expect("element not found");

    // Cast to specific type
    let html_element: HtmlElement = element.dyn_into()?;

    // Manipulate
    html_element.set_inner_text("Hello from Rust!");
    html_element.style().set_property("color", "red")?;

    // Create new element
    let new_div = document.create_element("div")?;
    new_div.set_text_content(Some("New element"));
    new_div.set_class_name("my-class");

    // Append to body
    document.body().unwrap().append_child(&new_div)?;

    Ok(())
}
```

### 6.2 Event Handling

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlElement, MouseEvent};

#[wasm_bindgen]
pub fn setup_click_handler(element_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let element = document.get_element_by_id(element_id).unwrap();
    let html_element: HtmlElement = element.dyn_into()?;

    // Create closure for event handler
    let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
        let x = event.client_x();
        let y = event.client_y();
        web_sys::console::log_1(&format!("Clicked at ({}, {})", x, y).into());
    }) as Box<dyn FnMut(MouseEvent)>);

    // Add event listener
    html_element.add_event_listener_with_callback(
        "click",
        closure.as_ref().unchecked_ref()
    )?;

    // Prevent closure from being dropped
    closure.forget();

    Ok(())
}
```

### 6.3 Fetch API

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[wasm_bindgen]
pub async fn fetch_data(url: &str) -> Result<JsValue, JsValue> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    let json = JsFuture::from(resp.json()?).await?;
    Ok(json)
}
```

### 6.4 Canvas

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[wasm_bindgen]
pub fn draw_on_canvas(canvas_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;

    // Draw rectangle
    context.set_fill_style_str("red");
    context.fill_rect(10.0, 10.0, 100.0, 100.0);

    // Draw text
    context.set_font("30px Arial");
    context.set_fill_style_str("blue");
    context.fill_text("Hello!", 50.0, 50.0)?;

    Ok(())
}
```

---

## 7. js-sys: JavaScript Standard Library

### 7.1 Array Operations

```rust
use js_sys::Array;

#[wasm_bindgen]
pub fn array_operations() -> Array {
    let arr = Array::new();

    // Push items
    arr.push(&1.into());
    arr.push(&2.into());
    arr.push(&3.into());

    // Map
    let doubled = arr.map(&mut |val, _, _| {
        let n: f64 = val.as_f64().unwrap();
        JsValue::from(n * 2.0)
    });

    // Filter
    let filtered = arr.filter(&mut |val, _, _| {
        val.as_f64().unwrap() > 1.0
    });

    // Reduce
    let sum = arr.reduce(&mut |acc, val, _, _| {
        let a = acc.as_f64().unwrap_or(0.0);
        let v = val.as_f64().unwrap();
        JsValue::from(a + v)
    }, &JsValue::from(0));

    doubled
}
```

### 7.2 Object Operations

```rust
use js_sys::{Object, Reflect};

#[wasm_bindgen]
pub fn object_operations() -> Object {
    let obj = Object::new();

    // Set properties
    Reflect::set(&obj, &"name".into(), &"Alice".into()).unwrap();
    Reflect::set(&obj, &"age".into(), &30.into()).unwrap();

    // Get properties
    let name = Reflect::get(&obj, &"name".into()).unwrap();

    // Check property exists
    let has_name = Reflect::has(&obj, &"name".into()).unwrap();

    // Get all keys
    let keys = Object::keys(&obj);

    // Get all values
    let values = Object::values(&obj);

    // Get entries
    let entries = Object::entries(&obj);

    obj
}
```

### 7.3 Date Operations

```rust
use js_sys::Date;

#[wasm_bindgen]
pub fn date_operations() -> Date {
    // Current date
    let now = Date::new_0();

    // Specific date
    let specific = Date::new_with_year_month_day(2025, 0, 1); // Jan 1, 2025

    // Get components
    let year = now.get_full_year();
    let month = now.get_month();
    let day = now.get_date();
    let hours = now.get_hours();

    // Get timestamp
    let timestamp = now.get_time();

    // Format
    let iso_string = now.to_iso_string();

    now
}
```

### 7.4 RegExp

```rust
use js_sys::RegExp;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
pub fn regex_test(pattern: &str, text: &str) -> bool {
    let regex = RegExp::new(pattern, "gi");
    regex.test(text)
}

#[wasm_bindgen]
pub fn regex_match(pattern: &str, text: &str) -> JsValue {
    let regex = RegExp::new(pattern, "g");
    regex.exec(text).map(|a| a.into()).unwrap_or(JsValue::NULL)
}
```

---

## 8. Closures & Callbacks

### 8.1 Basic Closures

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::closure::Closure;

// Closure that can be called multiple times
#[wasm_bindgen]
pub fn create_callback() -> JsValue {
    let closure = Closure::wrap(Box::new(move |x: i32| -> i32 {
        x * 2
    }) as Box<dyn Fn(i32) -> i32>);

    let js_val = closure.as_ref().clone();
    closure.forget(); // Prevent drop, leak memory
    js_val
}

// Closure with mutable state
#[wasm_bindgen]
pub fn create_counter() -> JsValue {
    let mut count = 0;

    let closure = Closure::wrap(Box::new(move || -> i32 {
        count += 1;
        count
    }) as Box<dyn FnMut() -> i32>);

    let js_val = closure.as_ref().clone();
    closure.forget();
    js_val
}
```

### 8.2 One-Time Closures (FnOnce)

```rust
use wasm_bindgen::closure::Closure;

#[wasm_bindgen]
pub fn create_one_time_callback() -> JsValue {
    let data = vec![1, 2, 3, 4, 5]; // Owned data

    // This closure can only be called once
    let closure = Closure::once(move || {
        // data is moved here
        data.iter().sum::<i32>()
    });

    closure.into_js_value()
}

// Alternative: once_into_js
#[wasm_bindgen]
pub fn create_once_callback() -> JsValue {
    let data = "Hello".to_string();

    Closure::once_into_js(move || {
        format!("{} World!", data)
    })
}
```

### 8.3 Event Listener Pattern

```rust
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

#[wasm_bindgen]
pub struct EventManager {
    closures: Rc<RefCell<Vec<Closure<dyn FnMut(web_sys::Event)>>>>,
}

#[wasm_bindgen]
impl EventManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        EventManager {
            closures: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn add_click_handler(
        &self,
        element: HtmlElement,
        callback: js_sys::Function,
    ) -> Result<(), JsValue> {
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let _ = callback.call1(&JsValue::NULL, &event);
        }) as Box<dyn FnMut(web_sys::Event)>);

        element.add_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref()
        )?;

        // Store closure to prevent drop
        self.closures.borrow_mut().push(closure);

        Ok(())
    }

    pub fn cleanup(&self) {
        self.closures.borrow_mut().clear();
    }
}
```

### 8.4 Receiving JS Closures in Rust

```rust
use js_sys::Function;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn call_js_function(func: &Function, arg: i32) -> Result<JsValue, JsValue> {
    // Call with single argument
    func.call1(&JsValue::NULL, &arg.into())
}

#[wasm_bindgen]
pub fn call_with_multiple_args(func: &Function) -> Result<JsValue, JsValue> {
    // Call with multiple arguments
    let args = js_sys::Array::new();
    args.push(&1.into());
    args.push(&2.into());
    args.push(&3.into());

    func.apply(&JsValue::NULL, &args)
}
```

---

## 9. Async & Promises

### 9.1 Exporting Async Functions

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

// Async function returns Promise to JS
#[wasm_bindgen]
pub async fn async_operation() -> Result<String, JsError> {
    // Simulate async work
    let result = do_async_work().await?;
    Ok(result)
}

// With explicit Promise return
#[wasm_bindgen]
pub fn get_data_promise() -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async {
        let data = fetch_something().await?;
        Ok(JsValue::from_str(&data))
    })
}
```

### 9.2 Consuming JS Promises

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use js_sys::Promise;

#[wasm_bindgen]
pub async fn consume_promise(promise: Promise) -> Result<JsValue, JsValue> {
    // Convert Promise to Future and await
    let result = JsFuture::from(promise).await?;
    Ok(result)
}

// Importing async JS function
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn fetch_api(url: &str) -> Result<JsValue, JsValue>;
}
```

### 9.3 spawn_local for Fire-and-Forget

```rust
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
pub fn start_background_task() {
    spawn_local(async {
        // This runs in background, no Promise returned
        loop {
            do_periodic_work().await;
            sleep_ms(1000).await;
        }
    });
}

async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        let window = web_sys::window().unwrap();
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            &resolve, ms
        ).unwrap();
    });
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}
```

### 9.4 Promise.all Equivalent

```rust
use js_sys::{Array, Promise};
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
pub async fn wait_for_all(promises: Array) -> Result<Array, JsValue> {
    let promise_all = Promise::all(&promises);
    let result = JsFuture::from(promise_all).await?;
    Ok(result.into())
}
```

---

## 10. Error Handling

### 10.1 JsError for Simple Errors

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn may_fail(input: &str) -> Result<String, JsError> {
    if input.is_empty() {
        return Err(JsError::new("Input cannot be empty"));
    }
    Ok(format!("Processed: {}", input))
}

// Convert from std::error::Error
#[wasm_bindgen]
pub fn parse_json(json: &str) -> Result<JsValue, JsError> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    Ok(serde_wasm_bindgen::to_value(&value)?)
}
```

### 10.2 Catching JS Exceptions

```rust
#[wasm_bindgen]
extern "C" {
    // Without catch: exception propagates, may crash WASM
    fn risky_operation();

    // With catch: exception becomes Result::Err
    #[wasm_bindgen(catch)]
    fn safe_operation() -> Result<JsValue, JsValue>;

    // Async with catch
    #[wasm_bindgen(catch)]
    async fn async_safe() -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub fn call_safely() -> Result<JsValue, JsValue> {
    // This won't crash if JS throws
    safe_operation()
}
```

### 10.3 Custom Error Types

```rust
use wasm_bindgen::prelude::*;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InvalidInput(String),
    NetworkError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<AppError> for JsError {
    fn from(err: AppError) -> Self {
        JsError::new(&err.to_string())
    }
}

#[wasm_bindgen]
pub fn process(id: &str) -> Result<String, JsError> {
    if id.is_empty() {
        return Err(AppError::InvalidInput("ID required".into()).into());
    }
    Ok(format!("Processed {}", id))
}
```

---

## 11. Memory Management

### 11.1 Understanding the Cost

**String passing is expensive:**
```rust
// Each call copies string from JS heap to WASM linear memory
// O(n) copy + UTF-8 to UTF-16 transcoding
#[wasm_bindgen]
pub fn process_string(s: &str) -> String {
    s.to_uppercase()
}
```

**Typed arrays can be more efficient:**
```rust
use js_sys::Uint8Array;

// Reference to JS memory - no copy for reading
#[wasm_bindgen]
pub fn sum_bytes(arr: &Uint8Array) -> u32 {
    let mut sum: u32 = 0;
    arr.for_each(&mut |val, _, _| sum += val as u32);
    sum
}

// Copy to Rust - required for mutation
#[wasm_bindgen]
pub fn process_bytes(arr: &[u8]) -> Vec<u8> {
    arr.iter().map(|b| b.wrapping_add(1)).collect()
}
```

### 11.2 Batch Operations

```rust
// BAD: Many small calls
#[wasm_bindgen]
pub fn process_one(x: f64) -> f64 {
    x * 2.0
}

// GOOD: Batch processing
#[wasm_bindgen]
pub fn process_many(data: &[f64]) -> Vec<f64> {
    data.iter().map(|x| x * 2.0).collect()
}
```

### 11.3 Shared Memory (Advanced)

```rust
use wasm_bindgen::prelude::*;
use js_sys::SharedArrayBuffer;

#[wasm_bindgen]
pub fn setup_shared_buffer(size: usize) -> SharedArrayBuffer {
    SharedArrayBuffer::new(size as u32)
}

// Process shared memory directly - zero copy
#[wasm_bindgen]
pub fn process_shared(buffer: &SharedArrayBuffer, offset: usize, len: usize) {
    // Access shared memory...
}
```

---

## 12. Raw WASM Exports

### 12.1 Without wasm-bindgen

For maximum performance, bypass wasm-bindgen:

```rust
// lib.rs
#![no_std]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Raw export - only numeric types
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

// For arrays, work with pointers
#[no_mangle]
pub extern "C" fn sum_array(ptr: *const f32, len: usize) -> f32 {
    let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
    slice.iter().sum()
}

// Allocator for memory management
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(ptr, 0, size);
    }
}
```

**JavaScript usage:**
```javascript
const result = await WebAssembly.instantiateStreaming(fetch('module.wasm'));
const { add, sum_array, alloc, memory } = result.instance.exports;

console.log(add(1, 2)); // 3

// For arrays
const data = new Float32Array([1, 2, 3, 4, 5]);
const ptr = alloc(data.byteLength);
new Float32Array(memory.buffer, ptr, data.length).set(data);
const sum = sum_array(ptr, data.length);
```

### 12.2 When to Use Raw Exports

| Use Case | Approach |
|----------|----------|
| Simple numeric operations | Raw exports |
| Hot inner loops | Raw exports + SharedArrayBuffer |
| Complex APIs | wasm-bindgen |
| DOM interaction | wasm-bindgen + web-sys |
| General application code | wasm-bindgen |

---

## 13. Patterns

### Pattern 1: Builder Pattern for Complex Objects

```rust
#[wasm_bindgen]
pub struct RequestBuilder {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

#[wasm_bindgen]
impl RequestBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new(url: &str) -> Self {
        RequestBuilder {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: Vec::new(),
            body: None,
        }
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub async fn send(self) -> Result<JsValue, JsValue> {
        // Execute request...
        Ok(JsValue::NULL)
    }
}
```

### Pattern 2: Module Pattern

```rust
// Organize related functionality
#[wasm_bindgen]
pub struct StringUtils;

#[wasm_bindgen]
impl StringUtils {
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().chain(chars).collect(),
        }
    }

    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }
}
```

### Pattern 3: Event Emitter

```rust
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub struct EventEmitter {
    listeners: Rc<RefCell<HashMap<String, Vec<js_sys::Function>>>>,
}

#[wasm_bindgen]
impl EventEmitter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        EventEmitter {
            listeners: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn on(&self, event: &str, callback: js_sys::Function) {
        self.listeners
            .borrow_mut()
            .entry(event.to_string())
            .or_default()
            .push(callback);
    }

    pub fn emit(&self, event: &str, data: JsValue) {
        if let Some(callbacks) = self.listeners.borrow().get(event) {
            for callback in callbacks {
                let _ = callback.call1(&JsValue::NULL, &data);
            }
        }
    }

    pub fn off(&self, event: &str) {
        self.listeners.borrow_mut().remove(event);
    }
}
```

### Pattern 4: Lazy Initialization

```rust
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

static CONFIG: OnceLock<Config> = OnceLock::new();

struct Config {
    api_url: String,
    debug: bool,
}

#[wasm_bindgen]
pub fn initialize(api_url: &str, debug: bool) {
    CONFIG.get_or_init(|| Config {
        api_url: api_url.to_string(),
        debug,
    });
}

#[wasm_bindgen]
pub fn get_api_url() -> String {
    CONFIG.get()
        .map(|c| c.api_url.clone())
        .unwrap_or_default()
}
```

### Pattern 5: Stream Processing

```rust
#[wasm_bindgen]
pub struct DataProcessor {
    buffer: Vec<f64>,
    window_size: usize,
}

#[wasm_bindgen]
impl DataProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new(window_size: usize) -> Self {
        DataProcessor {
            buffer: Vec::with_capacity(window_size),
            window_size,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        self.buffer.push(value);

        if self.buffer.len() >= self.window_size {
            let avg = self.buffer.iter().sum::<f64>() / self.buffer.len() as f64;
            self.buffer.remove(0);
            Some(avg)
        } else {
            None
        }
    }
}
```

---

## 14. Anti-Patterns

### Anti-Pattern 1: Frequent Boundary Crossings

```rust
// BAD: Many small calls
for item in items {
    process_single_item(item); // JS→WASM→JS for each!
}

// GOOD: Batch processing
process_all_items(&items); // Single JS→WASM→JS
```

### Anti-Pattern 2: Forgetting All Closures

```rust
// BAD: Memory leak
let closure = Closure::wrap(/* ... */);
closure.forget(); // Leaked forever!

// GOOD: Store and clean up
struct Handler {
    closure: Option<Closure<dyn FnMut()>>,
}

impl Drop for Handler {
    fn drop(&mut self) {
        // Closure dropped, JS callback invalidated
    }
}
```

### Anti-Pattern 3: Not Using catch

```rust
// BAD: WASM may crash if JS throws
#[wasm_bindgen]
extern "C" {
    fn might_throw();
}

// GOOD: Handle exceptions
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch)]
    fn might_throw() -> Result<(), JsValue>;
}
```

### Anti-Pattern 4: Blocking Async Operations

```rust
// BAD: This won't work - can't block in WASM
pub fn bad_sync_fetch() -> String {
    block_on(async_fetch()) // Not available!
}

// GOOD: Stay async
#[wasm_bindgen]
pub async fn good_fetch() -> Result<String, JsError> {
    async_fetch().await
}
```

### Anti-Pattern 5: String-Heavy APIs

```rust
// BAD: Expensive string operations
#[wasm_bindgen]
pub fn get_char(s: &str, i: usize) -> String {
    s.chars().nth(i).map(|c| c.to_string()).unwrap_or_default()
}

// GOOD: Work with bytes or indices
#[wasm_bindgen]
pub fn get_char_code(s: &str, i: usize) -> Option<u32> {
    s.chars().nth(i).map(|c| c as u32)
}
```

---

## 15. Common Failures & Solutions

### Failure 1: JsCast Type Mismatch

```
Error: null passed to Rust (JsCast failed)
```

**Solution:**
```rust
// Use dyn_ref for optional casts
if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
    // Safe to use
}

// Or handle the Result
match element.dyn_into::<HtmlInputElement>() {
    Ok(input) => { /* use input */ }
    Err(el) => { /* handle wrong type */ }
}
```

### Failure 2: Closure Already Dropped

```
Error: closure invoked after being dropped
```

**Solution:**
```rust
// Store closure reference
struct Handler {
    _closure: Closure<dyn FnMut(Event)>,
}

// Or use forget (but beware memory leak)
closure.forget();
```

### Failure 3: Missing web-sys Feature

```
error: no method named `inner_text` found
```

**Solution:**
```toml
# Add required feature
[dependencies.web-sys]
features = ["HtmlElement"]  # Add the missing feature
```

### Failure 4: Async Without wasm-bindgen-futures

```
error: `async fn` is not allowed in `extern` blocks
```

**Solution:**
```toml
[dependencies]
wasm-bindgen-futures = "0.4"
```

```rust
use wasm_bindgen_futures::JsFuture;
```

### Failure 5: Result Type Incompatibility

```
error: the trait `Into<JsValue>` is not implemented
```

**Solution:**
```rust
// Use JsError instead of custom error
pub fn operation() -> Result<String, JsError> {
    // ...
}

// Or implement conversion
impl From<MyError> for JsValue {
    fn from(err: MyError) -> Self {
        JsValue::from_str(&err.to_string())
    }
}
```

---

## 16. Quick Reference

### Import Patterns

```rust
#[wasm_bindgen]
extern "C" {
    // Global function
    fn alert(s: &str);

    // Namespaced function
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // Class type
    type MyClass;

    // Constructor
    #[wasm_bindgen(constructor)]
    fn new() -> MyClass;

    // Instance method
    #[wasm_bindgen(method)]
    fn do_thing(this: &MyClass);

    // Static method
    #[wasm_bindgen(static_method_of = MyClass)]
    fn create() -> MyClass;

    // Getter/Setter
    #[wasm_bindgen(method, getter)]
    fn value(this: &MyClass) -> i32;

    // Catch exceptions
    #[wasm_bindgen(catch)]
    fn risky() -> Result<(), JsValue>;

    // Async
    #[wasm_bindgen(catch)]
    async fn fetch_data() -> Result<JsValue, JsValue>;
}
```

### Export Patterns

```rust
// Function
#[wasm_bindgen]
pub fn my_function(arg: i32) -> i32 { arg }

// Struct
#[wasm_bindgen]
pub struct MyStruct { field: i32 }

#[wasm_bindgen]
impl MyStruct {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Self { field: 0 } }

    #[wasm_bindgen(getter)]
    pub fn field(&self) -> i32 { self.field }

    #[wasm_bindgen(setter)]
    pub fn set_field(&mut self, v: i32) { self.field = v; }
}

// Async
#[wasm_bindgen]
pub async fn async_fn() -> Result<JsValue, JsError> {
    Ok(JsValue::NULL)
}
```

### Common Features (web-sys)

```toml
features = [
    "console", "Window", "Document", "Element",
    "HtmlElement", "HtmlInputElement", "HtmlCanvasElement",
    "Node", "Event", "MouseEvent", "KeyboardEvent",
    "EventTarget", "CanvasRenderingContext2d",
    "Request", "RequestInit", "Response", "Headers",
]
```

---

## Sources

- [wasm-bindgen Guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [web-sys Documentation](https://rustwasm.github.io/wasm-bindgen/api/web_sys/)
- [js-sys Documentation](https://docs.rs/js-sys/latest/js_sys/)
- [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures)
- [Closures Example](https://rustwasm.github.io/docs/wasm-bindgen/examples/closures.html)
- [DOM Hello World](https://rustwasm.github.io/docs/wasm-bindgen/examples/dom.html)
- [Promises and Futures](https://rustwasm.github.io/docs/wasm-bindgen/reference/js-promises-and-rust-futures.html)
- [Rust to WebAssembly the Hard Way](https://surma.dev/things/rust-to-webassembly/)

---

*Document completed: Step 3 of Rust WebAssembly Skill Research*
