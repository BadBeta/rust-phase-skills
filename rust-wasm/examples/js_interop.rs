// JavaScript Interop Examples
// Demonstrates wasm-bindgen patterns for JS integration

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, Document, Element, Window};

// ========================================
// 1. Simple Function Exports
// ========================================

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

// ========================================
// 2. Struct with Methods
// ========================================

#[wasm_bindgen]
pub struct Counter {
    value: i32,
    step: i32,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new(initial: i32) -> Counter {
        Counter { value: initial, step: 1 }
    }

    #[wasm_bindgen(js_name = setStep)]
    pub fn set_step(&mut self, step: i32) {
        self.step = step;
    }

    pub fn increment(&mut self) {
        self.value += self.step;
    }

    pub fn decrement(&mut self) {
        self.value -= self.step;
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> i32 {
        self.value
    }
}

// ========================================
// 3. JavaScript Imports
// ========================================

#[wasm_bindgen]
extern "C" {
    // Console methods
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);

    // Math methods
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;

    #[wasm_bindgen(js_namespace = Math)]
    fn floor(x: f64) -> f64;

    // Alert dialog
    fn alert(s: &str);

    // Custom JS function (defined in JS)
    #[wasm_bindgen(js_name = myCustomFunction)]
    fn my_custom_function(x: i32) -> i32;
}

#[wasm_bindgen]
pub fn demo_imports() {
    log("Hello from WASM!");
    log_many("Random number:", &random().to_string());

    let random_int = floor(random() * 100.0) as i32;
    log(&format!("Random int: {}", random_int));
}

// ========================================
// 4. DOM Manipulation
// ========================================

#[wasm_bindgen]
pub fn create_element(tag: &str, content: &str) -> Result<Element, JsValue> {
    let window: Window = web_sys::window().ok_or("No window")?;
    let document: Document = window.document().ok_or("No document")?;

    let element = document.create_element(tag)?;
    element.set_inner_html(content);

    Ok(element)
}

#[wasm_bindgen]
pub fn append_to_body(element: &Element) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let body = document.body().ok_or("No body")?;

    body.append_child(element)?;
    Ok(())
}

#[wasm_bindgen]
pub fn query_selector(selector: &str) -> Option<Element> {
    let window = web_sys::window()?;
    let document = window.document()?;
    document.query_selector(selector).ok().flatten()
}

// ========================================
// 5. Error Handling
// ========================================

#[wasm_bindgen]
pub fn safe_divide(a: f64, b: f64) -> Result<f64, JsValue> {
    if b == 0.0 {
        Err(JsValue::from_str("Division by zero"))
    } else {
        Ok(a / b)
    }
}

// ========================================
// 6. Working with Arrays
// ========================================

#[wasm_bindgen]
pub fn sum_array(arr: &[i32]) -> i32 {
    arr.iter().sum()
}

#[wasm_bindgen]
pub fn double_array(arr: &[u8]) -> Vec<u8> {
    arr.iter().map(|x| x.wrapping_mul(2)).collect()
}

// ========================================
// 7. JSON/Serde Integration
// ========================================

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub email: String,
}

#[wasm_bindgen]
pub fn parse_user(json: &str) -> Result<JsValue, JsValue> {
    let user: User = serde_json::from_str(json)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&user)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn create_user(data: JsValue) -> Result<String, JsValue> {
    let user: User = serde_wasm_bindgen::from_value(data)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_string(&user)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ========================================
// Usage from JavaScript:
// ========================================
//
// import init, { add, greet, Counter, demo_imports } from './pkg/app.js';
//
// await init();
//
// console.log(add(2, 3));  // 5
// console.log(greet("World"));  // "Hello, World!"
//
// const counter = new Counter(10);
// counter.increment();
// console.log(counter.value);  // 11
//
// demo_imports();  // Logs to console
