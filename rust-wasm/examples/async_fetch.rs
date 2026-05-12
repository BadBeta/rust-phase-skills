// Async Operations and Fetch API
// Demonstrates async/await patterns in Rust WASM

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use serde::{Deserialize, Serialize};

// ========================================
// 1. Basic Fetch
// ========================================

#[wasm_bindgen]
pub async fn fetch_text(url: String) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or("No window")?;

    // Fetch the URL
    let response = JsFuture::from(window.fetch_with_str(&url)).await?;
    let response: Response = response.dyn_into()?;

    // Check status
    if !response.ok() {
        return Err(JsValue::from_str(&format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    // Get text body
    let text = JsFuture::from(response.text()?).await?;

    Ok(text.as_string().unwrap_or_default())
}

// ========================================
// 2. Fetch JSON with Serde
// ========================================

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub id: u32,
    pub name: String,
    pub email: String,
}

#[wasm_bindgen]
pub async fn fetch_user(user_id: u32) -> Result<JsValue, JsValue> {
    let url = format!("https://api.example.com/users/{}", user_id);

    let window = web_sys::window().ok_or("No window")?;
    let response = JsFuture::from(window.fetch_with_str(&url)).await?;
    let response: Response = response.dyn_into()?;

    if !response.ok() {
        return Err(JsValue::from_str("Failed to fetch user"));
    }

    let json = JsFuture::from(response.json()?).await?;
    let user: ApiResponse = serde_wasm_bindgen::from_value(json)?;

    serde_wasm_bindgen::to_value(&user)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ========================================
// 3. POST Request with JSON Body
// ========================================

#[derive(Serialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[wasm_bindgen]
pub async fn create_user(name: String, email: String) -> Result<JsValue, JsValue> {
    let request_body = CreateUserRequest { name, email };
    let body_json = serde_json::to_string(&request_body)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Create request options
    let mut opts = RequestInit::new();
    opts.method("POST");
    opts.mode(RequestMode::Cors);
    opts.body(Some(&JsValue::from_str(&body_json)));

    // Create request with headers
    let request = Request::new_with_str_and_init(
        "https://api.example.com/users",
        &opts
    )?;

    request.headers().set("Content-Type", "application/json")?;

    // Execute fetch
    let window = web_sys::window().ok_or("No window")?;
    let response = JsFuture::from(window.fetch_with_request(&request)).await?;
    let response: Response = response.dyn_into()?;

    if !response.ok() {
        let error_text = JsFuture::from(response.text()?).await?;
        return Err(JsValue::from_str(&format!(
            "API error: {}",
            error_text.as_string().unwrap_or_default()
        )));
    }

    JsFuture::from(response.json()?).await
}

// ========================================
// 4. Parallel Fetches
// ========================================

use futures::future::join_all;

#[wasm_bindgen]
pub async fn fetch_multiple(urls: Vec<String>) -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or("No window")?;

    // Create futures for all URLs
    let futures: Vec<_> = urls.iter().map(|url| async {
        let response = JsFuture::from(window.fetch_with_str(url)).await?;
        let response: Response = response.dyn_into()?;
        let text = JsFuture::from(response.text()?).await?;
        Ok::<String, JsValue>(text.as_string().unwrap_or_default())
    }).collect();

    // Wait for all to complete
    let results: Vec<Result<String, JsValue>> = join_all(futures).await;

    // Collect successful results
    let successful: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    serde_wasm_bindgen::to_value(&successful)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ========================================
// 5. Timeout Pattern
// ========================================

use gloo_timers::future::TimeoutFuture;
use futures::future::select;
use futures::pin_mut;

#[wasm_bindgen]
pub async fn fetch_with_timeout(url: String, timeout_ms: u32) -> Result<String, JsValue> {
    let window = web_sys::window().ok_or("No window")?;

    let fetch_future = async {
        let response = JsFuture::from(window.fetch_with_str(&url)).await?;
        let response: Response = response.dyn_into()?;
        let text = JsFuture::from(response.text()?).await?;
        Ok::<String, JsValue>(text.as_string().unwrap_or_default())
    };

    let timeout_future = async {
        TimeoutFuture::new(timeout_ms).await;
        Err::<String, JsValue>(JsValue::from_str("Request timed out"))
    };

    pin_mut!(fetch_future);
    pin_mut!(timeout_future);

    match select(fetch_future, timeout_future).await {
        futures::future::Either::Left((result, _)) => result,
        futures::future::Either::Right((result, _)) => result,
    }
}

// ========================================
// 6. AbortController for Cancellation
// ========================================

#[wasm_bindgen]
pub struct CancellableFetch {
    controller: web_sys::AbortController,
}

#[wasm_bindgen]
impl CancellableFetch {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<CancellableFetch, JsValue> {
        let controller = web_sys::AbortController::new()?;
        Ok(CancellableFetch { controller })
    }

    pub async fn fetch(&self, url: String) -> Result<String, JsValue> {
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.signal(Some(&self.controller.signal()));

        let request = Request::new_with_str_and_init(&url, &opts)?;

        let window = web_sys::window().ok_or("No window")?;
        let response = JsFuture::from(window.fetch_with_request(&request)).await?;
        let response: Response = response.dyn_into()?;

        let text = JsFuture::from(response.text()?).await?;
        Ok(text.as_string().unwrap_or_default())
    }

    pub fn abort(&self) {
        self.controller.abort();
    }
}

// ========================================
// Usage from JavaScript:
// ========================================
//
// // Basic fetch
// const text = await fetch_text("https://api.example.com/data");
//
// // Fetch JSON
// const user = await fetch_user(123);
// console.log(user.name);
//
// // POST request
// const newUser = await create_user("John", "john@example.com");
//
// // Parallel fetches
// const results = await fetch_multiple([
//     "https://api.example.com/1",
//     "https://api.example.com/2",
// ]);
//
// // With timeout
// try {
//     const data = await fetch_with_timeout("https://api.example.com", 5000);
// } catch (e) {
//     if (e === "Request timed out") { ... }
// }
//
// // Cancellable
// const request = new CancellableFetch();
// const promise = request.fetch("https://api.example.com");
// request.abort();  // Cancel if needed
