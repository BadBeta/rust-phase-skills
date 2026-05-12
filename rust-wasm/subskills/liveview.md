# LiveView Integration Subskill

> Quick reference for Phoenix LiveView + WASM integration.

## When to Activate

Activate when user asks about:
- Integrating WASM with Phoenix LiveView
- LiveView hooks for WASM components
- phx-update="ignore" pattern
- State synchronization between LiveView and WASM
- Orb (Elixir DSL for WebAssembly)
- Handling LiveView reconnection with WASM

## Full Reference

See `rust_wasm_liveview.md` for complete documentation.

## Basic Hook Pattern

```javascript
// assets/js/hooks.js
const WasmHook = {
    async mounted() {
        const wasm = await import('../pkg/app.js');
        await wasm.default();
        this.component = wasm.Component.new(this.el);

        // Listen for events from LiveView
        this.handleEvent("update_data", (data) => {
            this.component.update(data);
        });
    },

    updated() {
        // Called on LiveView patch - WASM handles its own DOM
    },

    destroyed() {
        this.component?.destroy();
    }
};

export default { WasmHook };
```

## LiveView Template

```elixir
def render(assigns) do
  ~H"""
  <div id="wasm-component" phx-hook="WasmHook" phx-update="ignore">
    <!-- WASM renders here -->
  </div>

  <button phx-click="send_to_wasm">Update WASM</button>
  """
end

def handle_event("send_to_wasm", _params, socket) do
  {:noreply, push_event(socket, "update_data", %{value: 42})}
end
```

## WASM Side (Rust)

```rust
#[wasm_bindgen]
pub struct Component {
    element: web_sys::Element,
}

#[wasm_bindgen]
impl Component {
    pub fn new(element: web_sys::Element) -> Self {
        element.set_inner_html("<div>WASM Loaded</div>");
        Self { element }
    }

    pub fn update(&mut self, data: JsValue) {
        let value: i32 = serde_wasm_bindgen::from_value(data).unwrap();
        self.element.set_inner_html(&format!("<div>Value: {}</div>", value));
    }

    pub fn destroy(&self) {
        self.element.set_inner_html("");
    }
}
```

## Key Patterns

1. **phx-update="ignore"** - Prevents LiveView from touching WASM-controlled DOM
2. **push_event** - Send data from LiveView to WASM
3. **pushEvent** - Send data from WASM to LiveView
4. **Handle reconnection** - Reinitialize WASM state on reconnect

## Orb (Elixir WASM)

```elixir
defmodule Counter do
  use Orb

  defw increment(value: I32), I32 do
    value + 1
  end
end

# Compile to WASM bytes
Counter.to_wat() |> Orb.to_wasm()
```
