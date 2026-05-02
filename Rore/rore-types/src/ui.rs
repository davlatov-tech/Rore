use crate::base::{Color, Size, LinearGradient};
use std::vec::Vec;

// ==================== LAYOUT VALUES ====================

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

// ==================== LAYOUT ENUMS ====================

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
    SpaceEvenly,
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

// ==================== GRID ====================

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
    pub start: i16,
    pub end: i16,
    pub span: u16,
}

// ==================== BORDER ====================

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

// ==================== BACKGROUND ====================

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

// ==================== DECORATION ====================

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
    Blur(f32),
    Opacity(f32),
    Grayscale(f32),
}

impl Default for Filter {
    fn default() -> Self { Self::None }
}

// ==================== TYPOGRAPHY ====================

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Overline,
    LineThrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub font_family: String,
    pub weight: FontWeight,
    pub align: TextAlign,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub decoration: TextDecoration,
    pub transform: TextTransform,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_family: "Inter".to_string(),
            weight: FontWeight::Regular,
            align: TextAlign::Left,
            line_height: 1.2,
            letter_spacing: 0.0,
            decoration: TextDecoration::None,
            transform: TextTransform::None,
        }
    }
}

// ==================== STYLE STRUCT ====================

#[derive(Debug, Clone, PartialEq)]
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
    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: Display::default(),
            position: Position::default(),
            box_sizing: BoxSizing::default(),
            width: Val::default(),
            height: Val::default(),
            min_width: Val::default(),
            min_height: Val::default(),
            max_width: Val::default(),
            max_height: Val::default(),
            margin: Thickness::default(),
            padding: Thickness::default(),
            flex_direction: FlexDirection::default(),
            flex_wrap: false,
            
            flex_grow: 0.0,
            flex_shrink: 1.0, 

            justify_content: Align::default(),
            align_items: Align::default(),
            align_content: Align::default(),
            
            gap: Size::default(),
            
            grid_template_columns: Vec::new(),
            grid_template_rows: Vec::new(),
            grid_column: GridPlacement::default(),
            grid_row: GridPlacement::default(),
            inset: Thickness::default(),
            aspect_ratio: None,
            z_index: ZIndex::default(),
            overflow: Overflow::default(),
        }
    }
}