use crate::geometry::Size; 

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Val {
    Px(f32),
    Percent(f32),
    Auto,
    Vw(f32),
    Vh(f32),
}

impl Default for Val {
    fn default() -> Self { Self::Auto }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Thickness {
    pub top: Val,
    pub right: Val,
    pub bottom: Val,
    pub left: Val,
}

impl Thickness {
    pub fn all(v: Val) -> Self {
        Self { top: v, right: v, bottom: v, left: v }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadius {
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Display {
    #[default]
    Flex,
    None,
    Grid,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Align {
    #[default]
    Stretch,
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    #[default]
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ZIndex(pub i32);

impl ZIndex {
    pub const AUTO: ZIndex = ZIndex(0);
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridLength {
    Auto,
    Fr(f32),
    Px(f32),
    Percent(f32),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridTemplate {
    pub columns: Vec<GridLength>,
    pub rows: Vec<GridLength>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GridPlacement {
    pub start: i16, // 0 = auto
    pub end: i16,   // 0 = auto
    pub span: u16,  // 0 = ishlatilmaydi
}


#[derive(Debug, Clone, PartialEq, Default)]
pub struct Style {
    pub display: Display,
    pub position: Position,
    pub box_sizing: BoxSizing,
    pub width: Val,
    pub height: Val,
    pub min_width: Val,
    pub min_height: Val,
    pub max_width: Val,
    pub max_height: Val,
    pub margin: Thickness,
    pub padding: Thickness,
    pub flex_direction: FlexDirection,
    pub flex_wrap: bool,
    pub justify_content: Align,
    pub align_items: Align,
    pub align_content: Align,
    pub gap: Size,
    pub grid_template_columns: Vec<GridLength>,
    pub grid_template_rows: Vec<GridLength>,
    pub grid_column: GridPlacement, 
    pub grid_row: GridPlacement,    
    pub inset: Thickness,           
    pub aspect_ratio: Option<f32>,  
    pub z_index: ZIndex,
    pub overflow: Overflow,
}