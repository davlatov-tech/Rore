
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


## Example Syntax (DSL) UI

``` #[component]
fn ProductCard() -> View {
    ui! {
        // --- OUTER CONTAINER (Div equivalent) ---
        Column {
            // 1. Appearance (CSS logic inside Style block)
            style: {
                background: Color::rgb(240, 240, 240), // #f0f0f0
                padding: 20,
                radius: 12,
                gap: 15, // Spacing between elements
            },

            // 2. Inner Elements (HTML structure)
            children: [
                Text {
                    content: "MacBook Pro",
                    style: {
                        size: 24,
                        weight: Bold,
                        color: Black,
                    }
                },

                Button {
                    content: "Buy Now",
                    
                    // Button specific styling
                    style: {
                        background: Blue,
                        color: White,
                        padding: [10, 20], // [Vertical, Horizontal]
                    },

                    // 3. Logic (JS equivalent - Actions)
                    on_click: move |_| {
                        println!("Item Purchased!");
                        // Triggering a service action:
                        // Services::cart().add(current_item_id);
                    }
                }
            ]
        }
    }
}





### Rore script

fn main() {
    // Define server address once on app startup.
    // This initializes the persistent connection pool.
    Rore::connect("https://api.myshop.com");

    // Start rendering the application
    Rore::run(App);
}

async fn handle_login(email: &str, pass: &str) {
    // Note: No URL, No JSON parsing.
    // "Backend" acts as a type-safe proxy to your server code.
    
    let result = Backend::Auth::login(email, pass).await;

    match result {
        Ok(user) => {
            println!("Welcome back, {}", user.name);
            // Navigate to the next screen
            Navigator::push(HomeScreen);
        },
        Err(error) => {
            // Handle specific errors from server (e.g., "Invalid Password")
            println!("Login Error: {}", error.message);
        }
    }
}
