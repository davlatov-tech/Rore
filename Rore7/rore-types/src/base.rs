use std::vec::Vec;

// ==================== GEOMETRY ====================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point { x, y },
            size: Size { width, height },
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.origin.x && p.x <= self.origin.x + self.size.width &&
        p.y >= self.origin.y && p.y <= self.origin.y + self.size.height
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.origin.x < other.origin.x + other.size.width &&
        self.origin.x + self.size.width > other.origin.x &&
        self.origin.y < other.origin.y + other.size.height &&
        self.origin.y + self.size.height > other.origin.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub scale: Point,
    pub translate: Point,
    pub rotate: f32,
    pub skew: Point,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            scale: Point { x: 1.0, y: 1.0 },
            translate: Point { x: 0.0, y: 0.0 },
            rotate: 0.0,
            skew: Point { x: 0.0, y: 0.0 },
        }
    }
}

// ==================== ANIMATION (LERP & EASING) ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseOutCubic,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseInQuad => t * t,
            Easing::EaseOutQuad => t * (2.0 - t),
            Easing::EaseInOutQuad => {
                if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
            },
            Easing::EaseOutCubic => {
                let f = 1.0 - t;
                1.0 - f * f * f
            }
        }
    }
}

// LERP TRAIT (Animatsiya uchun o'ta muhim)
pub trait Lerp {
    fn lerp(&self, end: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, end: &Self, t: f32) -> Self {
        self + (end - self) * t
    }
}

impl Lerp for [f32; 4] {
    fn lerp(&self, end: &Self, t: f32) -> Self {
        [
            self[0].lerp(&end[0], t),
            self[1].lerp(&end[1], t),
            self[2].lerp(&end[2], t),
            self[3].lerp(&end[3], t),
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transition {
    pub property: String,
    pub duration: u64,
    pub easing: Easing,
    pub delay: u64,
}

impl Default for Transition {
    fn default() -> Self {
        Self { property: "none".into(), duration: 0, easing: Easing::Linear, delay: 0 }
    }
}

// ==================== COLOR ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Color   = Color::new(1.0, 0.0, 0.0, 1.0);
    pub const BLUE: Color  = Color::new(0.0, 0.0, 1.0, 1.0);
    pub const TRANSPARENT: Color = Color::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a,
        }
    }

    pub fn hex(hex: &str) -> Self {
        let hex = hex.trim_start_matches('#');
        let (r, g, b, a) = if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            (r, g, b, 255)
        } else if hex.len() == 8 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            let a = u8::from_str_radix(&hex[6..8], 16).unwrap_or(255);
            (r, g, b, a)
        } else {
            (0, 0, 0, 255)
        };

        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn with_alpha(mut self, a: f32) -> Self {
        self.a = a;
        self
    }
}

// Color uchun Default trait
impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

// Color uchun Lerp implementatsiyasi
impl Lerp for Color {
    fn lerp(&self, end: &Self, t: f32) -> Self {
        Color {
            r: self.r.lerp(&end.r, t),
            g: self.g.lerp(&end.g, t),
            b: self.b.lerp(&end.b, t),
            a: self.a.lerp(&end.a, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub angle: f32,
    pub stops: Vec<GradientStop>,
}