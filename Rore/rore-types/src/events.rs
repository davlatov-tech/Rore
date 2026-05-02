// ==================== INPUT ====================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Checkbox,
    Radio,
    File,
    Date,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct InputState {
    pub disabled: bool,
    pub readonly: bool,
    pub required: bool,
    pub checked: bool,
}

// ==================== EVENTS ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Click { x: f32, y: f32, btn: MouseButton },
    MouseMove { x: f32, y: f32 },
    Hover(bool),
    KeyDown(String),
    KeyUp(String),
    Resize { width: f32, height: f32 },
    Scroll { delta_x: f32, delta_y: f32 },
    Focus(bool),
}

// ==================== ACCESSIBILITY ====================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Role {
    #[default]
    Generic,
    Button,
    Link,
    Image,
    Heading,
    Textbox,
    Checkbox,
    List,
    ListItem,
}