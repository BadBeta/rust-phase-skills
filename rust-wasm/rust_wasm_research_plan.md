# Rust WebAssembly Skill - Research & Implementation Plan

> **Created**: December 2025
> **Status**: Planning Phase

## Overview

This plan outlines the stepwise research and implementation process for creating a comprehensive Rust WebAssembly skill with subskills. Each step must be explicitly marked as done upon completion.

---

## Phase 1: Deep Research (8 Topics)

### Step 1: rust_wasm_core.md - Core Concepts & Toolchain
**Status**: [ ] Not Started

**Research Scope**:
- Modern toolchain setup (post wasm-pack era, 2025)
- Cargo.toml configuration for WASM targets
- wasm-bindgen-cli workflow and options
- wasm-opt optimization passes
- Trunk build system deep dive
- Binary size reduction techniques
- WASI basics and Component Model introduction
- Project structure best practices

**Must Include**:
- [ ] Complete project setup walkthrough
- [ ] Cargo.toml templates for different use cases
- [ ] Build scripts and automation examples
- [ ] Size optimization checklist with before/after measurements
- [ ] Common toolchain errors and solutions
- [ ] Anti-patterns: bloated builds, wrong target features
- [ ] Common failures: missing wasm32 target, incompatible crate versions

**Output**: Save to `rust_wasm_core.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 2: rust_wasm_frameworks.md - Frontend Frameworks
**Status**: [ ] Not Started

**Research Scope**:
- Leptos: Fine-grained reactivity, signals, SSR, hydration
- Yew: Virtual DOM, components, agents, hooks
- Dioxus: Multi-platform, RSX syntax, desktop/mobile
- Sycamore: Reactive primitives, lightweight
- Framework comparison matrix with benchmarks
- State management patterns per framework
- Routing solutions
- Form handling

**Must Include**:
- [ ] Complete component examples for each framework
- [ ] State management patterns (signals, atoms, context)
- [ ] Parent-child communication patterns
- [ ] Async data fetching patterns
- [ ] Error boundary implementations
- [ ] Anti-patterns: prop drilling, over-rendering, memory leaks
- [ ] Common failures: hydration mismatch, SSR issues, stale closures

**Output**: Save to `rust_wasm_frameworks.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 3: rust_wasm_interop.md - JavaScript Interoperability
**Status**: [ ] Not Started

**Research Scope**:
- wasm-bindgen attributes and macros deep dive
- Type conversions: primitives, strings, arrays, objects
- web-sys API coverage and usage
- js-sys for JavaScript standard library
- Async/Promise handling with wasm-bindgen-futures
- Callbacks and closures across boundary
- Error propagation between JS and Rust
- Raw WASM exports for performance-critical paths

**Must Include**:
- [ ] Complete type mapping reference table
- [ ] Memory management patterns (who owns what)
- [ ] Async patterns: Promises, async/await bridge
- [ ] Callback patterns: Closures, function references
- [ ] DOM manipulation examples via web-sys
- [ ] Anti-patterns: excessive boundary crossings, string-heavy APIs
- [ ] Common failures: memory leaks, dangling references, type mismatches

**Output**: Save to `rust_wasm_interop.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 4: rust_wasm_liveview.md - Phoenix LiveView Integration
**Status**: [ ] Not Started

**Research Scope**:
- LiveView hooks architecture for WASM integration
- Hook lifecycle and WASM module lifecycle alignment
- Event communication: pushEvent, handleEvent patterns
- Orb library for Elixir-authored WebAssembly
- SilverOrb standard library
- phx-update="ignore" for WASM-controlled regions
- State synchronization between server and WASM
- Offline/PWA considerations with WASM

**Must Include**:
- [ ] Complete hook implementation examples
- [ ] Orb module examples with LiveView integration
- [ ] Bidirectional event communication patterns
- [ ] State persistence across reconnections
- [ ] Loading states and progressive enhancement
- [ ] Anti-patterns: fighting LiveView DOM control, over-integration
- [ ] Common failures: hook lifecycle mismanagement, memory not freed

**Output**: Save to `rust_wasm_liveview.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 5: rust_wasm_security.md - Security Practices
**Status**: [ ] Not Started

**Research Scope**:
- WebAssembly security model deep dive
- Linear memory vulnerabilities and mitigations
- Buffer overflow scenarios in WASM
- Use-after-free in unsafe Rust WASM
- Input validation patterns
- Cryptographic considerations
- Supply chain security for Rust crates
- CSP headers for WASM applications
- Recent CVEs and lessons learned (2024-2025)

**Must Include**:
- [ ] Vulnerability taxonomy with examples
- [ ] Secure coding checklist
- [ ] Input validation patterns
- [ ] Memory safety patterns
- [ ] Dependency auditing workflow (cargo-audit)
- [ ] Anti-patterns: unsafe blocks, unchecked indexing, trusting JS input
- [ ] Common failures: memory corruption, information disclosure

