#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier(f32, f32, f32, f32),
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