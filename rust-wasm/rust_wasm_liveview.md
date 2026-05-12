# Rust WebAssembly + Phoenix LiveView Integration

> **Version**: 2025
> **Status**: Complete Reference

## Table of Contents
1. [Overview](#1-overview)
2. [LiveView Hook Architecture](#2-liveview-hook-architecture)
3. [WASM Hook Integration](#3-wasm-hook-integration)
4. [Event Communication](#4-event-communication)
5. [phx-update="ignore" Pattern](#5-phx-updateignore-pattern)
6. [Orb: Elixir-Authored WebAssembly](#6-orb-elixir-authored-webassembly)
7. [State Synchronization](#7-state-synchronization)
8. [Connection Handling](#8-connection-handling)
9. [Asset Management](#9-asset-management)
10. [Patterns](#10-patterns)
11. [Anti-Patterns](#11-anti-patterns)
12. [Common Failures & Solutions](#12-common-failures--solutions)
13. [Quick Reference](#13-quick-reference)

---

## 1. Overview

### 1.1 Integration Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
│  ┌─────────────────┐    ┌──────────────────────────────────┐   │
│  │   LiveView JS   │◄──►│         WASM Module              │   │
│  │     Client      │    │    (Rust via wasm-bindgen)       │   │
│  └────────┬────────┘    └──────────────┬───────────────────┘   │
│           │                            │                        │
│           │    ┌───────────────────────┘                        │
│           │    │ JS Hook (phx-hook)                             │
│           │    │ • mounted: init WASM                           │
│           │    │ • handleEvent: receive from server             │
│           │    │ • pushEvent: send to server                    │
│           ▼    ▼                                                │
└───────────WebSocket─────────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Phoenix Server                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    LiveView Process                      │   │
│  │  • handle_event/3: receive from client                   │   │
│  │  • push_event/3: send to client                          │   │
│  │  • assign/3: update state → re-render                    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Integration Approaches

| Approach | Best For | Complexity |
|----------|----------|------------|
| **JS Hooks + Rust WASM** | Heavy computation, existing Rust code | Medium |
| **Orb (Elixir WASM)** | Elixir-only teams, small modules | Low |
| **Hybrid** | Complex apps needing both | High |

### 1.3 When to Use WASM with LiveView

**Good Use Cases:**
- Image/video processing on client
- Real-time data visualization (charts, graphs)
- Client-side encryption/hashing
- Complex calculations (scientific, financial)
- Game logic, physics simulations
- Rich text editors, drawing tools

**Bad Use Cases:**
- Simple form validation (use Elixir)
- Basic DOM manipulation (use LiveView)
- Data that needs server validation anyway
- Simple UI interactions

---

## 2. LiveView Hook Architecture

### 2.1 Hook Lifecycle Callbacks

```javascript
// assets/js/hooks/index.js
let Hooks = {};

Hooks.MyHook = {
  // Called when element is added to DOM and LiveView is mounted
  mounted() {
    console.log("Element mounted:", this.el);
    this.setupEventListeners();
  },

  // Called before element is updated (synchronous only)
  beforeUpdate() {
    this.saveState();
  },

  // Called after element is updated by server
  updated() {
    this.restoreState();
  },

  // Called when element is removed from page
  destroyed() {
    this.cleanup();
  },

  // Called when parent LiveView disconnects
  disconnected() {
    this.showOfflineIndicator();
  },

  // Called when parent LiveView reconnects
  reconnected() {
    this.hideOfflineIndicator();
    this.resyncState();
  }
};

export default Hooks;
```

### 2.2 Hook Properties and Methods

```javascript
Hooks.Example = {
  mounted() {
    // Properties
    this.el;          // The bound DOM element
    this.liveSocket;  // The LiveSocket instance
    this.viewName;    // The LiveView module name

    // Methods
    this.pushEvent("event_name", { key: "value" }, (reply, ref) => {
      // Optional callback when server replies
    });

    this.pushEventTo(selector, "event_name", payload);  // Target specific element

    this.handleEvent("server_event", (payload) => {
      // Handle events from server
    });

    this.upload(name, files);  // Trigger file upload

    // Get data attributes
    const value = this.el.dataset.myValue;
  }
};
```

### 2.3 Registering Hooks

```javascript
// assets/js/app.js
import { Socket } from "phoenix";
import { LiveSocket } from "phoenix_live_view";
import Hooks from "./hooks";

let liveSocket = new LiveSocket("/live", Socket, {
  hooks: Hooks,
  params: { _csrf_token: csrfToken },
  dom: {
    // Preserve client-side attributes during patching
    onBeforeElUpdated(from, to) {
      // Copy data-js-* attributes
      for (const attr of from.attributes) {
        if (attr.name.startsWith("data-js-")) {
          to.setAttribute(attr.name, attr.value);
        }
      }
    }
  }
});

liveSocket.connect();
```

---

## 3. WASM Hook Integration

### 3.1 Basic WASM Hook

**Rust Side (src/lib.rs):**
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Processor {
    data: Vec<f64>,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Processor { data: Vec::new() }
    }

    pub fn process(&mut self, input: &[f64]) -> Vec<f64> {
        input.iter().map(|x| x * 2.0).collect()
    }

    pub fn get_stats(&self) -> JsValue {
        let stats = serde_json::json!({
            "count": self.data.len(),
            "sum": self.data.iter().sum::<f64>(),
        });
        serde_wasm_bindgen::to_value(&stats).unwrap()
    }
}
```

**JavaScript Hook (assets/js/hooks/wasm_processor.js):**
```javascript
import init, { Processor } from "../../../priv/static/wasm/my_wasm.js";

const WasmProcessor = {
  processor: null,

  async mounted() {
    try {
      // Initialize WASM module
      await init("/wasm/my_wasm_bg.wasm");
      this.processor = new Processor();
      console.log("WASM Processor initialized");

      // Listen for server events
      this.handleEvent("process_data", ({ data }) => {
        const result = this.processor.process(new Float64Array(data));
        this.pushEvent("data_processed", { result: Array.from(result) });
      });

      // Notify server that WASM is ready
      this.pushEvent("wasm_ready", {});

    } catch (error) {
      console.error("Failed to load WASM:", error);
      this.pushEvent("wasm_error", { message: error.toString() });
    }
  },

  updated() {
    // Re-process if data attribute changed
    const newData = JSON.parse(this.el.dataset.input || "[]");
    if (this.processor && newData.length > 0) {
      const result = this.processor.process(new Float64Array(newData));
      this.el.querySelector(".result").textContent = result.join(", ");
    }
  },

  destroyed() {
    // Clean up WASM resources
    if (this.processor) {
      this.processor.free();
      this.processor = null;
    }
  }
};

export default WasmProcessor;
```

**LiveView (lib/my_app_web/live/processor_live.ex):**
```elixir
defmodule MyAppWeb.ProcessorLive do
  use MyAppWeb, :live_view

  def mount(_params, _session, socket) do
    {:ok, assign(socket, wasm_ready: false, result: nil, input: [1, 2, 3, 4, 5])}
  end

  def render(assigns) do
    ~H"""
    <div
      id="wasm-processor"
      phx-hook="WasmProcessor"
      data-input={Jason.encode!(@input)}
    >
      <div class="status">
        WASM Status: <%= if @wasm_ready, do: "Ready", else: "Loading..." %>
      </div>

      <button phx-click="trigger_process">Process Data</button>

      <%= if @result do %>
        <div class="result">Result: <%= inspect(@result) %></div>
      <% end %>
    </div>
    """
  end

  def handle_event("wasm_ready", _params, socket) do
    {:noreply, assign(socket, wasm_ready: true)}
  end

  def handle_event("wasm_error", %{"message" => msg}, socket) do
    {:noreply, put_flash(socket, :error, "WASM Error: #{msg}")}
  end

  def handle_event("trigger_process", _params, socket) do
    # Push data to client for WASM processing
    {:noreply, push_event(socket, "process_data", %{data: socket.assigns.input})}
  end

  def handle_event("data_processed", %{"result" => result}, socket) do
    {:noreply, assign(socket, result: result)}
  end
end
```

### 3.2 Async WASM Loading Pattern

```javascript
const AsyncWasmHook = {
  async mounted() {
    // Show loading state
    this.el.classList.add("loading");

    // Lazy load WASM module
    const { default: init, MyModule } = await import(
      /* webpackChunkName: "wasm" */
      "../../../priv/static/wasm/my_module.js"
    );

    await init();
    this.module = new MyModule();

    // Hide loading state
    this.el.classList.remove("loading");

    this.setupHandlers();
  },

  setupHandlers() {
    this.handleEvent("compute", async ({ input }) => {
      // Run in next tick to not block UI
      requestAnimationFrame(() => {
        const result = this.module.compute(input);
        this.pushEvent("computed", { result });
      });
    });
  }
};
```

---

## 4. Event Communication

### 4.1 Client → Server (pushEvent)

```javascript
// In hook
this.pushEvent("my_event", { key: "value" }, (reply, ref) => {
  // Optional: handle server reply
  console.log("Server replied:", reply);
});

// Target specific LiveComponent
this.pushEventTo("#my-component", "component_event", { data: 123 });
```

**Server handling:**
```elixir
def handle_event("my_event", %{"key" => value}, socket) do
  # Process event
  result = do_something(value)

  # Option 1: Just update assigns (triggers re-render)
  {:noreply, assign(socket, result: result)}

  # Option 2: Reply to client callback
  {:reply, %{status: "ok", data: result}, socket}
end
```

### 4.2 Server → Client (push_event)

```elixir
def handle_info({:data_ready, data}, socket) do
  # Push event to all hooks listening for "data_update"
  {:noreply, push_event(socket, "data_update", %{payload: data})}
end

# Can also push in handle_event
def handle_event("start_process", _params, socket) do
  socket =
    socket
    |> assign(processing: true)
    |> push_event("show_progress", %{step: 1})

  {:noreply, socket}
end
```

**Client handling:**
```javascript
const MyHook = {
  mounted() {
    // Handle events from server
    this.handleEvent("data_update", ({ payload }) => {
      this.updateDisplay(payload);
    });

    this.handleEvent("show_progress", ({ step }) => {
      this.progressBar.style.width = `${step * 25}%`;
    });
  }
};
```

### 4.3 Bidirectional Communication Pattern

```javascript
const BidirectionalHook = {
  mounted() {
    // Server → Client
    this.handleEvent("server_command", async ({ action, data }) => {
      let result;

      switch (action) {
        case "compute":
          result = await this.wasmModule.compute(data);
          break;
        case "validate":
          result = await this.wasmModule.validate(data);
          break;
        default:
          result = { error: "Unknown action" };
      }

      // Client → Server
      this.pushEvent("command_result", { action, result });
    });
  }
};
```

```elixir
# Server-side command dispatcher
def handle_event("run_wasm", %{"action" => action, "data" => data}, socket) do
  {:noreply, push_event(socket, "server_command", %{action: action, data: data})}
end

def handle_event("command_result", %{"action" => action, "result" => result}, socket) do
  # Process result based on action
  {:noreply, assign(socket, "#{action}_result": result)}
end
```

---

## 5. phx-update="ignore" Pattern

### 5.1 Basic Ignore Pattern

For elements controlled entirely by JavaScript/WASM:

```elixir
def render(assigns) do
  ~H"""
  <div id="wasm-canvas-wrapper"
       phx-hook="WasmCanvas"
       data-config={Jason.encode!(@canvas_config)}>
    <%!-- This canvas is controlled by WASM, LiveView won't touch it --%>
    <canvas id="my-canvas" phx-update="ignore" width="800" height="600">
      Canvas not supported
    </canvas>
  </div>
  """
end
```

**Important:** The hook goes on the **wrapper**, not the ignored element!

### 5.2 Data Attribute Updates with Ignore

LiveView still updates data attributes on ignored elements:

```elixir
def render(assigns) do
  ~H"""
  <div id="chart-container"
       phx-hook="ChartHook"
       phx-update="ignore"
       data-chart-data={Jason.encode!(@chart_data)}
       data-chart-type={@chart_type}>
    <%!-- Chart rendered by WASM/JS --%>
  </div>
  """
end
```

```javascript
const ChartHook = {
  mounted() {
    this.initChart();
    this.observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        if (mutation.type === "attributes" &&
            mutation.attributeName.startsWith("data-")) {
          this.updateChart();
        }
      }
    });
    this.observer.observe(this.el, { attributes: true });
  },

  initChart() {
    const data = JSON.parse(this.el.dataset.chartData);
    // Initialize WASM chart renderer
    this.chart = this.wasmModule.createChart(this.el, data);
  },

  updateChart() {
    const data = JSON.parse(this.el.dataset.chartData);
    this.chart.update(data);
  },

  destroyed() {
    this.observer.disconnect();
    this.chart.destroy();
  }
};
```

### 5.3 Wrapper Pattern for Canvas

The recommended pattern when both hook callbacks AND ignore are needed:

```elixir
def render(assigns) do
  ~H"""
  <%!-- Wrapper receives hook callbacks --%>
  <div id="canvas-hook"
       phx-hook="CanvasHook"
       data-particles={Jason.encode!(@particles)}>
    <%!-- Canvas is ignored, WASM controls it --%>
    <canvas id="particle-canvas" phx-update="ignore"></canvas>
  </div>
  """
end
```

```javascript
const CanvasHook = {
  mounted() {
    // Find canvas inside wrapper
    this.canvas = this.el.querySelector("canvas");
    this.ctx = this.canvas.getContext("2d");

    // Initialize WASM renderer
    this.initWasm();
  },

  updated() {
    // This IS called because wrapper doesn't have ignore
    const particles = JSON.parse(this.el.dataset.particles);
    this.wasmRenderer.updateParticles(particles);
  },

  async initWasm() {
    const { default: init, ParticleRenderer } = await import("./wasm/particles.js");
    await init();
    this.wasmRenderer = new ParticleRenderer(this.canvas);
    this.startRenderLoop();
  },

  startRenderLoop() {
    const render = () => {
      this.wasmRenderer.render();
      this.animationId = requestAnimationFrame(render);
    };
    render();
  },

  destroyed() {
    cancelAnimationFrame(this.animationId);
    this.wasmRenderer.free();
  }
};
```

---

## 6. Orb: Elixir-Authored WebAssembly

### 6.1 Orb Basics

Orb lets you write WebAssembly directly in Elixir:

```elixir
# lib/my_app/wasm/calculator.ex
defmodule MyApp.Wasm.Calculator do
  use Orb

  # Global mutable variable
  global do
    @accumulator 0
  end

  # Exported function (visible to JS)
  defw add(a: I32, b: I32), I32 do
    a + b
  end

  # Exported function that uses global
  defw accumulate(value: I32), I32 do
    @accumulator = @accumulator + value
    @accumulator
  end

  # Private function (internal use only)
  defwp square(n: I32), I32 do
    n * n
  end

  # Using private function
  defw sum_of_squares(a: I32, b: I32), I32 do
    square(a) + square(b)
  end
end
```

### 6.2 Compiling Orb to WASM

```elixir
# In a Mix task or at compile time
defmodule Mix.Tasks.CompileWasm do
  use Mix.Task

  def run(_) do
    # Generate .wat (WebAssembly Text Format)
    wat = Orb.to_wat(MyApp.Wasm.Calculator)
    File.write!("priv/static/wasm/calculator.wat", wat)

    # Use external tool to convert to .wasm
    # wabt's wat2wasm or similar
    System.cmd("wat2wasm", [
      "priv/static/wasm/calculator.wat",
      "-o", "priv/static/wasm/calculator.wasm"
    ])
  end
end
```

### 6.3 Orb with Memory

```elixir
defmodule MyApp.Wasm.StringProcessor do
  use Orb

  # Allocate memory pages (64 KiB each)
  Memory.pages(1)

  # String constant (automatically placed in memory)
  @greeting "Hello, "

  defw greet_length(), I32 do
    # Returns byte length of greeting
    7
  end

  defw get_greeting_ptr(), I32 do
    # Returns memory offset of string
    @greeting
  end
end
```

### 6.4 Orb Module Composition

```elixir
defmodule MyApp.Wasm.MathUtils do
  use Orb

  defw factorial(n: I32), I32 do
    if n <= 1 do
      1
    else
      n * factorial(n - 1)
    end
  end
end

defmodule MyApp.Wasm.Stats do
  use Orb

  # Include functions from another module
  Orb.include(MyApp.Wasm.MathUtils)

  defw permutations(n: I32, r: I32), I32 do
    MathUtils.factorial(n) / MathUtils.factorial(n - r)
  end
end
```

### 6.5 Using Orb WASM in LiveView

```javascript
// assets/js/hooks/orb_calculator.js
const OrbCalculatorHook = {
  async mounted() {
    // Load the Orb-generated WASM
    const response = await fetch("/wasm/calculator.wasm");
    const bytes = await response.arrayBuffer();
    const { instance } = await WebAssembly.instantiate(bytes);

    this.wasm = instance.exports;

    // Test it
    console.log("2 + 3 =", this.wasm.add(2, 3));

    this.handleEvent("calculate", ({ a, b, op }) => {
      let result;
      switch (op) {
        case "add": result = this.wasm.add(a, b); break;
        case "sum_squares": result = this.wasm.sum_of_squares(a, b); break;
      }
      this.pushEvent("result", { value: result });
    });
  }
};

export default OrbCalculatorHook;
```

---

## 7. State Synchronization

### 7.1 Server as Source of Truth

```javascript
const StateSyncHook = {
  mounted() {
    // Initial state from server via data attribute
    this.state = JSON.parse(this.el.dataset.state);
    this.wasmModule.setState(this.state);

    // Listen for state updates from server
    this.handleEvent("state_update", (newState) => {
      this.state = { ...this.state, ...newState };
      this.wasmModule.setState(this.state);
      this.render();
    });
  },

  updated() {
    // Server updated data attributes
    const newState = JSON.parse(this.el.dataset.state);
    if (JSON.stringify(newState) !== JSON.stringify(this.state)) {
      this.state = newState;
      this.wasmModule.setState(this.state);
      this.render();
    }
  }
};
```

### 7.2 Client State with Server Validation

```javascript
const ValidatedStateHook = {
  mounted() {
    this.localState = {};
    this.pendingUpdates = new Map();
    this.updateId = 0;

    this.handleEvent("state_validated", ({ id, state, errors }) => {
      if (errors.length > 0) {
        // Rollback local state
        this.rollback(id);
        this.showErrors(errors);
      } else {
        // Confirm local state
        this.confirm(id, state);
      }
    });
  },

  updateState(changes) {
    const id = ++this.updateId;

    // Optimistically apply locally
    const oldState = { ...this.localState };
    this.localState = { ...this.localState, ...changes };
    this.pendingUpdates.set(id, oldState);

    // Update WASM immediately
    this.wasmModule.setState(this.localState);
    this.render();

    // Send to server for validation
    this.pushEvent("validate_state", { id, changes });
  },

  rollback(id) {
    const oldState = this.pendingUpdates.get(id);
    if (oldState) {
      this.localState = oldState;
      this.wasmModule.setState(this.localState);
      this.render();
    }
    this.pendingUpdates.delete(id);
  },

  confirm(id, serverState) {
    this.localState = serverState;
    this.pendingUpdates.delete(id);
  }
};
```

### 7.3 Periodic State Sync

```javascript
const PeriodicSyncHook = {
  mounted() {
    this.wasmModule = new WasmModule();
    this.dirty = false;

    // Mark dirty on local changes
    this.wasmModule.onChange(() => {
      this.dirty = true;
    });

    // Sync every 5 seconds if dirty
    this.syncInterval = setInterval(() => {
      if (this.dirty) {
        const state = this.wasmModule.getState();
        this.pushEvent("sync_state", { state });
        this.dirty = false;
      }
    }, 5000);
  },

  destroyed() {
    clearInterval(this.syncInterval);
    // Final sync on destroy
    if (this.dirty) {
      this.pushEvent("sync_state", { state: this.wasmModule.getState() });
    }
  }
};
```

---

## 8. Connection Handling

### 8.1 Handling Disconnection

```javascript
const ResilientWasmHook = {
  mounted() {
    this.isConnected = true;
    this.pendingEvents = [];

    this.initWasm();
  },

  disconnected() {
    this.isConnected = false;
    this.el.classList.add("offline");
    this.showOfflineIndicator();

    // Save state to localStorage
    const state = this.wasmModule.getState();
    localStorage.setItem("wasm_state", JSON.stringify(state));
  },

  reconnected() {
    this.isConnected = true;
    this.el.classList.remove("offline");
    this.hideOfflineIndicator();

    // Restore and sync state
    const savedState = localStorage.getItem("wasm_state");
    if (savedState) {
      this.pushEvent("restore_state", { state: JSON.parse(savedState) });
    }

    // Flush pending events
    for (const event of this.pendingEvents) {
      this.pushEvent(event.name, event.payload);
    }
    this.pendingEvents = [];
  },

  safePushEvent(name, payload) {
    if (this.isConnected) {
      this.pushEvent(name, payload);
    } else {
      this.pendingEvents.push({ name, payload });
    }
  },

  showOfflineIndicator() {
    // Show visual indicator
  },

  hideOfflineIndicator() {
    // Hide indicator
  }
};
```

### 8.2 Server-Side Reconnection Handling

```elixir
defmodule MyAppWeb.WasmLive do
  use MyAppWeb, :live_view

  def mount(_params, session, socket) do
    socket =
      socket
      |> assign(state: %{}, connected_at: nil)
      |> maybe_restore_state(session)

    {:ok, socket}
  end

  defp maybe_restore_state(socket, %{"state_key" => key}) do
    case MyApp.Cache.get(key) do
      nil -> socket
      state -> assign(socket, state: state)
    end
  end

  defp maybe_restore_state(socket, _), do: socket

  def handle_event("restore_state", %{"state" => client_state}, socket) do
    # Merge client state with any server state
    merged = merge_states(socket.assigns.state, client_state)
    {:noreply, assign(socket, state: merged)}
  end

  def handle_event("sync_state", %{"state" => state}, socket) do
    # Persist state for recovery
    key = socket.assigns.state_key
    MyApp.Cache.put(key, state, ttl: :timer.hours(24))
    {:noreply, assign(socket, state: state)}
  end
end
```

---

## 9. Asset Management

### 9.1 WASM File Placement

```
priv/
└── static/
    └── wasm/
        ├── my_module.js         # wasm-bindgen generated
        ├── my_module_bg.wasm    # Compiled WASM binary
        └── my_module.d.ts       # TypeScript definitions (optional)
```

### 9.2 Endpoint Configuration

```elixir
# lib/my_app_web/endpoint.ex
plug Plug.Static,
  at: "/",
  from: :my_app,
  gzip: true,
  only: ~w(assets fonts images wasm favicon.ico robots.txt),
  # Set proper MIME type for WASM
  headers: %{
    "content-type" => "application/wasm"
  }
```

### 9.3 Build Integration

**In mix.exs:**
```elixir
defp aliases do
  [
    setup: ["deps.get", "ecto.setup", "cmd npm install --prefix assets"],
    "assets.build": [
      "cmd npm run build --prefix assets",
      "cmd ./build_wasm.sh"  # Build WASM
    ],
    "assets.deploy": [
      "cmd npm run deploy --prefix assets",
      "cmd ./build_wasm.sh --release",
      "phx.digest"
    ]
  ]
end
```

**build_wasm.sh:**
```bash
#!/bin/bash
cd rust_wasm

if [ "$1" == "--release" ]; then
  cargo build --target wasm32-unknown-unknown --release
  wasm-bindgen --target web --out-dir ../priv/static/wasm \
    ./target/wasm32-unknown-unknown/release/my_module.wasm
  wasm-opt -Oz -o ../priv/static/wasm/my_module_bg.wasm \
    ../priv/static/wasm/my_module_bg.wasm
else
  cargo build --target wasm32-unknown-unknown
  wasm-bindgen --target web --out-dir ../priv/static/wasm \
    ./target/wasm32-unknown-unknown/debug/my_module.wasm
fi
```

---

## 10. Patterns

### Pattern 1: Computation Offload

```javascript
const ComputeOffloadHook = {
  async mounted() {
    await this.initWasm();

    this.handleEvent("heavy_computation", async ({ data }) => {
      // Show loading state
      this.pushEvent("computation_started", {});

      // Run in WASM (non-blocking via requestIdleCallback)
      requestIdleCallback(async () => {
        const result = this.wasmModule.process(data);
        this.pushEvent("computation_complete", { result });
      });
    });
  }
};
```

```elixir
def handle_event("compute", %{"data" => data}, socket) do
  # Offload to client WASM
  {:noreply, push_event(socket, "heavy_computation", %{data: data})}
end

def handle_event("computation_complete", %{"result" => result}, socket) do
  {:noreply, assign(socket, result: result, computing: false)}
end
```

### Pattern 2: Real-Time Visualization

```javascript
const VisualizationHook = {
  async mounted() {
    await this.initWasm();
    this.canvas = this.el.querySelector("canvas");
    this.renderer = new this.WasmRenderer(this.canvas);

    // Initial data from server
    const initialData = JSON.parse(this.el.dataset.chartData);
    this.renderer.setData(initialData);

    // Stream updates from server
    this.handleEvent("data_point", ({ point }) => {
      this.renderer.addPoint(point);
    });

    this.startRenderLoop();
  },

  startRenderLoop() {
    const render = () => {
      this.renderer.render();
      this.frameId = requestAnimationFrame(render);
    };
    render();
  },

  destroyed() {
    cancelAnimationFrame(this.frameId);
    this.renderer.free();
  }
};
```

### Pattern 3: Client-Side Validation

```javascript
const ValidationHook = {
  mounted() {
    const form = this.el.querySelector("form");

    form.addEventListener("input", (e) => {
      const field = e.target.name;
      const value = e.target.value;

      // Instant client-side validation via WASM
      const errors = this.wasmValidator.validate(field, value);

      if (errors.length > 0) {
        this.showFieldErrors(field, errors);
      } else {
        this.clearFieldErrors(field);
      }
    });

    form.addEventListener("submit", (e) => {
      e.preventDefault();

      const formData = new FormData(form);
      const data = Object.fromEntries(formData);

      // Full validation
      const allErrors = this.wasmValidator.validateAll(data);

      if (Object.keys(allErrors).length === 0) {
        // Send to server
        this.pushEvent("form_submit", data);
      } else {
        this.showAllErrors(allErrors);
      }
    });
  }
};
```

### Pattern 4: Progressive Enhancement

```elixir
def render(assigns) do
  ~H"""
  <div id="editor"
       phx-hook="RichTextEditor"
       data-content={@content}
       data-wasm-url="/wasm/editor.wasm">

    <%!-- Fallback for no-JS/no-WASM --%>
    <noscript>
      <textarea name="content"><%= @content %></textarea>
    </noscript>

    <%!-- Loading state --%>
    <div class="editor-loading" phx-update="ignore">
      Loading editor...
    </div>

    <%!-- WASM-controlled area --%>
    <div class="editor-container" phx-update="ignore"></div>
  </div>
  """
end
```

```javascript
const RichTextEditorHook = {
  async mounted() {
    try {
      const wasmUrl = this.el.dataset.wasmUrl;
      await this.initWasm(wasmUrl);

      // Hide loading, show editor
      this.el.querySelector(".editor-loading").style.display = "none";
      const container = this.el.querySelector(".editor-container");
      container.style.display = "block";

      this.editor = new this.WasmEditor(container);
      this.editor.setContent(this.el.dataset.content);

    } catch (e) {
      // Fallback to basic textarea
      console.warn("WASM editor failed, using fallback:", e);
      this.el.innerHTML = `<textarea name="content">${this.el.dataset.content}</textarea>`;
    }
  }
};
```

### Pattern 5: Shared WASM Instance

```javascript
// Singleton WASM loader
class WasmManager {
  static instance = null;
  static loading = null;

  static async getInstance() {
    if (this.instance) return this.instance;

    if (!this.loading) {
      this.loading = (async () => {
        const { default: init, MyModule } = await import("./wasm/my_module.js");
        await init();
        this.instance = new MyModule();
        return this.instance;
      })();
    }

    return this.loading;
  }
}

// Usage in multiple hooks
const Hook1 = {
  async mounted() {
    this.wasm = await WasmManager.getInstance();
  }
};

const Hook2 = {
  async mounted() {
    this.wasm = await WasmManager.getInstance(); // Same instance
  }
};
```

---

## 11. Anti-Patterns

### Anti-Pattern 1: WASM for Simple Operations

```javascript
// BAD: Using WASM for trivial computation
const result = wasmModule.add(1, 2);  // JS is faster for this!

// GOOD: Use WASM for complex operations
const result = wasmModule.processLargeDataset(data);
```

### Anti-Pattern 2: Fighting LiveView DOM Control

```javascript
// BAD: Manually manipulating DOM that LiveView manages
mounted() {
  document.getElementById("server-element").innerHTML = "Changed!";
  // LiveView will overwrite this on next patch!
}

// GOOD: Use phx-update="ignore" or data attributes
mounted() {
  // Only touch elements marked with ignore
  this.el.querySelector("[phx-update='ignore']").innerHTML = "Safe!";
}
```

### Anti-Pattern 3: Excessive Event Frequency

```javascript
// BAD: Pushing every keystroke
input.addEventListener("input", (e) => {
  this.pushEvent("validate", { value: e.target.value });
});

// GOOD: Debounce high-frequency events
input.addEventListener("input", debounce((e) => {
  this.pushEvent("validate", { value: e.target.value });
}, 300));
```

### Anti-Pattern 4: Not Cleaning Up Resources

```javascript
// BAD: Memory leak
mounted() {
  this.interval = setInterval(() => this.update(), 100);
  this.wasm = new WasmModule();
}
// No destroyed() callback!

// GOOD: Proper cleanup
destroyed() {
  clearInterval(this.interval);
  this.wasm.free();
}
```

### Anti-Pattern 5: Blocking During WASM Load

```javascript
// BAD: Blocking UI during load
async mounted() {
  await init();  // User sees frozen UI
  this.setup();
}

// GOOD: Show loading state
async mounted() {
  this.el.classList.add("loading");
  try {
    await init();
    this.setup();
  } finally {
    this.el.classList.remove("loading");
  }
}
```

---

## 12. Common Failures & Solutions

### Failure 1: Hook updated() Not Called with ignore

```
Hook updated callback never fires
```

**Cause:** Using `phx-update="ignore"` on the hooked element.

**Solution:** Use wrapper pattern:
```html
<div phx-hook="MyHook" data-value={@value}>
  <div phx-update="ignore">
    <!-- WASM-controlled content -->
  </div>
</div>
```

### Failure 2: WASM Module Not Found

```
Failed to load WASM: TypeError: Failed to fetch
```

**Solution:**
1. Check file path in static directory
2. Verify endpoint serves `/wasm` directory
3. Check CORS headers if loading from CDN

### Failure 3: handleEvent Not Receiving Events

```
Events pushed from server not received
```

**Solution:**
```javascript
// Events are global - check you're in mounted()
mounted() {
  this.handleEvent("my_event", (payload) => {
    // Handler must be registered here
  });
}
```

### Failure 4: Memory Leak on Navigation

```
WASM memory grows continuously during SPA navigation
```

**Solution:**
```javascript
destroyed() {
  // Always free WASM resources
  if (this.wasmInstance) {
    this.wasmInstance.free();
    this.wasmInstance = null;
  }
}
```

### Failure 5: Stale Closure in Event Handler

```
Handler uses old state values
```

**Solution:**
```javascript
// BAD: Closure captures initial state
mounted() {
  const initialValue = this.el.dataset.value;
  this.handleEvent("update", () => {
    console.log(initialValue);  // Always old value!
  });
}

// GOOD: Read current value
mounted() {
  this.handleEvent("update", () => {
    console.log(this.el.dataset.value);  // Current value
  });
}
```

---

## 13. Quick Reference

### LiveView Hook Template

```javascript
const WasmHook = {
  wasm: null,

  async mounted() {
    await this.initWasm();
    this.setupEventHandlers();
  },

  async initWasm() {
    const { default: init, Module } = await import("./wasm/module.js");
    await init();
    this.wasm = new Module();
  },

  setupEventHandlers() {
    this.handleEvent("server_event", (payload) => {
      const result = this.wasm.process(payload);
      this.pushEvent("client_event", { result });
    });
  },

  updated() {
    const data = JSON.parse(this.el.dataset.data);
    this.wasm.update(data);
  },

  disconnected() {
    this.saveState();
  },

  reconnected() {
    this.restoreState();
  },

  destroyed() {
    this.wasm?.free();
  }
};
```

### LiveView Template

```elixir
~H"""
<div id="wasm-container"
     phx-hook="WasmHook"
     data-config={Jason.encode!(@config)}>

  <div phx-update="ignore">
    <!-- WASM-controlled content -->
  </div>

</div>
"""
```

### Server Event Handlers

```elixir
# Receive from client
def handle_event("client_event", params, socket)

# Send to client
push_event(socket, "server_event", payload)

# Reply to client callback
{:reply, %{data: result}, socket}
```

---

## Sources

- [Phoenix LiveView JS Interop](https://hexdocs.pm/phoenix_live_view/js-interop.html)
- [Phoenix.LiveView Documentation](https://hexdocs.pm/phoenix_live_view/Phoenix.LiveView.html)
- [Orb: Write WebAssembly with Elixir](https://useorb.dev/)
- [Orb GitHub](https://github.com/RoyalIcing/Orb)
- [Orb HexDocs](https://hexdocs.pm/orb/Orb.html)
- [Animating Canvas with LiveView](https://www.petecorey.com/blog/2019/10/01/animating-a-canvas-with-phoenix-liveview-an-update/)
- [LiveView Reconnection Strategies](https://dev.to/hexshift/staying-alive-phoenix-liveviews-strategies-for-reconnects-recovery-and-real-time-resilience-43l7)
- [Rust WASM + LiveView Rich Text Editor](https://wrinkleinthefabric.com/posts/rust-and-webassembly-chapter-2)

---

*Document completed: Step 4 of Rust WebAssembly Skill Research*