**Output**: Save to `rust_wasm_security.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 6: rust_wasm_performance.md - Optimization & Profiling
**Status**: [ ] Not Started

**Research Scope**:
- SIMD optimization (128-bit operations)
- Memory layout optimization
- Binary size vs runtime performance tradeoffs
- Profiling with twiggy and browser DevTools
- Web Workers for parallel processing
- SharedArrayBuffer for zero-copy data sharing
- Benchmark methodology (avoiding DevTools pitfalls)
- wasm-opt optimization levels

**Must Include**:
- [ ] SIMD code examples with benchmarks
- [ ] Memory alignment patterns
- [ ] Profiling workflow step-by-step
- [ ] Web Worker integration examples
- [ ] Performance comparison tables (JS vs WASM vs WASM+SIMD)
- [ ] Anti-patterns: premature optimization, DOM from WASM, micro-benchmarks
- [ ] Common failures: measuring with DevTools open, ignoring startup cost

**Output**: Save to `rust_wasm_performance.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 7: rust_wasm_testing.md - Testing & Debugging
**Status**: [ ] Not Started

**Research Scope**:
- Unit testing pure Rust code (no WASM)
- wasm-bindgen-test for browser testing
- Headless browser configuration (Chrome, Firefox, Safari)
- DWARF debugging setup
- console_error_panic_hook usage
- Source maps and debugging workflow
- CI/CD integration for WASM tests
- Property-based testing with proptest

**Must Include**:
- [ ] Test file structure and organization
- [ ] Browser test configuration examples
- [ ] Debugging setup guide with screenshots/steps
- [ ] CI/CD pipeline examples (GitHub Actions)
- [ ] Mock patterns for web APIs
- [ ] Anti-patterns: testing WASM-specific code without browser, ignoring async
- [ ] Common failures: headless browser setup issues, DWARF stripping

**Output**: Save to `rust_wasm_testing.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 8: rust_wasm_styling.md - CSS & Tailwind Integration
**Status**: [ ] Not Started

**Research Scope**:
- Tailwind CSS integration with Trunk
- tailwind.config.js for Rust source scanning
- CSS-in-Rust solutions (stylist-rs, styled-yew)
- CSS Modules approach
- Dark mode implementation
- Responsive design patterns
- Animation with WASM (requestAnimationFrame)
- Icon libraries integration

**Must Include**:
- [ ] Tailwind + Trunk setup walkthrough
- [ ] Component styling examples per framework
- [ ] Dark mode toggle implementation
- [ ] Responsive component patterns
- [ ] Animation examples
- [ ] Anti-patterns: inline styles everywhere, fighting framework styling
- [ ] Common failures: Tailwind not scanning Rust files, purging issues

**Output**: Save to `rust_wasm_styling.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

## Phase 2: Skill Implementation

### Step 9: Create Master Skill File
**Status**: [ ] Not Started

**Tasks**:
- [ ] Create `rust_wasm_skill.md` as the master skill entry point
- [ ] Write skill description and purpose
- [ ] Define when to use this skill (trigger conditions)
- [ ] List all subskill references
- [ ] Write quick reference section for common tasks
- [ ] Include cross-references to related skills (rust-nif, phoenix-liveview, tailwind)

