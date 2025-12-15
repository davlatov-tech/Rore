use rore_primitives::layout::{
    Style as RoreStyle,
    GridPlacement as RorePlacement,
    GridLength as RoreGridLength,
    Thickness as RoreThickness,
    Val as RoreVal,
    Display as RoreDisplay,
    Position as RorePosition,
    FlexDirection as RoreFlexDirection,
    Align as RoreAlign,
};

use taffy::prelude::*;
use taffy::style::{Dimension, LengthPercentage, LengthPercentageAuto, GridPlacement as TaffyPlacement}; 

pub fn to_taffy_style(s: &RoreStyle) -> Style {
    Style {
        // --- Layout Mode ---
        display: match s.display {
            RoreDisplay::Flex => Display::Flex,
            RoreDisplay::Grid => Display::Grid,
            RoreDisplay::None => Display::None,
        },

        position: match s.position {
            RorePosition::Absolute => Position::Absolute,
            _ => Position::Relative,
        },

        // --- O'lchamlar (Eng muhim qism) ---
        size: taffy::geometry::Size { width: to_dim(s.width), height: to_dim(s.height) },
        min_size: taffy::geometry::Size { width: to_dim(s.min_width), height: to_dim(s.min_height) },
        max_size: taffy::geometry::Size { width: to_dim(s.max_width), height: to_dim(s.max_height) },
        aspect_ratio: s.aspect_ratio,

        // --- Margin / Padding ---
        margin: to_rect_auto(s.margin),
        padding: to_rect(s.padding),
        inset: to_rect_auto(s.inset),

        // --- Flexbox ---
        flex_direction: match s.flex_direction {
            RoreFlexDirection::Row => FlexDirection::Row,
            RoreFlexDirection::Column => FlexDirection::Column,
            RoreFlexDirection::RowReverse => FlexDirection::RowReverse,
            RoreFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        },
        flex_wrap: if s.flex_wrap { FlexWrap::Wrap } else { FlexWrap::NoWrap },
        
        justify_content: to_justify_content(s.justify_content),
        align_items: to_align_items(s.align_items),
        align_content: to_align_content(s.align_content),

        // --- Grid ---
        gap: taffy::geometry::Size {
            width: to_len(s.gap.width),
            height: to_len(s.gap.height),
        },
        ..Default::default()
    }
}

// --- YORDAMCHI FUNKSIYALAR ---

fn to_dim(v: RoreVal) -> Dimension {
    match v {
        RoreVal::Px(v) => Dimension::Length(v),
        RoreVal::Percent(v) => Dimension::Percent(v / 100.0),
        RoreVal::Auto => Dimension::Auto,
        _ => Dimension::Auto, // Boshqa hollar uchun Auto
    }
}

fn to_len(v: f32) -> LengthPercentage {
    LengthPercentage::Length(v)
}

fn to_rect(t: RoreThickness) -> taffy::geometry::Rect<LengthPercentage> {
    taffy::geometry::Rect {
        left: LengthPercentage::Length(match t.left { RoreVal::Px(v) => v, _ => 0.0 }),
        right: LengthPercentage::Length(match t.right { RoreVal::Px(v) => v, _ => 0.0 }),
        top: LengthPercentage::Length(match t.top { RoreVal::Px(v) => v, _ => 0.0 }),
        bottom: LengthPercentage::Length(match t.bottom { RoreVal::Px(v) => v, _ => 0.0 }),
    }
}

fn to_rect_auto(t: RoreThickness) -> taffy::geometry::Rect<LengthPercentageAuto> {
    let conv = |v| match v {
        RoreVal::Px(x) => LengthPercentageAuto::Length(x),
        RoreVal::Percent(x) => LengthPercentageAuto::Percent(x / 100.0),
        RoreVal::Auto => LengthPercentageAuto::Auto,
        _ => LengthPercentageAuto::Auto,
    };
    taffy::geometry::Rect {
        left: conv(t.left), right: conv(t.right), top: conv(t.top), bottom: conv(t.bottom)
    }
}

fn to_justify_content(a: RoreAlign) -> Option<JustifyContent> {
    match a {
        RoreAlign::Start => Some(JustifyContent::FlexStart),
        RoreAlign::End => Some(JustifyContent::FlexEnd),
        RoreAlign::Center => Some(JustifyContent::Center),
        RoreAlign::SpaceBetween => Some(JustifyContent::SpaceBetween),
        RoreAlign::SpaceAround => Some(JustifyContent::SpaceAround),
        RoreAlign::Stretch => Some(JustifyContent::Stretch),
    }
}

fn to_align_content(a: RoreAlign) -> Option<AlignContent> {
    match a {
        RoreAlign::Start => Some(AlignContent::FlexStart),
        RoreAlign::End => Some(AlignContent::FlexEnd),
        RoreAlign::Center => Some(AlignContent::Center),
        RoreAlign::SpaceBetween => Some(AlignContent::SpaceBetween),
        RoreAlign::SpaceAround => Some(AlignContent::SpaceAround),
        RoreAlign::Stretch => Some(AlignContent::Stretch),
    }
}

fn to_align_items(a: RoreAlign) -> Option<AlignItems> {
    match a {
        RoreAlign::Start => Some(AlignItems::FlexStart),
        RoreAlign::End => Some(AlignItems::FlexEnd),
        RoreAlign::Center => Some(AlignItems::Center),
        RoreAlign::Stretch => Some(AlignItems::Stretch),
        _ => None,
    }
}