use crate::color::{Color, LinearGradient};

#[derive(Debug, Clone, PartialEq)]
pub enum ImageFit {
    Cover,
    Contain,
    Fill,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    Solid(Color),
    Gradient(LinearGradient),
    Image {
        url: String,
        fit: ImageFit,
        repeat: bool,
    },
}

impl Default for Background {
    fn default() -> Self {
        Self::Solid(Color::TRANSPARENT)
    }
}