**Output**: Save to `rust_wasm_skill.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 10: Create Subskill Reference Files
**Status**: [ ] Not Started

**Tasks**:
- [ ] Create `subskills/rust_wasm_core_skill.md` - toolchain skill
- [ ] Create `subskills/rust_wasm_frameworks_skill.md` - frameworks skill
- [ ] Create `subskills/rust_wasm_interop_skill.md` - JS interop skill
- [ ] Create `subskills/rust_wasm_liveview_skill.md` - LiveView integration skill
- [ ] Create `subskills/rust_wasm_security_skill.md` - security skill
- [ ] Create `subskills/rust_wasm_performance_skill.md` - performance skill
- [ ] Create `subskills/rust_wasm_testing_skill.md` - testing skill
- [ ] Create `subskills/rust_wasm_styling_skill.md` - styling skill

Each subskill file should:
- Reference the corresponding research document
- Define specific trigger conditions
- Provide concise guidance format

**Output**: Save to `subskills/` directory
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 11: Create Code Examples Directory
**Status**: [ ] Not Started

**Tasks**:
- [ ] Create `examples/` directory
- [ ] Create `examples/leptos_hello/` - Leptos starter
- [ ] Create `examples/yew_component/` - Yew component example
- [ ] Create `examples/liveview_wasm_hook/` - LiveView integration
- [ ] Create `examples/orb_calculator/` - Orb example
- [ ] Create `examples/simd_image_processing/` - SIMD example
- [ ] Create `examples/testing_setup/` - Test configuration example
- [ ] Add README.md to each example explaining usage

**Output**: Save to `examples/` directory
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 12: Create Quick Reference Cheatsheet
**Status**: [ ] Not Started

**Tasks**:
- [ ] Create `rust_wasm_cheatsheet.md`
- [ ] Common commands reference
- [ ] Type mapping quick reference
- [ ] Build optimization flags
- [ ] Debug commands
- [ ] Testing commands
- [ ] Performance tips summary
- [ ] Security checklist summary

**Output**: Save to `rust_wasm_cheatsheet.md`
**Completion**: Mark this step as DONE in the todo list when complete

---

## Phase 3: Packaging & Distribution

### Step 13: Review and Cross-Reference
**Status**: [ ] Not Started

**Tasks**:
- [ ] Review all documents for consistency
- [ ] Verify all code examples compile/work
- [ ] Add cross-references between documents
- [ ] Check for gaps in coverage
- [ ] Validate against existing skills for overlap handling
- [ ] Proofread for clarity and accuracy

**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 14: Create Package Manifest
**Status**: [ ] Not Started

**Tasks**:
- [ ] Create `manifest.json` with skill metadata
- [ ] List all included files
- [ ] Define version number
- [ ] Add author and license information
- [ ] Include dependencies on other skills

**Output**: Save to `manifest.json`
**Completion**: Mark this step as DONE in the todo list when complete

---

### Step 15: Create Portable ZIP Archive
**Status**: [ ] Not Started

**Tasks**:
- [ ] Verify all files are complete
- [ ] Create directory structure for packaging:
  ```
  rust_wasm_skill/
  ├── manifest.json
  ├── rust_wasm_skill.md (master)
  ├── research/
  │   ├── rust_wasm_core.md
  │   ├── rust_wasm_frameworks.md
  │   ├── rust_wasm_interop.md
  │   ├── rust_wasm_liveview.md
  │   ├── rust_wasm_security.md
  │   ├── rust_wasm_performance.md
  │   ├── rust_wasm_testing.md
  │   └── rust_wasm_styling.md
  ├── subskills/
  │   ├── rust_wasm_core_skill.md
  │   ├── rust_wasm_frameworks_skill.md
  │   ├── rust_wasm_interop_skill.md
  │   ├── rust_wasm_liveview_skill.md
  │   ├── rust_wasm_security_skill.md
  │   ├── rust_wasm_performance_skill.md
  │   ├── rust_wasm_testing_skill.md
  │   └── rust_wasm_styling_skill.md
  ├── examples/
  │   └── [example directories]
  ├── rust_wasm_cheatsheet.md
  └── rust_wasm_initial_rd.md (initial research)
  ```
- [ ] Create ZIP archive: `rust_wasm_skill.zip`
- [ ] Verify ZIP contents are complete and extractable

**Output**: Save to `rust_wasm_skill.zip`
**Completion**: Mark this step as DONE in the todo list when complete

---

## Execution Notes

### For Each Research Step (1-8):

1. **Start**: Update todo list to mark step as `in_progress`
2. **Research**: Conduct thorough web searches on all topics
3. **Examples**: Include working code examples
4. **Patterns**: Document at least 3-5 patterns per topic
5. **Anti-patterns**: Document at least 3-5 anti-patterns per topic
6. **Failures**: Document common failures with solutions
7. **Save**: Write the complete document to the specified file
8. **Complete**: Update todo list to mark step as `completed`

### Research Depth Requirements:

- Each topic file should be **2000-4000 words minimum**
- Include **code examples** that can be copy-pasted
- Cite sources with **markdown links**
- Include **tables** for quick reference where appropriate
- Add **diagrams** (ASCII art) for architecture/flow explanations

### Quality Checklist Per Document:

- [ ] Comprehensive coverage of the topic
- [ ] At least 5 working code examples
- [ ] Pattern documentation with rationale
- [ ] Anti-pattern documentation with consequences
- [ ] Common failures with diagnostic steps and solutions
- [ ] Cross-references to related topics
- [ ] Sources cited

---

## Timeline Tracking

| Step | Topic | Status | File |
|------|-------|--------|------|
| 1 | Core & Toolchain | [ ] Not Started | rust_wasm_core.md |
| 2 | Frameworks | [ ] Not Started | rust_wasm_frameworks.md |
| 3 | JS Interop | [ ] Not Started | rust_wasm_interop.md |
| 4 | LiveView | [ ] Not Started | rust_wasm_liveview.md |
| 5 | Security | [ ] Not Started | rust_wasm_security.md |
| 6 | Performance | [ ] Not Started | rust_wasm_performance.md |
| 7 | Testing | [ ] Not Started | rust_wasm_testing.md |
| 8 | Styling | [ ] Not Started | rust_wasm_styling.md |
| 9 | Master Skill | [ ] Not Started | rust_wasm_skill.md |
| 10 | Subskills | [ ] Not Started | subskills/*.md |
| 11 | Examples | [ ] Not Started | examples/ |
| 12 | Cheatsheet | [ ] Not Started | rust_wasm_cheatsheet.md |
| 13 | Review | [ ] Not Started | - |
| 14 | Manifest | [ ] Not Started | manifest.json |
| 15 | ZIP Package | [ ] Not Started | rust_wasm_skill.zip |

---

*Plan created December 2025*
