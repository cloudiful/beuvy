use super::*;

pub(crate) fn parse_node_style(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeNodeStyle, DeclarativeUiAssetLoadError> {
    let class_patch = parse_utility_class_patch(node)?;
    Ok(DeclarativeNodeStyle {
        width: map_utility_val(class_patch.width),
        height: map_utility_val(class_patch.height),
        min_width: map_utility_val(class_patch.min_width),
        min_height: map_utility_val(class_patch.min_height),
        max_width: map_utility_val(class_patch.max_width),
        max_height: map_utility_val(class_patch.max_height),
        flex_direction: class_patch.flex_direction.map(flex_direction_from_utility),
        justify_content: class_patch.justify_content.map(justify_from_utility),
        align_items: class_patch.align_items.map(align_items_from_utility),
        align_content: class_patch.align_content.map(align_content_from_utility),
        align_self: class_patch.align_self.map(align_self_from_utility),
        flex_wrap: class_patch.flex_wrap.map(flex_wrap_from_utility),
        flex_grow: class_patch.flex_grow,
        flex_shrink: class_patch.flex_shrink,
        flex_basis: map_utility_val(class_patch.flex_basis),
        row_gap: map_utility_val(class_patch.row_gap),
        column_gap: map_utility_val(class_patch.column_gap),
        padding: class_patch.padding.as_ref().map(rect_from_utility),
        margin: class_patch.margin.as_ref().map(rect_from_utility),
        border: class_patch.border.as_ref().map(rect_from_utility),
        border_radius: class_patch.border_radius.map(border_radius_from_utility),
        overflow_x: class_patch.overflow_x.map(overflow_from_utility),
        overflow_y: class_patch.overflow_y.map(overflow_from_utility),
        display: class_patch.display.map(display_from_utility),
        position_type: class_patch.position_type.map(position_from_utility),
        left: map_utility_val(class_patch.left),
        right: map_utility_val(class_patch.right),
        top: map_utility_val(class_patch.top),
        bottom: map_utility_val(class_patch.bottom),
    })
}

pub(super) fn parse_text_style(
    node: XmlNode<'_, '_>,
    tag: &str,
) -> Result<DeclarativeTextStyle, DeclarativeUiAssetLoadError> {
    let class_patch = parse_utility_class_patch(node)?;
    Ok(DeclarativeTextStyle {
        size: class_patch
            .text_size
            .unwrap_or_else(|| default_text_size_for_tag(tag)),
        color: class_patch.visual.text_color.clone(),
        visual_style: visual_style_from_utility(&class_patch.visual),
        state_visual_styles: state_visual_styles_from_utility(&class_patch),
    })
}

pub(crate) fn parse_visual_style(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeVisualStyle, DeclarativeUiAssetLoadError> {
    let class_patch = parse_utility_class_patch(node)?;
    Ok(visual_style_from_utility(&class_patch.visual))
}

pub(crate) fn parse_state_visual_styles(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeStateVisualStyles, DeclarativeUiAssetLoadError> {
    let class_patch = parse_utility_class_patch(node)?;
    Ok(state_visual_styles_from_utility(&class_patch))
}

pub(crate) fn parse_utility_class_patch(
    node: XmlNode<'_, '_>,
) -> Result<UtilityStylePatch, DeclarativeUiAssetLoadError> {
    let classes = attr(node, "class").unwrap_or_default();
    crate::style::parse_style_classes(classes).map_err(|error| match error {
        DeclarativeUiAssetLoadError::InvalidDsl(reason) => {
            attr_error(node, "class", classes, &reason)
        }
        other => other,
    })
}

pub(crate) fn parse_class_bindings(
    node: XmlNode<'_, '_>,
    _state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<Vec<DeclarativeClassBinding>, DeclarativeUiAssetLoadError> {
    let Some(raw) = bound_attr(node, "class") else {
        return Ok(Vec::new());
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![DeclarativeClassBinding::RuntimeExpr {
        expr: parse_runtime_expr(node, "v-bind-class", raw)?,
    }])
}

pub(super) fn map_utility_val(value: Option<UtilityVal>) -> Option<DeclarativeVal> {
    match value? {
        UtilityVal::Auto => Some(DeclarativeVal::Auto),
        UtilityVal::Px(value) => Some(DeclarativeVal::Px(value)),
        UtilityVal::Percent(value) => Some(DeclarativeVal::Percent(value)),
        UtilityVal::Vw(value) => Some(DeclarativeVal::Vw(value)),
        UtilityVal::Vh(value) => Some(DeclarativeVal::Vh(value)),
    }
}

pub(super) fn visual_style_from_utility(value: &UtilityVisualStylePatch) -> DeclarativeVisualStyle {
    DeclarativeVisualStyle {
        background_color: value.background_color.clone(),
        text_color: value.text_color.clone(),
        border_color: value.border_color.clone(),
        outline_width: map_utility_val(value.outline_width),
        outline_color: value.outline_color.clone(),
        opacity: value.opacity,
        transition_property: value
            .transition_property
            .map(transition_property_from_utility),
        transition_duration_ms: value.transition_duration_ms,
        transition_timing: value.transition_timing.map(transition_timing_from_utility),
    }
}

pub(super) fn state_visual_styles_from_utility(
    value: &UtilityStylePatch,
) -> DeclarativeStateVisualStyles {
    DeclarativeStateVisualStyles {
        hover: visual_style_from_utility(&value.hover),
        active: visual_style_from_utility(&value.active),
        focus: visual_style_from_utility(&value.focus),
    }
}

pub(super) fn transition_property_from_utility(
    value: UtilityTransitionProperty,
) -> DeclarativeTransitionProperty {
    value.into()
}

pub(super) fn transition_timing_from_utility(
    value: UtilityTransitionTiming,
) -> DeclarativeTransitionTiming {
    value.into()
}

pub(super) fn rect_from_utility(rect: &beuvy_runtime::utility::UtilityRect) -> DeclarativeUiRect {
    DeclarativeUiRect {
        left: map_utility_val(rect.left),
        right: map_utility_val(rect.right),
        top: map_utility_val(rect.top),
        bottom: map_utility_val(rect.bottom),
    }
}

pub(super) fn border_radius_from_utility(value: UtilityVal) -> DeclarativeBorderRadius {
    DeclarativeBorderRadius::All {
        radius: map_utility_val(Some(value)).expect("utility border radius should map"),
    }
}

pub(super) fn flex_direction_from_utility(value: UtilityFlexDirection) -> DeclarativeFlexDirection {
    match value {
        UtilityFlexDirection::Row => DeclarativeFlexDirection::Row,
        UtilityFlexDirection::Column => DeclarativeFlexDirection::Column,
        UtilityFlexDirection::RowReverse => DeclarativeFlexDirection::RowReverse,
        UtilityFlexDirection::ColumnReverse => DeclarativeFlexDirection::ColumnReverse,
    }
}

pub(super) fn justify_from_utility(value: UtilityJustifyContent) -> DeclarativeJustifyContent {
    match value {
        UtilityJustifyContent::FlexStart => DeclarativeJustifyContent::FlexStart,
        UtilityJustifyContent::FlexEnd => DeclarativeJustifyContent::FlexEnd,
        UtilityJustifyContent::Center => DeclarativeJustifyContent::Center,
        UtilityJustifyContent::SpaceBetween => DeclarativeJustifyContent::SpaceBetween,
        UtilityJustifyContent::SpaceAround | UtilityJustifyContent::SpaceEvenly => {
            DeclarativeJustifyContent::SpaceAround
        }
    }
}

pub(super) fn align_items_from_utility(value: UtilityAlignItems) -> DeclarativeAlignItems {
    match value {
        UtilityAlignItems::FlexStart => DeclarativeAlignItems::FlexStart,
        UtilityAlignItems::FlexEnd => DeclarativeAlignItems::FlexEnd,
        UtilityAlignItems::Center => DeclarativeAlignItems::Center,
        UtilityAlignItems::Baseline => DeclarativeAlignItems::Baseline,
        UtilityAlignItems::Stretch => DeclarativeAlignItems::Stretch,
    }
}

pub(super) fn align_content_from_utility(value: UtilityAlignContent) -> DeclarativeAlignContent {
    match value {
        UtilityAlignContent::FlexStart => DeclarativeAlignContent::FlexStart,
        UtilityAlignContent::FlexEnd => DeclarativeAlignContent::FlexEnd,
        UtilityAlignContent::Center => DeclarativeAlignContent::Center,
        UtilityAlignContent::Stretch => DeclarativeAlignContent::Stretch,
        UtilityAlignContent::SpaceBetween => DeclarativeAlignContent::SpaceBetween,
        UtilityAlignContent::SpaceAround => DeclarativeAlignContent::SpaceAround,
        UtilityAlignContent::SpaceEvenly => DeclarativeAlignContent::SpaceEvenly,
    }
}

pub(super) fn align_self_from_utility(value: UtilityAlignSelf) -> DeclarativeAlignSelf {
    match value {
        UtilityAlignSelf::Auto => DeclarativeAlignSelf::Auto,
        UtilityAlignSelf::FlexStart => DeclarativeAlignSelf::FlexStart,
        UtilityAlignSelf::FlexEnd => DeclarativeAlignSelf::FlexEnd,
        UtilityAlignSelf::Center => DeclarativeAlignSelf::Center,
        UtilityAlignSelf::Baseline => DeclarativeAlignSelf::Baseline,
        UtilityAlignSelf::Stretch => DeclarativeAlignSelf::Stretch,
    }
}

pub(super) fn flex_wrap_from_utility(value: UtilityFlexWrap) -> DeclarativeFlexWrap {
    match value {
        UtilityFlexWrap::NoWrap => DeclarativeFlexWrap::NoWrap,
        UtilityFlexWrap::Wrap => DeclarativeFlexWrap::Wrap,
        UtilityFlexWrap::WrapReverse => DeclarativeFlexWrap::WrapReverse,
    }
}

pub(super) fn overflow_from_utility(value: UtilityOverflowAxis) -> DeclarativeOverflowAxis {
    match value {
        UtilityOverflowAxis::Visible => DeclarativeOverflowAxis::Visible,
        UtilityOverflowAxis::Clip => DeclarativeOverflowAxis::Clip,
        UtilityOverflowAxis::Hidden => DeclarativeOverflowAxis::Hidden,
        UtilityOverflowAxis::Scroll => DeclarativeOverflowAxis::Scroll,
    }
}

pub(super) fn display_from_utility(value: UtilityDisplay) -> DeclarativeDisplay {
    match value {
        UtilityDisplay::Flex => DeclarativeDisplay::Flex,
        UtilityDisplay::Grid => DeclarativeDisplay::Grid,
        UtilityDisplay::None => DeclarativeDisplay::None,
        UtilityDisplay::Block => DeclarativeDisplay::Block,
    }
}

pub(super) fn position_from_utility(value: UtilityPositionType) -> DeclarativePositionType {
    match value {
        UtilityPositionType::Relative => DeclarativePositionType::Relative,
        UtilityPositionType::Absolute => DeclarativePositionType::Absolute,
    }
}
