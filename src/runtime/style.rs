use crate::ast::*;
use beuvy_runtime::interaction_style::UiStateVisualStyles;
use beuvy_runtime::utility::{
    UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal, UtilityVisualStylePatch,
};
use bevy::prelude::*;
use bevy::ui::OverflowAxis;

pub(crate) trait DeclarativeEntityInsert {
    fn insert_component(&mut self, bundle: impl Bundle);
    fn observe_state_visuals(&mut self);
}

impl<'w> DeclarativeEntityInsert for EntityWorldMut<'w> {
    fn insert_component(&mut self, bundle: impl Bundle) {
        self.insert(bundle);
    }

    fn observe_state_visuals(&mut self) {
        self.observe(beuvy_runtime::interaction_style::pointer_hover_over)
            .observe(beuvy_runtime::interaction_style::pointer_hover_out)
            .observe(beuvy_runtime::interaction_style::pointer_press)
            .observe(beuvy_runtime::interaction_style::pointer_release)
            .observe(beuvy_runtime::interaction_style::pointer_cancel)
            .observe(beuvy_runtime::interaction_style::pointer_drag_end);
    }
}

impl DeclarativeEntityInsert for EntityCommands<'_> {
    fn insert_component(&mut self, bundle: impl Bundle) {
        self.insert(bundle);
    }

    fn observe_state_visuals(&mut self) {
        self.observe(beuvy_runtime::interaction_style::pointer_hover_over)
            .observe(beuvy_runtime::interaction_style::pointer_hover_out)
            .observe(beuvy_runtime::interaction_style::pointer_press)
            .observe(beuvy_runtime::interaction_style::pointer_release)
            .observe(beuvy_runtime::interaction_style::pointer_cancel)
            .observe(beuvy_runtime::interaction_style::pointer_drag_end);
    }
}

pub fn runtime_visual_styles(
    visual_style: &DeclarativeVisualStyle,
    state_visual_styles: &DeclarativeStateVisualStyles,
) -> Option<UiStateVisualStyles> {
    let styles = UiStateVisualStyles {
        base: utility_visual_style(visual_style),
        hover: utility_visual_style(&state_visual_styles.hover),
        active: utility_visual_style(&state_visual_styles.active),
        focus: utility_visual_style(&state_visual_styles.focus),
        disabled: UtilityVisualStylePatch::default(),
    };
    (!styles.is_empty()).then_some(styles)
}

#[allow(dead_code)]
pub(crate) fn runtime_text_visual_styles(
    visual_style: &DeclarativeVisualStyle,
    state_visual_styles: &DeclarativeStateVisualStyles,
) -> Option<UiStateVisualStyles> {
    let mut base = DeclarativeVisualStyle::default();
    base.text_color = visual_style.text_color.clone();
    base.opacity = visual_style.opacity;
    base.transition_property = visual_style.transition_property;
    base.transition_duration_ms = visual_style.transition_duration_ms;
    base.transition_timing = visual_style.transition_timing;

    let mut hover = DeclarativeVisualStyle::default();
    hover.text_color = state_visual_styles.hover.text_color.clone();
    hover.opacity = state_visual_styles.hover.opacity;

    let mut active = DeclarativeVisualStyle::default();
    active.text_color = state_visual_styles.active.text_color.clone();
    active.opacity = state_visual_styles.active.opacity;

    let mut focus = DeclarativeVisualStyle::default();
    focus.text_color = state_visual_styles.focus.text_color.clone();
    focus.opacity = state_visual_styles.focus.opacity;

    runtime_visual_styles(
        &base,
        &DeclarativeStateVisualStyles {
            hover,
            active,
            focus,
        },
    )
}

pub(crate) fn insert_runtime_visuals(
    entity: &mut impl DeclarativeEntityInsert,
    visual_style: &DeclarativeVisualStyle,
    state_visual_styles: &DeclarativeStateVisualStyles,
) {
    let Some(styles) = runtime_visual_styles(visual_style, state_visual_styles) else {
        return;
    };
    entity.insert_component(styles);
    entity.observe_state_visuals();
}

fn utility_visual_style(value: &DeclarativeVisualStyle) -> UtilityVisualStylePatch {
    UtilityVisualStylePatch {
        background_color: value.background_color.clone(),
        text_color: value.text_color.clone(),
        border_color: value.border_color.clone(),
        outline_width: value.outline_width.map(declarative_val_to_utility),
        outline_color: value.outline_color.clone(),
        opacity: value.opacity,
        transition_property: value.transition_property.map(utility_transition_property),
        transition_duration_ms: value.transition_duration_ms,
        transition_timing: value.transition_timing.map(utility_transition_timing),
    }
}

fn declarative_val_to_utility(value: DeclarativeVal) -> UtilityVal {
    match value {
        DeclarativeVal::Auto => UtilityVal::Auto,
        DeclarativeVal::Px(value) => UtilityVal::Px(value),
        DeclarativeVal::Percent(value) => UtilityVal::Percent(value),
        DeclarativeVal::Vw(value) => UtilityVal::Vw(value),
        DeclarativeVal::Vh(value) => UtilityVal::Vh(value),
    }
}

