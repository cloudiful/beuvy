use crate::utility::{
    UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay, UtilityFlexDirection,
    UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis, UtilityPositionType, UtilityRect,
    UtilityStylePatch, UtilityVal,
};
use bevy::prelude::*;
use bevy::ui::OverflowAxis;

pub fn apply_utility_patch(node: &mut Node, patch: &UtilityStylePatch) {
    if let Some(value) = patch.width {
        node.width = utility_val_to_val(value);
    }
    if let Some(value) = patch.height {
        node.height = utility_val_to_val(value);
    }
    if let Some(value) = patch.min_width {
        node.min_width = utility_val_to_val(value);
    }
    if let Some(value) = patch.min_height {
        node.min_height = utility_val_to_val(value);
    }
    if let Some(value) = patch.max_width {
        node.max_width = utility_val_to_val(value);
    }
    if let Some(value) = patch.max_height {
        node.max_height = utility_val_to_val(value);
    }
    if let Some(value) = patch.flex_direction {
        node.flex_direction = match value {
            UtilityFlexDirection::Row => FlexDirection::Row,
            UtilityFlexDirection::Column => FlexDirection::Column,
            UtilityFlexDirection::RowReverse => FlexDirection::RowReverse,
            UtilityFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        };
    }
    if let Some(value) = patch.justify_content {
        node.justify_content = match value {
            UtilityJustifyContent::FlexStart => JustifyContent::FlexStart,
            UtilityJustifyContent::FlexEnd => JustifyContent::FlexEnd,
            UtilityJustifyContent::Center => JustifyContent::Center,
            UtilityJustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
            UtilityJustifyContent::SpaceAround => JustifyContent::SpaceAround,
            UtilityJustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
        };
    }
    if let Some(value) = patch.align_items {
        node.align_items = match value {
            UtilityAlignItems::FlexStart => AlignItems::FlexStart,
            UtilityAlignItems::FlexEnd => AlignItems::FlexEnd,
            UtilityAlignItems::Center => AlignItems::Center,
            UtilityAlignItems::Baseline => AlignItems::Baseline,
            UtilityAlignItems::Stretch => AlignItems::Stretch,
        };
    }
    if let Some(value) = patch.align_content {
        node.align_content = match value {
            UtilityAlignContent::FlexStart => AlignContent::FlexStart,
            UtilityAlignContent::FlexEnd => AlignContent::FlexEnd,
            UtilityAlignContent::Center => AlignContent::Center,
            UtilityAlignContent::Stretch => AlignContent::Stretch,
            UtilityAlignContent::SpaceBetween => AlignContent::SpaceBetween,
            UtilityAlignContent::SpaceAround => AlignContent::SpaceAround,
            UtilityAlignContent::SpaceEvenly => AlignContent::SpaceEvenly,
        };
    }
    if let Some(value) = patch.align_self {
        node.align_self = match value {
            UtilityAlignSelf::Auto => AlignSelf::Auto,
            UtilityAlignSelf::FlexStart => AlignSelf::FlexStart,
            UtilityAlignSelf::FlexEnd => AlignSelf::FlexEnd,
            UtilityAlignSelf::Center => AlignSelf::Center,
            UtilityAlignSelf::Baseline => AlignSelf::Baseline,
            UtilityAlignSelf::Stretch => AlignSelf::Stretch,
        };
    }
    if let Some(value) = patch.flex_wrap {
        node.flex_wrap = match value {
            UtilityFlexWrap::NoWrap => FlexWrap::NoWrap,
            UtilityFlexWrap::Wrap => FlexWrap::Wrap,
            UtilityFlexWrap::WrapReverse => FlexWrap::WrapReverse,
        };
    }
    if let Some(value) = patch.flex_grow {
        node.flex_grow = value;
    }
    if let Some(value) = patch.flex_shrink {
        node.flex_shrink = value;
    }
    if let Some(value) = patch.flex_basis {
        node.flex_basis = utility_val_to_val(value);
    }
    if let Some(value) = patch.row_gap {
        node.row_gap = utility_val_to_val(value);
    }
    if let Some(value) = patch.column_gap {
        node.column_gap = utility_val_to_val(value);
    }
    if let Some(value) = &patch.padding {
        node.padding = utility_padding_rect_to_rect(value);
    }
    if let Some(value) = &patch.margin {
        node.margin = utility_margin_rect_to_rect(value);
    }
    if let Some(value) = &patch.border {
        node.border = utility_border_rect_to_rect(value);
    }
    if let Some(value) = patch.border_radius {
        node.border_radius = BorderRadius::all(utility_val_to_val(value));
    }
    if patch.overflow_x.is_some() || patch.overflow_y.is_some() {
        node.overflow = Overflow {
            x: patch
                .overflow_x
                .map(utility_overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
            y: patch
                .overflow_y
                .map(utility_overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
        };
    }
    if let Some(value) = patch.display {
        node.display = match value {
            UtilityDisplay::Flex => Display::Flex,
            UtilityDisplay::Grid => Display::Grid,
            UtilityDisplay::None => Display::None,
            UtilityDisplay::Block => Display::Block,
        };
    }
    if let Some(value) = patch.position_type {
        node.position_type = match value {
            UtilityPositionType::Relative => PositionType::Relative,
            UtilityPositionType::Absolute => PositionType::Absolute,
        };
    }
    if let Some(value) = patch.left {
        node.left = utility_val_to_val(value);
    }
    if let Some(value) = patch.right {
        node.right = utility_val_to_val(value);
    }
    if let Some(value) = patch.top {
        node.top = utility_val_to_val(value);
    }
    if let Some(value) = patch.bottom {
        node.bottom = utility_val_to_val(value);
    }
}

fn utility_val_to_val(value: UtilityVal) -> Val {
    match value {
        UtilityVal::Auto => Val::Auto,
        UtilityVal::Px(value) => Val::Px(value),
        UtilityVal::Percent(value) => Val::Percent(value),
        UtilityVal::Vw(value) => Val::Vw(value),
        UtilityVal::Vh(value) => Val::Vh(value),
    }
}

fn utility_margin_rect_to_rect(value: &UtilityRect) -> UiRect {
    UiRect {
        left: value.left.map(utility_val_to_val).unwrap_or(Val::ZERO),
        right: value.right.map(utility_val_to_val).unwrap_or(Val::ZERO),
        top: value.top.map(utility_val_to_val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(utility_val_to_val).unwrap_or(Val::ZERO),
    }
}

fn utility_padding_rect_to_rect(value: &UtilityRect) -> UiRect {
    utility_zero_default_rect(value)
}

fn utility_border_rect_to_rect(value: &UtilityRect) -> UiRect {
    utility_zero_default_rect(value)
}

fn utility_zero_default_rect(value: &UtilityRect) -> UiRect {
    UiRect {
        left: value.left.map(utility_val_to_val).unwrap_or(Val::ZERO),
        right: value.right.map(utility_val_to_val).unwrap_or(Val::ZERO),
        top: value.top.map(utility_val_to_val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(utility_val_to_val).unwrap_or(Val::ZERO),
    }
}

fn utility_overflow_axis(value: UtilityOverflowAxis) -> OverflowAxis {
    match value {
        UtilityOverflowAxis::Visible => OverflowAxis::Visible,
        UtilityOverflowAxis::Clip => OverflowAxis::Clip,
        UtilityOverflowAxis::Hidden => OverflowAxis::Hidden,
        UtilityOverflowAxis::Scroll => OverflowAxis::Scroll,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_rect_defaults_missing_edges_to_zero() {
        let rect = utility_margin_rect_to_rect(&UtilityRect {
            left: Some(UtilityVal::Auto),
            right: Some(UtilityVal::Auto),
            ..default()
        });

        assert_eq!(rect.left, Val::Auto);
        assert_eq!(rect.right, Val::Auto);
        assert_eq!(rect.top, Val::ZERO);
        assert_eq!(rect.bottom, Val::ZERO);
    }

    #[test]
    fn padding_rect_defaults_missing_edges_to_zero() {
        let rect = utility_padding_rect_to_rect(&UtilityRect {
            left: Some(UtilityVal::Px(10.0)),
            right: Some(UtilityVal::Px(10.0)),
            ..default()
        });

        assert_eq!(rect.left, Val::Px(10.0));
        assert_eq!(rect.right, Val::Px(10.0));
        assert_eq!(rect.top, Val::ZERO);
        assert_eq!(rect.bottom, Val::ZERO);
    }

    #[test]
    fn border_rect_defaults_missing_edges_to_zero() {
        let rect = utility_border_rect_to_rect(&UtilityRect {
            top: Some(UtilityVal::Px(1.0)),
            ..default()
        });

        assert_eq!(rect.left, Val::ZERO);
        assert_eq!(rect.right, Val::ZERO);
        assert_eq!(rect.top, Val::Px(1.0));
        assert_eq!(rect.bottom, Val::ZERO);
    }
}
