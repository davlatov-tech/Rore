use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl Default for Shadow {
    fn default() -> Self {
        Self { offset_x: 0.0, offset_y: 0.0, blur: 0.0, spread: 0.0, color: Color::TRANSPARENT }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Wait,
    Crosshair,
    NotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Filter {
    None,
    Blur(f32),       // px
    Opacity(f32),    // 0.0 - 1.0
    Grayscale(f32),  // 0.0 - 1.0
}

impl Default for Filter {
    fn default() -> Self { Self::None }
}