use rore_types::{Align, Display, FlexDirection, Position, Style, Val};
use taffy::geometry::{Point, Rect, Size};
use taffy::style::{
    AlignItems, Dimension, Display as TaffyDisplay, FlexDirection as TaffyFlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Overflow as TaffyOverflow,
    Position as TaffyPosition, Style as TaffyStyle,
};

fn map_dimension(val: Val) -> Dimension {
    match val {
        Val::Px(v) => Dimension::length(v), // Taffy 0.10: funksiyaga o'tdi
        Val::Percent(v) => Dimension::percent(v / 100.0),
        Val::Auto => Dimension::auto(),
        _ => Dimension::auto(),
    }
}

fn map_length_auto(val: Val) -> LengthPercentageAuto {
    match val {
        Val::Px(v) => LengthPercentageAuto::length(v),
        Val::Percent(v) => LengthPercentageAuto::percent(v / 100.0),
        Val::Auto => LengthPercentageAuto::auto(),
        _ => LengthPercentageAuto::auto(),
    }
}

fn map_length(val: Val) -> LengthPercentage {
    match val {
        Val::Px(v) => LengthPercentage::length(v),
        Val::Percent(v) => LengthPercentage::percent(v / 100.0),
        _ => LengthPercentage::length(0.0),
    }
}

pub fn map_style(style: &Style) -> TaffyStyle {
    TaffyStyle {
        display: match style.display {
            Display::Flex => TaffyDisplay::Flex,
            Display::None => TaffyDisplay::None,
            Display::Grid => TaffyDisplay::Grid,
        },
        position: match style.position {
            Position::Relative => TaffyPosition::Relative,
            Position::Absolute => TaffyPosition::Absolute,
        },
        overflow: Point {
            x: match style.overflow {
                rore_types::Overflow::Visible => TaffyOverflow::Visible,
                rore_types::Overflow::Hidden => TaffyOverflow::Hidden,
                rore_types::Overflow::Scroll => TaffyOverflow::Scroll,
                rore_types::Overflow::Auto => TaffyOverflow::Scroll, // Taffy 0.10 Auto'ni qabul qilmaydi, Scroll eng yaxshi alternativ
            },
            y: match style.overflow {
                rore_types::Overflow::Visible => TaffyOverflow::Visible,
                rore_types::Overflow::Hidden => TaffyOverflow::Hidden,
                rore_types::Overflow::Scroll => TaffyOverflow::Scroll,
                rore_types::Overflow::Auto => TaffyOverflow::Scroll,
            },
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
        flex_wrap: if style.flex_wrap {
            FlexWrap::Wrap
        } else {
            FlexWrap::NoWrap
        },
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
            Align::SpaceEvenly => Some(JustifyContent::SpaceEvenly),
            _ => None,
        },
        gap: Size {
            width: map_length(Val::Px(style.gap.width)),
            height: map_length(Val::Px(style.gap.height)),
        },
        ..Default::default()
    }
}
