// Minimal Leptos WASM Application
// Cargo.toml dependencies:
//   leptos = { version = "0.7", features = ["csr"] }
//   wasm-bindgen = "0.2"
//   console_error_panic_hook = "0.1"

use leptos::*;

// Entry point
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

// Root component
#[component]
fn App() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <main class="container mx-auto p-8">
            <h1 class="text-3xl font-bold mb-4">"Leptos Counter"</h1>

            <div class="flex items-center gap-4">
                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                    on:click=move |_| set_count.update(|c| *c -= 1)
                >
                    "-"
                </button>

                <span class="text-2xl font-mono w-16 text-center">
                    {count}
                </span>

                <button
                    class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
                    on:click=move |_| set_count.update(|c| *c += 1)
                >
                    "+"
                </button>
            </div>

            <p class="mt-4 text-gray-600">
                "Count is: " {count}
            </p>
        </main>
    }
}
