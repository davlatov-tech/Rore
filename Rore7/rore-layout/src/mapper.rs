use rore_types::{Style, Val, Display, FlexDirection, Align, Position};
use taffy::style::{
    Style as TaffyStyle, 
    Dimension, 
    LengthPercentage, 
    LengthPercentageAuto,
    Display as TaffyDisplay,
    FlexDirection as TaffyFlexDirection,
    AlignItems, 
    JustifyContent,
    Position as TaffyPosition,
    FlexWrap,
    // BoxSizing olib tashlandi (Taffy da yo'q yoki kerak emas)
};
use taffy::geometry::{Rect, Size};

fn map_dimension(val: Val) -> Dimension {
    match val {
        Val::Px(v) => Dimension::Points(v), 
        Val::Percent(v) => Dimension::Percent(v / 100.0),
        Val::Auto => Dimension::Auto,
        _ => Dimension::Auto,
    }
}

fn map_length_auto(val: Val) -> LengthPercentageAuto {
    match val {
        Val::Px(v) => LengthPercentageAuto::Points(v),
        Val::Percent(v) => LengthPercentageAuto::Percent(v / 100.0),
        Val::Auto => LengthPercentageAuto::Auto,
        _ => LengthPercentageAuto::Auto,
    }
}

fn map_length(val: Val) -> LengthPercentage {
    match val {
        Val::Px(v) => LengthPercentage::Points(v),
        Val::Percent(v) => LengthPercentage::Percent(v / 100.0),
        _ => LengthPercentage::Points(0.0),
    }
}

pub fn map_style(style: &Style) -> TaffyStyle {
    TaffyStyle {
        display: match style.display {
            Display::Flex => TaffyDisplay::Flex,
            Display::None => TaffyDisplay::None,
            Display::Grid => TaffyDisplay::Grid,
        },
        
        // box_sizing olib tashlandi (Xato berayotgan edi)

        position: match style.position {
            Position::Relative => TaffyPosition::Relative,
            Position::Absolute => TaffyPosition::Absolute,
        },

        size: Size {
            width: map_dimension(style.width),
            height: map_dimension(style.height),
        },

        min_size: Size {
            width: map_dimension(style.min_width),
            height: map_dimension(style.min_height),
        },

        max_size: Size {
            width: map_dimension(style.max_width),
            height: map_dimension(style.max_height),
        },

        margin: Rect {
            left: map_length_auto(style.margin.left),
            right: map_length_auto(style.margin.right),
            top: map_length_auto(style.margin.top),
            bottom: map_length_auto(style.margin.bottom),
        },

        padding: Rect {
            left: map_length(style.padding.left),
            right: map_length(style.padding.right),
            top: map_length(style.padding.top),
            bottom: map_length(style.padding.bottom),
        },
        
        flex_direction: match style.flex_direction {
            FlexDirection::Row => TaffyFlexDirection::Row,
            FlexDirection::Column => TaffyFlexDirection::Column,
            FlexDirection::RowReverse => TaffyFlexDirection::RowReverse,
            FlexDirection::ColumnReverse => TaffyFlexDirection::ColumnReverse,
        },

        flex_wrap: if style.flex_wrap { FlexWrap::Wrap } else { FlexWrap::NoWrap },

        // Endi rore_types da bu maydonlar bor, xato bermaydi
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,

        align_items: match style.align_items {
            Align::Start => Some(AlignItems::FlexStart),
            Align::End => Some(AlignItems::FlexEnd),
            Align::Center => Some(AlignItems::Center),
            Align::Stretch => Some(AlignItems::Stretch),
            _ => None,
        },

        justify_content: match style.justify_content {
            Align::Start => Some(JustifyContent::FlexStart),
            Align::End => Some(JustifyContent::FlexEnd),
            Align::Center => Some(JustifyContent::Center),
            Align::SpaceBetween => Some(JustifyContent::SpaceBetween),
            Align::SpaceAround => Some(JustifyContent::SpaceAround),
            Align::SpaceEvenly => Some(JustifyContent::SpaceEvenly), // Endi ishlaydi
            _ => None,
        },
        
        // GAP XATOSI TUZATILDI: f32 ni Val::Px ga o'raymiz
        gap: Size {
            width: map_length(Val::Px(style.gap.width)),
            height: map_length(Val::Px(style.gap.height)),
        },

        ..Default::default()
    }
}