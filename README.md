
      Logo:  https://github.com/user-attachments/assets/bae93097-9eea-4981-8da2-665c85ed6ce7
# RORE!


**The Post-Web Interface Engine**

> "A nuclear reactor on the inside, a TV remote on the outside."

**Rore** is a high-performance, GPU-accelerated UI framework built entirely in Rust. It abandons HTML, CSS, and JavaScript in favor of a unified, type-safe, and purely native architecture that targets every platform from a single codebase.

[📄 **Read the Technical Whitepaper**](docs/WHITEPAPER.md)

## Why Rore?

| Feature | The Legacy Web Stack | Rore Engine |
| :--- | :--- | :--- |
| **Rendering** | DOM / Virtual DOM | **WGPU (Direct GPU)** |
| **Language** | JavaScript / TS | **Rust** |
| **Performance** | Heavy (WebView) | **Native (120 FPS)** |
| **Build** | Dependency Hell | **`rore run android`** |                          


## Example Syntax (DSL)

```rust
ui! {
    Column {
        padding: 20
        center_content

        Text(f"Hello Rore") -> size(32), bold, color(Blue)
        
        Button("Click Me") 
            -> on_click(move |_| println!("Clicked!"))
    }
}
