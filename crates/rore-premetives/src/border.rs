use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderSide {
    pub width: f32,
    pub style: BorderStyle,
    pub color: Color,
}

impl Default for BorderSide {
    fn default() -> Self {
        Self { width: 0.0, style: BorderStyle::None, color: Color::TRANSPARENT }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Border {
    pub left: BorderSide,
    pub right: BorderSide,
    pub top: BorderSide,
    pub bottom: BorderSide,
}

impl Border {
    pub fn all(width: f32, style: BorderStyle, color: Color) -> Self {
        let side = BorderSide { width, style, color };
        Self { left: side, right: side, top: side, bottom: side }
    }
}