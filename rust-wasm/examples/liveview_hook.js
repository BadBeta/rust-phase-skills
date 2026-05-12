// Phoenix LiveView WASM Hook
// Complete example of integrating Rust WASM with LiveView

// ========================================
// 1. Hook Definition
// ========================================

const WasmHook = {
    // Called when the element is added to the DOM
    async mounted() {
        console.log('[WasmHook] Mounting...');

        try {
            // Dynamic import of WASM module
            const wasm = await import('./pkg/app.js');
            await wasm.default();

            // Store reference for cleanup
            this.wasm = wasm;

            // Initialize WASM component with the hook element
            this.component = wasm.Component.new(this.el, {
                theme: this.el.dataset.theme || 'light',
                initialValue: parseInt(this.el.dataset.value) || 0,
            });

            // Remove loading state
            this.el.classList.remove('loading');

            console.log('[WasmHook] Mounted successfully');
        } catch (error) {
            console.error('[WasmHook] Failed to load WASM:', error);
            this.el.innerHTML = '<p class="error">Failed to load component</p>';
        }

        // ========================================
        // Event Handlers: LiveView → WASM
        // ========================================

        // Handle custom events from server
        this.handleEvent('update_value', (data) => {
            console.log('[WasmHook] Received update_value:', data);
            this.component?.updateValue(data.value);
        });

        this.handleEvent('set_theme', (data) => {
            console.log('[WasmHook] Received set_theme:', data);
            this.component?.setTheme(data.theme);
        });

        this.handleEvent('reset', () => {
            console.log('[WasmHook] Received reset');
            this.component?.reset();
        });
    },

    // Called when LiveView patches the DOM
    // Note: With phx-update="ignore", this is rarely called
    updated() {
        console.log('[WasmHook] Updated');

        // If data attributes changed, update WASM
        const newTheme = this.el.dataset.theme;
        if (newTheme && this.component) {
            this.component.setTheme(newTheme);
        }
    },

    // Called when the element is removed
    destroyed() {
        console.log('[WasmHook] Destroying...');

        // Clean up WASM resources
        if (this.component) {
            this.component.destroy();
            this.component = null;
        }

        console.log('[WasmHook] Destroyed');
    },

    // Called when LiveView disconnects
    disconnected() {
        console.log('[WasmHook] Disconnected');
        // Optionally pause WASM operations
        this.component?.pause?.();
    },

    // Called when LiveView reconnects
    reconnected() {
        console.log('[WasmHook] Reconnected');
        // Resume WASM operations and sync state
        this.component?.resume?.();

        // Request fresh data from server
        this.pushEvent('request_sync', {});
    },
};

// ========================================
// 2. WASM → LiveView Communication
// ========================================

// Set up event listener for WASM to push events
// This is called from Rust via wasm-bindgen
window.wasmPushEvent = (hookEl, event, payload) => {
    const hook = hookEl.__phx_hook__;
    if (hook) {
        hook.pushEvent(event, payload);
    }
};

// Alternative: Push to specific LiveView target
window.wasmPushEventTo = (hookEl, selector, event, payload) => {
    const hook = hookEl.__phx_hook__;
    if (hook) {
        hook.pushEventTo(selector, event, payload);
    }
};

// ========================================
// 3. Export Hooks
// ========================================

export default {
    WasmHook,
};

// ========================================
// 4. Elixir LiveView Template
// ========================================
/*
defmodule MyAppWeb.WasmLive do
  use MyAppWeb, :live_view

  def mount(_params, _session, socket) do
    {:ok, assign(socket, value: 0, theme: "light")}
  end

  def render(assigns) do
    ~H"""
    <div class="wasm-container">
      <%!-- WASM-controlled area --%>
      <div
        id="wasm-component"
        phx-hook="WasmHook"
        phx-update="ignore"
        data-theme={@theme}
        data-value={@value}
        class="loading"
      >
        <div class="loading-spinner">Loading...</div>
      </div>

      <%!-- LiveView controls --%>
      <div class="controls mt-4">
        <button phx-click="increment">Increment</button>
        <button phx-click="toggle_theme">Toggle Theme</button>
      </div>
    </div>
    """
  end

  def handle_event("increment", _params, socket) do
    new_value = socket.assigns.value + 1
    {:noreply,
      socket
      |> assign(:value, new_value)
      |> push_event("update_value", %{value: new_value})}
  end

  def handle_event("toggle_theme", _params, socket) do
    new_theme = if socket.assigns.theme == "light", do: "dark", else: "light"
    {:noreply,
      socket
      |> assign(:theme, new_theme)
      |> push_event("set_theme", %{theme: new_theme})}
  end

  # Handle events from WASM
  def handle_event("wasm_action", %{"action" => action, "data" => data}, socket) do
    # Process WASM events
    {:noreply, socket}
  end

  def handle_event("request_sync", _params, socket) do
    {:noreply,
      socket
      |> push_event("update_value", %{value: socket.assigns.value})
      |> push_event("set_theme", %{theme: socket.assigns.theme})}
  end
end
*/

// ========================================
// 5. Rust WASM Component (corresponding)
// ========================================
/*
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Component {
    element: web_sys::Element,
    value: i32,
    theme: String,
}

#[wasm_bindgen]
impl Component {
    pub fn new(element: web_sys::Element, options: JsValue) -> Component {
        let opts: Options = serde_wasm_bindgen::from_value(options).unwrap();

        let component = Component {
            element,
            value: opts.initial_value,
            theme: opts.theme,
        };

        component.render();
        component
    }

    #[wasm_bindgen(js_name = updateValue)]
    pub fn update_value(&mut self, value: i32) {
        self.value = value;
        self.render();
    }

    #[wasm_bindgen(js_name = setTheme)]
    pub fn set_theme(&mut self, theme: String) {
        self.theme = theme;
        self.render();
    }

    pub fn reset(&mut self) {
        self.value = 0;
        self.render();
    }

    pub fn destroy(&self) {
        self.element.set_inner_html("");
    }

    fn render(&self) {
        let html = format!(
            r#"<div class="wasm-component theme-{}">
                <span class="value">{}</span>
            </div>"#,
            self.theme, self.value
        );
        self.element.set_inner_html(&html);
    }

    // Push event to LiveView
    fn push_to_liveview(&self, event: &str, payload: JsValue) {
        js_sys::Reflect::get(&web_sys::window().unwrap(), &"wasmPushEvent".into())
            .ok()
            .and_then(|f| f.dyn_ref::<js_sys::Function>().cloned())
            .map(|f| f.call3(&JsValue::NULL, &self.element, &event.into(), &payload));
    }
}

#[derive(serde::Deserialize)]
struct Options {
    theme: String,
    #[serde(rename = "initialValue")]
    initial_value: i32,
}
*/