fn utility_transition_property(value: DeclarativeTransitionProperty) -> UtilityTransitionProperty {
    match value {
        DeclarativeTransitionProperty::All => UtilityTransitionProperty::All,
        DeclarativeTransitionProperty::Colors => UtilityTransitionProperty::Colors,
    }
}

fn utility_transition_timing(value: DeclarativeTransitionTiming) -> UtilityTransitionTiming {
    match value {
        DeclarativeTransitionTiming::Linear => UtilityTransitionTiming::Linear,
        DeclarativeTransitionTiming::EaseIn => UtilityTransitionTiming::EaseIn,
        DeclarativeTransitionTiming::EaseOut => UtilityTransitionTiming::EaseOut,
        DeclarativeTransitionTiming::EaseInOut => UtilityTransitionTiming::EaseInOut,
    }
}

pub fn apply_node_style(mut node: Node, style: &DeclarativeNodeStyle) -> Node {
    if let Some(value) = style.width {
        node.width = val(value);
    }
    if let Some(value) = style.height {
        node.height = val(value);
    }
    if let Some(value) = style.min_width {
        node.min_width = val(value);
    }
    if let Some(value) = style.min_height {
        node.min_height = val(value);
    }
    if let Some(value) = style.max_width {
        node.max_width = val(value);
    }
    if let Some(value) = style.max_height {
        node.max_height = val(value);
    }
    if let Some(value) = style.flex_direction {
        node.flex_direction = match value {
            DeclarativeFlexDirection::Row => FlexDirection::Row,
            DeclarativeFlexDirection::Column => FlexDirection::Column,
            DeclarativeFlexDirection::RowReverse => FlexDirection::RowReverse,
            DeclarativeFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        };
    }
    if let Some(value) = style.justify_content {
        node.justify_content = match value {
            DeclarativeJustifyContent::FlexStart => JustifyContent::FlexStart,
            DeclarativeJustifyContent::FlexEnd => JustifyContent::FlexEnd,
            DeclarativeJustifyContent::Center => JustifyContent::Center,
            DeclarativeJustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
            DeclarativeJustifyContent::SpaceAround => JustifyContent::SpaceAround,
            DeclarativeJustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
        };
    }
    if let Some(value) = style.align_items {
        node.align_items = match value {
            DeclarativeAlignItems::Default => AlignItems::DEFAULT,
            DeclarativeAlignItems::Start => AlignItems::Start,
            DeclarativeAlignItems::End => AlignItems::End,
            DeclarativeAlignItems::FlexStart => AlignItems::FlexStart,
            DeclarativeAlignItems::FlexEnd => AlignItems::FlexEnd,
            DeclarativeAlignItems::Center => AlignItems::Center,
            DeclarativeAlignItems::Baseline => AlignItems::Baseline,
            DeclarativeAlignItems::Stretch => AlignItems::Stretch,
        };
    }
    if let Some(value) = style.align_content {
        node.align_content = match value {
            DeclarativeAlignContent::Default => AlignContent::DEFAULT,
            DeclarativeAlignContent::Start => AlignContent::Start,
            DeclarativeAlignContent::End => AlignContent::End,
            DeclarativeAlignContent::FlexStart => AlignContent::FlexStart,
            DeclarativeAlignContent::FlexEnd => AlignContent::FlexEnd,
            DeclarativeAlignContent::Center => AlignContent::Center,
            DeclarativeAlignContent::Stretch => AlignContent::Stretch,
            DeclarativeAlignContent::SpaceBetween => AlignContent::SpaceBetween,
            DeclarativeAlignContent::SpaceAround => AlignContent::SpaceAround,
            DeclarativeAlignContent::SpaceEvenly => AlignContent::SpaceEvenly,
        };
    }
    if let Some(value) = style.align_self {
        node.align_self = match value {
            DeclarativeAlignSelf::Auto => AlignSelf::Auto,
            DeclarativeAlignSelf::Start => AlignSelf::Start,
            DeclarativeAlignSelf::End => AlignSelf::End,
            DeclarativeAlignSelf::FlexStart => AlignSelf::FlexStart,
            DeclarativeAlignSelf::FlexEnd => AlignSelf::FlexEnd,
            DeclarativeAlignSelf::Center => AlignSelf::Center,
            DeclarativeAlignSelf::Baseline => AlignSelf::Baseline,
            DeclarativeAlignSelf::Stretch => AlignSelf::Stretch,
        };
    }
    if let Some(value) = style.flex_wrap {
        node.flex_wrap = match value {
            DeclarativeFlexWrap::NoWrap => FlexWrap::NoWrap,
            DeclarativeFlexWrap::Wrap => FlexWrap::Wrap,
            DeclarativeFlexWrap::WrapReverse => FlexWrap::WrapReverse,
        };
    }
    if let Some(value) = style.flex_grow {
        node.flex_grow = value;
    }
    if let Some(value) = style.flex_shrink {
        node.flex_shrink = value;
    }
    if let Some(value) = style.flex_basis {
        node.flex_basis = val(value);
    }
    if let Some(value) = style.row_gap {
        node.row_gap = val(value);
    }
    if let Some(value) = style.column_gap {
        node.column_gap = val(value);
    }
    if let Some(value) = &style.padding {
        node.padding = padding_rect(value);
    }
    if let Some(value) = &style.margin {
        node.margin = margin_rect(value);
    }
    if let Some(value) = &style.border {
        node.border = border_rect(value);
    }
    if let Some(value) = &style.border_radius {
        node.border_radius = border_radius(value);
    }
    if style.overflow_x.is_some() || style.overflow_y.is_some() {
        node.overflow = Overflow {
            x: style
                .overflow_x
                .map(overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
            y: style
                .overflow_y
                .map(overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
        };
    }
    if let Some(value) = style.display {
        node.display = match value {
            DeclarativeDisplay::Flex => Display::Flex,
            DeclarativeDisplay::Grid => Display::Grid,
            DeclarativeDisplay::Block => Display::Block,
            DeclarativeDisplay::None => Display::None,
        };
    }
    if let Some(value) = style.position_type {
        node.position_type = match value {
            DeclarativePositionType::Relative => PositionType::Relative,
            DeclarativePositionType::Absolute => PositionType::Absolute,
        };
    }
    if let Some(value) = style.left {
        node.left = val(value);
    }
    if let Some(value) = style.right {
        node.right = val(value);
    }
    if let Some(value) = style.top {
        node.top = val(value);
    }
    if let Some(value) = style.bottom {
        node.bottom = val(value);
    }

    node
}

fn margin_rect(value: &DeclarativeUiRect) -> UiRect {
    UiRect {
        left: value.left.map(val).unwrap_or(Val::ZERO),
        right: value.right.map(val).unwrap_or(Val::ZERO),
        top: value.top.map(val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(val).unwrap_or(Val::ZERO),
    }
}

fn padding_rect(value: &DeclarativeUiRect) -> UiRect {
    zero_default_rect(value)
}

fn border_rect(value: &DeclarativeUiRect) -> UiRect {
    zero_default_rect(value)
}

fn zero_default_rect(value: &DeclarativeUiRect) -> UiRect {
    UiRect {
        left: value.left.map(val).unwrap_or(Val::ZERO),
        right: value.right.map(val).unwrap_or(Val::ZERO),
        top: value.top.map(val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(val).unwrap_or(Val::ZERO),
    }
}

fn border_radius(value: &DeclarativeBorderRadius) -> BorderRadius {
    match value {
        DeclarativeBorderRadius::All { radius } => BorderRadius::all(val(*radius)),
    }
}

fn overflow_axis(value: DeclarativeOverflowAxis) -> OverflowAxis {
    match value {
        DeclarativeOverflowAxis::Visible => OverflowAxis::Visible,
        DeclarativeOverflowAxis::Clip => OverflowAxis::Clip,
        DeclarativeOverflowAxis::Hidden => OverflowAxis::Hidden,
        DeclarativeOverflowAxis::Scroll => OverflowAxis::Scroll,
    }
}

fn val(value: DeclarativeVal) -> Val {
    match value {
        DeclarativeVal::Auto => Val::Auto,
        DeclarativeVal::Px(value) => Val::Px(value),
        DeclarativeVal::Percent(value) => Val::Percent(value),
        DeclarativeVal::Vw(value) => Val::Vw(value),
        DeclarativeVal::Vh(value) => Val::Vh(value),
    }
}

pub fn parse_hex_color(raw: &str) -> Option<Color> {
    crate::style::resolve_color_value(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declarative_margin_rect_defaults_missing_edges_to_zero() {
        let rect = margin_rect(&DeclarativeUiRect {
            left: Some(DeclarativeVal::Auto),
            right: Some(DeclarativeVal::Auto),
            top: None,
            bottom: None,
        });

        assert_eq!(rect.left, Val::Auto);
        assert_eq!(rect.right, Val::Auto);
        assert_eq!(rect.top, Val::ZERO);
        assert_eq!(rect.bottom, Val::ZERO);
    }

    #[test]
    fn declarative_border_rect_defaults_missing_edges_to_zero() {
        let rect = border_rect(&DeclarativeUiRect {
            left: None,
            right: None,
            top: Some(DeclarativeVal::Px(1.0)),
            bottom: None,
        });

        assert_eq!(rect.left, Val::ZERO);
        assert_eq!(rect.right, Val::ZERO);
        assert_eq!(rect.top, Val::Px(1.0));
        assert_eq!(rect.bottom, Val::ZERO);
    }
}
