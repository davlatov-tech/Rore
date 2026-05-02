
# RORE!


**Rore - The High-Performance, Zero-VDOM UI Engine for Rust**

 "Rendering 100+ Data-Grids and Web3 Terminals at ~130MB RAM and ~3% CPU. No Electron bloat. Pure WGPU & Rust."

**Rore** is a high-performance, GPU-accelerated UI framework built entirely in Rust. Specially optimized for high-end and heavy desktop application development. Cross-platform performance based on the pixel-first concept.

[📄 **Read the Architectural docs**](docs/WHITEPAPER.md)

## Why Rore?

| Feature | The Legacy Web Stack | Rore Engine |
| :--- | :--- | :--- |
| **Rendering** | DOM / Virtual DOM | **WGPU (Direct GPU)** |
| **Language** | JavaScript / TS | **Rust** |
| **Performance** | Heavy (WebView) | **Native (120 FPS)** |
| **Memory** | from 350+ RAM  | **`110-150 MB RAM** |                          




 #  Current Status: Rore UI Framework

**Rore** is an ultra-fast, reactive, and hardware-accelerated UI framework written in Rust, powered by the WGPU engine and Taffy (Flexbox) layout system. This document outlines the current technical status, core achievements, and future roadmap of the project.

---

## 🟢 What is Working (Core Features)

Rore's core has reached a level where it effortlessly handles the most complex operations (e.g., rendering data-grid-style charts at 60 FPS).

### 1. Render Pipeline (GPU)
*   **Sparse Instancing:** Instead of drawing objects one by one, hundreds to thousands of UI elements (`InstanceRaw`) are batched and sent to the GPU simultaneously in a single WGPU `RenderPass`.
*   **SDF (Signed Distance Fields):** All corner radii (`border-radius`), shadows (`box-shadow`), and borders are calculated with mathematical precision and drawn pixel-perfect (anti-aliased) entirely within the fragment shader (WGSL).
*   **Partial Redraw:** Only the changed portions of the screen (Dirty Rects) are recalculated. Unchanged areas are not redrawn, keeping both CPU and GPU in an absolute Idle state.
*   **Custom Shaders:** UI widgets are reactively bound directly to GPU shaders via `CustomPaint` and `ShaderBox`.

### 2. Reactivity and Layout (CPU)
*   **Fine-grained Reactivity:** The Signal, Effect, and Memo systems are fully operational. Most importantly, communication with the Taffy layout engine is heavily optimized: when a Signal changes, only the GPU command is updated (`DIRTY_COLOR`), meaning Taffy does not perform unnecessary recalculations.
*   **O(N) Smart Diffing:** Through the `ForList` widget, when thousands of list items change, only the differences (diffs) are identified, and obsolete items are sent to the Garbage Collector (Drop Queue).
*   **O(1) Z-Index & Draw Order:** The drawing order of elements is tracked and updated in `O(1)` time using `HashSet` and `HashMap`.
*   **Mathematical Culling:** Elements that do not fit on the screen (scrolled out of view or clipped) are never sent to the GPU (Clip Rect validation).

### 3. Complex Widgets (UI Toolkit)
*   **VirtualList & ScrollView:** A standalone stateful virtualization mechanism capable of rendering millions of rows at 60 FPS is fully operational.
*   **Router:** Global routing without prop-drilling is working. Old pages are completely removed from memory upon navigation (Zero Memory Leaks).
*   **TextInput:** partially integration of cursor positioning (via SDF font measurer), multiline text splitting, and keyboard events.

---

## 🟡 What is Not Working or Being Optimized (WIP)

While the architecture is highly stable, some system-level adaptations are currently underway:

*   **Different OS Memory Consumption (RAM Overhead):** Currently, on Windows systems, DirectX 12 is chosen by default due to `wgpu::Backends::all()`, causing the NT Heap to hold onto memory aggressively. 
    *   *Solution (Planned):* Enable the `mimalloc` global allocator specifically for Windows and implement a Graceful Degradation cascade search (Vulkan -> DX12 -> GL) for GPU selection.
*   
*   **Accessibility (a11y):** A Semantic Tree for screen readers has not yet been formed within the WGPU render pipeline.
*   **The framework's GPU-level control system has not yet been developed. This is necessary for us to handle the most complex animations and events.

---

##  Technical Metrics

| Metric | Status | Notes |
| :--- | :--- | :--- |
| **Baseline Memory (Linux)** | ~130 MB | Highly efficient thanks to pure Rust and Vulkan. |
| **UI Refresh Rate** | 60 - 120 FPS | Renders complex graphics without lag due to GPU Instancing. |
| **CPU (Idle)** | ~0.00% | Conserves energy when there are no animations thanks to `ControlFlow::Wait`. |
| **Reactivity (Signal Update)** | Microseconds | No DOM, no VDOM. Binds directly to the components. |

---
**Note; Rore is not yet ready to build a fully functional application. Aplha stage
