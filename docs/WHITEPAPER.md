# 🏛 Rore UI Framework: Architectural Whitepaper

**Document Version:** 1.0  
**Core Technologies:** Rust, WGPU, Taffy (Flexbox), WGSL  
**Objective:** To outline the underlying architecture, design philosophies, and technical solutions that make Rore a high-performance, hardware-accelerated UI framework.

---

## 1. Executive Summary
Rore is a next-generation, Rust-based UI framework designed for maximum hardware utilization and minimal CPU overhead. Unlike traditional Virtual DOM (VDOM) frameworks, Rore directly bridges fine-grained reactive state to a highly optimized GPU render pipeline. By rendering entire UI layouts as mathematically calculated Signed Distance Fields (SDFs) via WGPU instancing, Rore achieves consistent 60-120 FPS even under extreme loads (e.g., real-time trading charts or particle simulations).

---

## 2. High-Level Architecture Flow
The Rore architecture is divided into three completely decoupled subsystems. This isolation ensures that heavy mathematical calculations in one layer do not block the others.

1.  **Reactive Core (The Brain):** Manages State, Signals, Effects, and the Event Loop.
2.  **Layout Engine (The Skeleton):** Powered by Taffy, computes Flexbox layouts and boundaries.
3.  **Render Pipeline (The Muscle):** Powered by WGPU, takes computed boundaries and issues highly batched GPU draw calls.

**Data Flow:**  
`User Input -> Event Router -> Signal Mutation -> Effect Trigger -> Command Queue -> [Layout Update?] -> GPU Buffer Update -> Render Pass`

---

## 3. The Reactive Core (Fine-Grained Reactivity)
Rore discards the traditional "VDOM Diffing" approach in favor of a **Signal/Effect** paradigm.

*   **Zero DOM Diffing:** UI Components are functions that run exactly once during initialization. They return an execution graph, not a virtual node tree.
*   **Targeted Updates:** When a `Signal` mutates, the attached `Effect` runs. Rore intelligently differentiates between mutations. For instance, if a color changes, it sends a `DIRTY_COLOR` command to the GPU; the CPU-heavy Layout Engine (Taffy) remains completely asleep.
*   **O(N) Smart Diffing for Lists:** The `ForList` widget uses a custom smart-diffing algorithm. Instead of re-rendering lists, it identifies identical prefixes and suffixes, sending only the obsolete middle elements to the Garbage Collector (Drop Queue) and generating new nodes specifically for the inserted data.

---

## 4. The Layout Engine (Taffy Integration)
Rore utilizes the `Taffy` crate for standard CSS Flexbox layouts, but with strict performance safeguards.

*   **Partial Re-computation:** Taffy only computes the layout of a node if it receives a `DIRTY_LAYOUT` flag. 
*   **Mathematical Culling:** Before any rendering happens, Rore computes "Dirty Rects" (changed areas). If a node falls outside the visible `clip_rect` (e.g., inside a `ScrollView`), it is mathematically discarded on the CPU side. It never reaches the GPU buffer.
*   **O(1) Node Resolution:** The framework maintains an `UiArena` where every UI element is assigned a `NodeId`. Looking up relationships (Parent-Child, Z-Index, Draw Order) is executed via O(1) HashMaps and HashSets, eliminating slow tree traversals during the render loop.

---

## 5. The GPU Render Pipeline (The Secret Sauce)
The most revolutionary aspect of Rore is its rendering engine. It does not draw UI elements using traditional geometry (creating hundreds of vertices for a rounded box).

### 5.1. Sparse Instancing
Rore pushes almost all UI elements (Boxes, Buttons, Inputs) through a single WGSL shader using **WGPU Instancing**. 
*   A single flat quad (4 vertices) is stored in the GPU memory.
*   The framework sends an array of `InstanceRaw` structs containing positions, sizes, and a `style_index`.
*   This allows Rore to render 100,000+ UI elements in a **single draw call**, bypassing the infamous CPU-to-GPU communication bottleneck.

### 5.2. SDF-Based Fragment Shaders
Corners (`border-radius`), borders, and soft shadows (`box-shadow`) are completely calculated within the Fragment Shader using **Signed Distance Fields (SDF)**.
*   This guarantees infinite scalability and pixel-perfect anti-aliasing without the memory cost of high-resolution textures or complex meshes.
*   Animations (like hover color transitions) are interpolated directly on the GPU based on a `TimeUniform` buffer, freeing the CPU from calculating intermediate animation frames.

### 5.3. Custom Shaders as First-Class Citizens
Widgets like `CustomPaint` and `ShaderBox` allow developers to inject custom WGSL code directly into the render pass. Due to Rore's reactive uniform binding (`live_uniforms`), data streams (like audio frequencies or stock prices) can mutate GPU buffers instantly without rebuilding the UI tree.

---

## 6. Advanced Optimizations
*   **Virtualization (`VirtualList`):** For massive data rendering, Rore computes visible indices `(local_clip_top / item_height)`. Only the strictly visible fraction of children is built and passed to the renderer.
*   **Memory Management (Scope & Drop Queue):** Memory leaks are prevented using a deterministic `create_scope` system. When the `Router` navigates or a `Show` condition flips, the old Scope ID is pushed to a `Drop Queue`. The framework securely wipes the `Taffy Node`, `WGPU Buffer`, and `Widget Memory` in the next Idle cycle.

---

## 7. Conclusion
Rore is architected not just to be a UI framework, but a high-performance simulation engine disguised as a UI toolkit. By combining fine-grained reactivity with SDF-based GPU instancing and strict CPU-GPU decoupling, Rore eliminates traditional UI bottlenecks, making it the ideal choice for heavy data-visualization, trading platforms, and high-framerate applications.
