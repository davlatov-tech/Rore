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