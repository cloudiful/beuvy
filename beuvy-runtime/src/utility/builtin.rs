use super::types::{
    ParseUtilityError, UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay,
    UtilityFlexDirection, UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis,
    UtilityPositionType, UtilityRect, UtilityStateVariant, UtilityStylePatch,
    UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal, UtilityVisualStylePatch,
};
use crate::theme_config::{UiThemeConfig, resolve_theme_numeric_value_in};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Width,
    Height,
}

#[allow(dead_code)]
pub(crate) fn is_builtin_utility_token(config: &UiThemeConfig, token: &str) -> bool {
    parse_builtin_utility_tokens(config, &[token.to_string()]).is_ok()
}

pub(crate) fn parse_builtin_utility_tokens(
    config: &UiThemeConfig,
    tokens: &[String],
) -> Result<UtilityStylePatch, ParseUtilityError> {
    let mut patch = UtilityStylePatch::default();

    for token in tokens {
        if let Some((variant, inner)) = parse_variant_token(token)? {
            apply_visual_utility_token(
                config,
                inner,
                state_patch_mut(&mut patch, variant),
                true,
                token,
            )?;
        } else {
            apply_utility_token(config, token, &mut patch)?;
        }
    }

    Ok(patch)
}

fn parse_variant_token(
    token: &str,
) -> Result<Option<(UtilityStateVariant, &str)>, ParseUtilityError> {
    let Some((variant, inner)) = token.split_once(':') else {
        return Ok(None);
    };
    if inner.contains(':') {
        return Err(ParseUtilityError::new(
            token,
            "chained utility variants are not supported yet",
        ));
    }
    let variant = match variant {
        "hover" => UtilityStateVariant::Hover,
        "active" => UtilityStateVariant::Active,
        "focus" => UtilityStateVariant::Focus,
        "disabled" => UtilityStateVariant::Disabled,
        _ => {
            return Err(ParseUtilityError::new(token, "unsupported utility variant"));
        }
    };
    Ok(Some((variant, inner)))
}

fn state_patch_mut(
    patch: &mut UtilityStylePatch,
    variant: UtilityStateVariant,
) -> &mut UtilityVisualStylePatch {
    match variant {
        UtilityStateVariant::Hover => &mut patch.hover,
        UtilityStateVariant::Active => &mut patch.active,
        UtilityStateVariant::Focus => &mut patch.focus,
        UtilityStateVariant::Disabled => &mut patch.disabled,
    }
}

fn apply_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityStylePatch,
) -> Result<(), ParseUtilityError> {
    if apply_border_edge_utility_token(config, token, patch)? {
        return Ok(());
    }
    if let Some(value) = token.strip_prefix("text-")
        && let Some(size) = parse_text_size_token(config, token, value)?
    {
        patch.text_size = Some(size);
        return Ok(());
    }
    if apply_visual_utility_token(config, token, &mut patch.visual, false, token)? {
        return Ok(());
    }

    match token {
        "flex" => patch.display = Some(UtilityDisplay::Flex),
        "grid" => patch.display = Some(UtilityDisplay::Grid),
        "block" => patch.display = Some(UtilityDisplay::Block),
        "hidden" => patch.display = Some(UtilityDisplay::None),
        "flex-row" => patch.flex_direction = Some(UtilityFlexDirection::Row),
        "flex-col" => patch.flex_direction = Some(UtilityFlexDirection::Column),
        "flex-row-reverse" => patch.flex_direction = Some(UtilityFlexDirection::RowReverse),
        "flex-col-reverse" => patch.flex_direction = Some(UtilityFlexDirection::ColumnReverse),
        "flex-wrap" => patch.flex_wrap = Some(UtilityFlexWrap::Wrap),
        "flex-nowrap" => patch.flex_wrap = Some(UtilityFlexWrap::NoWrap),
        "flex-wrap-reverse" => patch.flex_wrap = Some(UtilityFlexWrap::WrapReverse),
        "grow" => patch.flex_grow = Some(1.0),
        "grow-0" => patch.flex_grow = Some(0.0),
        "shrink" => patch.flex_shrink = Some(1.0),
        "shrink-0" => patch.flex_shrink = Some(0.0),
        "flex-1" => {
            patch.flex_grow = Some(1.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Percent(0.0));
        }
        "flex-auto" => {
            patch.flex_grow = Some(1.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "flex-initial" => {
            patch.flex_grow = Some(0.0);
            patch.flex_shrink = Some(1.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "flex-none" => {
            patch.flex_grow = Some(0.0);
            patch.flex_shrink = Some(0.0);
            patch.flex_basis = Some(UtilityVal::Auto);
        }
        "items-start" => patch.align_items = Some(UtilityAlignItems::FlexStart),
        "items-end" => patch.align_items = Some(UtilityAlignItems::FlexEnd),
        "items-center" => patch.align_items = Some(UtilityAlignItems::Center),
        "items-baseline" => patch.align_items = Some(UtilityAlignItems::Baseline),
        "items-stretch" => patch.align_items = Some(UtilityAlignItems::Stretch),
        "content-start" => patch.align_content = Some(UtilityAlignContent::FlexStart),
        "content-end" => patch.align_content = Some(UtilityAlignContent::FlexEnd),
        "content-center" => patch.align_content = Some(UtilityAlignContent::Center),
        "content-stretch" => patch.align_content = Some(UtilityAlignContent::Stretch),
        "content-between" => patch.align_content = Some(UtilityAlignContent::SpaceBetween),
        "content-around" => patch.align_content = Some(UtilityAlignContent::SpaceAround),
        "content-evenly" => patch.align_content = Some(UtilityAlignContent::SpaceEvenly),
        "self-auto" => patch.align_self = Some(UtilityAlignSelf::Auto),
        "self-start" => patch.align_self = Some(UtilityAlignSelf::FlexStart),
        "self-end" => patch.align_self = Some(UtilityAlignSelf::FlexEnd),
        "self-center" => patch.align_self = Some(UtilityAlignSelf::Center),
        "self-baseline" => patch.align_self = Some(UtilityAlignSelf::Baseline),
        "self-stretch" => patch.align_self = Some(UtilityAlignSelf::Stretch),
        "justify-start" => patch.justify_content = Some(UtilityJustifyContent::FlexStart),
        "justify-end" => patch.justify_content = Some(UtilityJustifyContent::FlexEnd),
        "justify-center" => patch.justify_content = Some(UtilityJustifyContent::Center),
        "justify-between" => patch.justify_content = Some(UtilityJustifyContent::SpaceBetween),
        "justify-around" => patch.justify_content = Some(UtilityJustifyContent::SpaceAround),
        "justify-evenly" => patch.justify_content = Some(UtilityJustifyContent::SpaceEvenly),
        "relative" => patch.position_type = Some(UtilityPositionType::Relative),
        "absolute" => patch.position_type = Some(UtilityPositionType::Absolute),
        "overflow-visible" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Visible);
            patch.overflow_y = Some(UtilityOverflowAxis::Visible);
        }
        "overflow-hidden" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Hidden);
            patch.overflow_y = Some(UtilityOverflowAxis::Hidden);
        }
        "overflow-clip" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Clip);
            patch.overflow_y = Some(UtilityOverflowAxis::Clip);
        }
        "overflow-scroll" => {
            patch.overflow_x = Some(UtilityOverflowAxis::Scroll);
            patch.overflow_y = Some(UtilityOverflowAxis::Scroll);
        }
        "overflow-x-visible" => patch.overflow_x = Some(UtilityOverflowAxis::Visible),
        "overflow-x-hidden" => patch.overflow_x = Some(UtilityOverflowAxis::Hidden),
        "overflow-x-clip" => patch.overflow_x = Some(UtilityOverflowAxis::Clip),
        "overflow-x-scroll" => patch.overflow_x = Some(UtilityOverflowAxis::Scroll),
        "overflow-y-visible" => patch.overflow_y = Some(UtilityOverflowAxis::Visible),
        "overflow-y-hidden" => patch.overflow_y = Some(UtilityOverflowAxis::Hidden),
        "overflow-y-clip" => patch.overflow_y = Some(UtilityOverflowAxis::Clip),
        "overflow-y-scroll" => patch.overflow_y = Some(UtilityOverflowAxis::Scroll),
        "w-full" => patch.width = Some(UtilityVal::Percent(100.0)),
        "w-auto" => patch.width = Some(UtilityVal::Auto),
        "w-screen" => patch.width = Some(UtilityVal::Vw(100.0)),
        "h-full" => patch.height = Some(UtilityVal::Percent(100.0)),
        "h-auto" => patch.height = Some(UtilityVal::Auto),
        "h-screen" => patch.height = Some(UtilityVal::Vh(100.0)),
        "min-w-0" => patch.min_width = Some(UtilityVal::Px(0.0)),
        "min-w-full" => patch.min_width = Some(UtilityVal::Percent(100.0)),
        "min-w-auto" => patch.min_width = Some(UtilityVal::Auto),
        "min-w-screen" => patch.min_width = Some(UtilityVal::Vw(100.0)),
        "min-h-0" => patch.min_height = Some(UtilityVal::Px(0.0)),
        "min-h-full" => patch.min_height = Some(UtilityVal::Percent(100.0)),
        "min-h-auto" => patch.min_height = Some(UtilityVal::Auto),
        "min-h-screen" => patch.min_height = Some(UtilityVal::Vh(100.0)),
        "max-w-full" => patch.max_width = Some(UtilityVal::Percent(100.0)),
        "max-w-screen" => patch.max_width = Some(UtilityVal::Vw(100.0)),
        "max-h-full" => patch.max_height = Some(UtilityVal::Percent(100.0)),
        "max-h-screen" => patch.max_height = Some(UtilityVal::Vh(100.0)),
        "basis-0" => patch.flex_basis = Some(UtilityVal::Px(0.0)),
        "basis-auto" => patch.flex_basis = Some(UtilityVal::Auto),
        "basis-full" => patch.flex_basis = Some(UtilityVal::Percent(100.0)),
        "inset-0" => apply_inset_all(patch, UtilityVal::Px(0.0)),
        "inset-x-0" => apply_inset_x(patch, UtilityVal::Px(0.0)),
        "inset-y-0" => apply_inset_y(patch, UtilityVal::Px(0.0)),
        "border" => merge_rect(&mut patch.border, rect_all(UtilityVal::Px(1.0))),
        "border-0" => merge_rect(&mut patch.border, rect_all(UtilityVal::Px(0.0))),
        "border-2" => merge_rect(&mut patch.border, rect_all(UtilityVal::Px(2.0))),
        "border-4" => merge_rect(&mut patch.border, rect_all(UtilityVal::Px(4.0))),
        "border-8" => merge_rect(&mut patch.border, rect_all(UtilityVal::Px(8.0))),
        "rounded-none" => patch.border_radius = Some(UtilityVal::Px(0.0)),
        "rounded" | "rounded-ui" => patch.border_radius = Some(theme_radius(config, "ui")),
        "rounded-panel" => patch.border_radius = Some(theme_radius(config, "panel")),
        "rounded-control" => patch.border_radius = Some(theme_radius(config, "control")),
        "rounded-pill" => patch.border_radius = Some(theme_radius(config, "pill")),
        _ => {
            if let Some(value) = token.strip_prefix("w-") {
                patch.width = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("h-") {
                patch.height = Some(parse_size_value(config, token, value, Axis::Height)?);
            } else if let Some(value) = token.strip_prefix("min-w-") {
                patch.min_width = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("min-h-") {
                patch.min_height = Some(parse_size_value(config, token, value, Axis::Height)?);
            } else if let Some(value) = token.strip_prefix("max-w-") {
                patch.max_width = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("max-h-") {
                patch.max_height = Some(parse_size_value(config, token, value, Axis::Height)?);
            } else if let Some(value) = token.strip_prefix("basis-") {
                patch.flex_basis = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("gap-x-") {
                patch.column_gap = Some(parse_spacing_value(config, token, value)?);
            } else if let Some(value) = token.strip_prefix("gap-y-") {
                patch.row_gap = Some(parse_spacing_value(config, token, value)?);
            } else if let Some(value) = token.strip_prefix("gap-") {
                let val = parse_spacing_value(config, token, value)?;
                patch.row_gap = Some(val);
                patch.column_gap = Some(val);
            } else if let Some(value) = token.strip_prefix("px-") {
                merge_rect(
                    &mut patch.padding,
                    rect_x(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("py-") {
                merge_rect(
                    &mut patch.padding,
                    rect_y(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("pt-") {
                merge_rect(
                    &mut patch.padding,
                    rect_top(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("pr-") {
                merge_rect(
                    &mut patch.padding,
                    rect_right(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("pb-") {
                merge_rect(
                    &mut patch.padding,
                    rect_bottom(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("pl-") {
                merge_rect(
                    &mut patch.padding,
                    rect_left(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("p-") {
                merge_rect(
                    &mut patch.padding,
                    rect_all(parse_spacing_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("mx-") {
                merge_rect(
                    &mut patch.margin,
                    rect_x(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("my-") {
                merge_rect(
                    &mut patch.margin,
                    rect_y(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("mt-") {
                merge_rect(
                    &mut patch.margin,
                    rect_top(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("mr-") {
                merge_rect(
                    &mut patch.margin,
                    rect_right(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("mb-") {
                merge_rect(
                    &mut patch.margin,
                    rect_bottom(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("ml-") {
                merge_rect(
                    &mut patch.margin,
                    rect_left(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("m-") {
                merge_rect(
                    &mut patch.margin,
                    rect_all(parse_margin_value(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("inset-x-") {
                let inset = parse_size_value(config, token, value, Axis::Width)?;
                apply_inset_x(patch, inset);
            } else if let Some(value) = token.strip_prefix("inset-y-") {
                let inset = parse_size_value(config, token, value, Axis::Height)?;
                apply_inset_y(patch, inset);
            } else if let Some(value) = token.strip_prefix("inset-") {
                let inset = parse_size_value(config, token, value, Axis::Width)?;
                apply_inset_all(patch, inset);
            } else if let Some(value) = token.strip_prefix("top-") {
                patch.top = Some(parse_size_value(config, token, value, Axis::Height)?);
            } else if let Some(value) = token.strip_prefix("right-") {
                patch.right = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("bottom-") {
                patch.bottom = Some(parse_size_value(config, token, value, Axis::Height)?);
            } else if let Some(value) = token.strip_prefix("left-") {
                patch.left = Some(parse_size_value(config, token, value, Axis::Width)?);
            } else if let Some(value) = token.strip_prefix("border-") {
                merge_rect(
                    &mut patch.border,
                    rect_all(parse_border_width(config, token, value)?),
                );
            } else if let Some(value) = token.strip_prefix("rounded-") {
                patch.border_radius = Some(parse_rounded_value(config, token, value)?);
            } else {
                return Err(ParseUtilityError::new(token, "unknown utility class"));
            }
        }
    }

    Ok(())
}

fn apply_border_edge_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityStylePatch,
) -> Result<bool, ParseUtilityError> {
    for (prefix, rect) in [
        ("border-x", rect_x as fn(UtilityVal) -> UtilityRect),
        ("border-y", rect_y as fn(UtilityVal) -> UtilityRect),
        ("border-t", rect_top as fn(UtilityVal) -> UtilityRect),
        ("border-r", rect_right as fn(UtilityVal) -> UtilityRect),
        ("border-b", rect_bottom as fn(UtilityVal) -> UtilityRect),
        ("border-l", rect_left as fn(UtilityVal) -> UtilityRect),
    ] {
        if token == prefix {
            merge_rect(&mut patch.border, rect(UtilityVal::Px(1.0)));
            return Ok(true);
        }

        let Some(raw) = token.strip_prefix(prefix) else {
            continue;
        };
        let Some(raw) = raw.strip_prefix('-') else {
            continue;
        };
        let width = parse_border_width(config, token, raw)?;
        merge_rect(&mut patch.border, rect(width));
        return Ok(true);
    }

    Ok(false)
}

fn apply_visual_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityVisualStylePatch,
    variant: bool,
    original_token: &str,
) -> Result<bool, ParseUtilityError> {
    match token {
        "transition" => {
            patch.transition_property = Some(UtilityTransitionProperty::All);
            return Ok(true);
        }
        "transition-colors" => {
            patch.transition_property = Some(UtilityTransitionProperty::Colors);
            return Ok(true);
        }
        "ease-linear" => {
            patch.transition_timing = Some(UtilityTransitionTiming::Linear);
            return Ok(true);
        }
        "ease-in" => {
            patch.transition_timing = Some(UtilityTransitionTiming::EaseIn);
            return Ok(true);
        }
        "ease-out" => {
            patch.transition_timing = Some(UtilityTransitionTiming::EaseOut);
            return Ok(true);
        }
        "ease-in-out" => {
            patch.transition_timing = Some(UtilityTransitionTiming::EaseInOut);
            return Ok(true);
        }
        "outline" => {
            patch.outline_width = Some(UtilityVal::Px(1.0));
            return Ok(true);
        }
        "outline-0" => {
            patch.outline_width = Some(UtilityVal::Px(0.0));
            return Ok(true);
        }
        "outline-1" => {
            patch.outline_width = Some(UtilityVal::Px(1.0));
            return Ok(true);
        }
        "outline-2" => {
            patch.outline_width = Some(UtilityVal::Px(2.0));
            return Ok(true);
        }
        "outline-4" => {
            patch.outline_width = Some(UtilityVal::Px(4.0));
            return Ok(true);
        }
        "outline-8" => {
            patch.outline_width = Some(UtilityVal::Px(8.0));
            return Ok(true);
        }
        _ => {}
    }

    if let Some(value) = token.strip_prefix("bg-") {
        patch.background_color = Some(theme_color_reference(value));
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("text-") {
        if variant && parse_text_size_token(config, token, value)?.is_some() {
            return Err(ParseUtilityError::new(
                original_token,
                "state variants do not support text size utilities",
            ));
        }
        patch.text_color = Some(theme_color_reference(value));
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("border-") {
        if parse_border_width(config, token, value).is_ok() {
            if variant {
                return Err(ParseUtilityError::new(
                    original_token,
                    "state variants do not support border width utilities",
                ));
            }
            return Ok(false);
        }
        patch.border_color = Some(theme_color_reference(value));
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("outline-") {
        if let Some(width) = parse_arbitrary_outline_width(config, token, value)? {
            patch.outline_width = Some(width);
        } else {
            patch.outline_color = Some(theme_color_reference(value));
        }
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("opacity-") {
        patch.opacity = Some(parse_opacity(token, value)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("duration-") {
        patch.transition_duration_ms = Some(parse_duration(token, value)?);
        return Ok(true);
    }

    if variant {
        return Err(ParseUtilityError::new(
            original_token,
            "state variants only support visual utilities",
        ));
    }

    Ok(false)
}

fn theme_color_reference(raw: &str) -> String {
    let value = unwrap_arbitrary_value(raw);
    if value.starts_with('#') || value.starts_with("var(") {
        value.to_string()
    } else {
        format!("var(--color-{value})")
    }
}

fn parse_arbitrary_outline_width(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<Option<UtilityVal>, ParseUtilityError> {
    if raw.starts_with('[') && raw.ends_with(']') {
        let inner = unwrap_arbitrary_value(raw);
        if inner.starts_with("var(") || inner.starts_with('#') {
            return Ok(None);
        }
        return parse_numeric_value(config, token, inner).map(Some);
    }
    Ok(None)
}

fn parse_opacity(token: &str, raw: &str) -> Result<f32, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    if let Some(percent) = raw.strip_suffix('%') {
        return percent
            .parse::<f32>()
            .map(|value| (value / 100.0).clamp(0.0, 1.0))
            .map_err(|_| ParseUtilityError::new(token, "invalid opacity value"));
    }
    let value = raw
        .parse::<f32>()
        .map_err(|_| ParseUtilityError::new(token, "invalid opacity value"))?;
    if value > 1.0 {
        Ok((value / 100.0).clamp(0.0, 1.0))
    } else {
        Ok(value.clamp(0.0, 1.0))
    }
}

fn parse_duration(token: &str, raw: &str) -> Result<f32, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    let raw = raw.strip_suffix("ms").unwrap_or(raw);
    raw.parse::<f32>()
        .map_err(|_| ParseUtilityError::new(token, "invalid transition duration"))
}

fn parse_text_size_token(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<Option<f32>, ParseUtilityError> {
    match raw {
        "hint" => Ok(Some(theme_text_size(config, "hint"))),
        "meta" => Ok(Some(theme_text_size(config, "meta"))),
        "body" => Ok(Some(theme_text_size(config, "body"))),
        "control" => Ok(Some(theme_text_size(config, "control"))),
        "control-compact" => Ok(Some(theme_text_size(config, "control-compact"))),
        "title" => Ok(Some(theme_text_size(config, "title"))),
        "display" => Ok(Some(theme_text_size(config, "display"))),
        _ => {
            if raw.starts_with('[') {
                return Ok(parse_text_size_value(config, token, raw).ok());
            }
            Ok(None)
        }
    }
}

fn parse_text_size_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<f32, ParseUtilityError> {
    match parse_numeric_value(config, token, unwrap_arbitrary_value(raw))? {
        UtilityVal::Px(value) => Ok(value),
        UtilityVal::Percent(_) | UtilityVal::Vw(_) | UtilityVal::Vh(_) | UtilityVal::Auto => {
            Err(ParseUtilityError::new(token, "invalid text size value"))
        }
    }
}

fn parse_size_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
    axis: Axis,
) -> Result<UtilityVal, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    match raw {
        "auto" => Ok(UtilityVal::Auto),
        "full" => Ok(UtilityVal::Percent(100.0)),
        "screen" => match axis {
            Axis::Width => Ok(UtilityVal::Vw(100.0)),
            Axis::Height => Ok(UtilityVal::Vh(100.0)),
        },
        _ => parse_numeric_value(config, token, raw),
    }
}

fn parse_spacing_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<UtilityVal, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    if raw == "px" {
        return Ok(UtilityVal::Px(1.0));
    }
    parse_numeric_value(config, token, raw)
}

fn parse_margin_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<UtilityVal, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    if raw == "auto" {
        return Ok(UtilityVal::Auto);
    }
    parse_spacing_value(config, token, raw)
}

fn parse_border_width(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<UtilityVal, ParseUtilityError> {
    let raw = unwrap_arbitrary_value(raw);
    if let Some(value) = raw.strip_suffix("px") {
        return value
            .parse::<f32>()
            .map(UtilityVal::Px)
            .map_err(|_| ParseUtilityError::new(token, "invalid border width"));
    }
    match raw {
        "0" => Ok(UtilityVal::Px(0.0)),
        "2" => Ok(UtilityVal::Px(2.0)),
        "4" => Ok(UtilityVal::Px(4.0)),
        "8" => Ok(UtilityVal::Px(8.0)),
        _ => match parse_numeric_value(config, token, raw)? {
            UtilityVal::Px(value) => Ok(UtilityVal::Px(value)),
            UtilityVal::Percent(_) | UtilityVal::Vw(_) | UtilityVal::Vh(_) | UtilityVal::Auto => {
                Err(ParseUtilityError::new(token, "unsupported border width"))
            }
        },
    }
}

fn parse_rounded_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<UtilityVal, ParseUtilityError> {
    match unwrap_arbitrary_value(raw) {
        "none" => Ok(UtilityVal::Px(0.0)),
        "ui" => Ok(theme_radius(config, "ui")),
        "panel" => Ok(theme_radius(config, "panel")),
        "control" => Ok(theme_radius(config, "control")),
        "pill" => Ok(theme_radius(config, "pill")),
        other => parse_numeric_value(config, token, other),
    }
}

fn unwrap_arbitrary_value(raw: &str) -> &str {
    raw.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(raw)
}

fn parse_numeric_value(
    config: &UiThemeConfig,
    token: &str,
    raw: &str,
) -> Result<UtilityVal, ParseUtilityError> {
    if raw.starts_with("var(") {
        let value = resolve_theme_numeric_value_in(config, raw)
            .ok_or_else(|| ParseUtilityError::new(token, "unknown theme numeric variable"))?;
        return Ok(UtilityVal::Px(value));
    }
    if let Some(num) = raw.strip_suffix("vw") {
        return num
            .parse::<f32>()
            .map(UtilityVal::Vw)
            .map_err(|_| ParseUtilityError::new(token, "invalid vw value"));
    }
    if let Some(num) = raw.strip_suffix("vh") {
        return num
            .parse::<f32>()
            .map(UtilityVal::Vh)
            .map_err(|_| ParseUtilityError::new(token, "invalid vh value"));
    }
    if let Some(num) = raw.strip_suffix('%') {
        return num
            .parse::<f32>()
            .map(UtilityVal::Percent)
            .map_err(|_| ParseUtilityError::new(token, "invalid percent value"));
    }
    if let Some(num) = raw.strip_suffix("px") {
        let normalized = num.replace('_', ".");
        let number = normalized
            .parse::<f32>()
            .map_err(|_| ParseUtilityError::new(token, "invalid px value"))?;
        return Ok(UtilityVal::Px(number));
    }
    let normalized = raw.replace('_', ".");
    let number = normalized
        .parse::<f32>()
        .map_err(|_| ParseUtilityError::new(token, "invalid numeric value"))?;
    Ok(UtilityVal::Px(number * 4.0))
}

fn theme_radius(config: &UiThemeConfig, name: &str) -> UtilityVal {
    UtilityVal::Px(
        resolve_theme_numeric_value_in(config, &format!("var(--radius-{name})")).unwrap_or(0.0),
    )
}

fn theme_text_size(config: &UiThemeConfig, name: &str) -> f32 {
    resolve_theme_numeric_value_in(config, &format!("var(--text-{name})")).unwrap_or(0.0)
}

fn merge_rect(target: &mut Option<UtilityRect>, patch: UtilityRect) {
    let rect = target.get_or_insert_with(UtilityRect::default);
    if patch.left.is_some() {
        rect.left = patch.left;
    }
    if patch.right.is_some() {
        rect.right = patch.right;
    }
    if patch.top.is_some() {
        rect.top = patch.top;
    }
    if patch.bottom.is_some() {
        rect.bottom = patch.bottom;
    }
}

fn rect_all(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        right: Some(value),
        top: Some(value),
        bottom: Some(value),
    }
}

fn rect_x(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        right: Some(value),
        ..Default::default()
    }
}

fn rect_y(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        top: Some(value),
        bottom: Some(value),
        ..Default::default()
    }
}

fn rect_top(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        top: Some(value),
        ..Default::default()
    }
}

fn rect_right(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        right: Some(value),
        ..Default::default()
    }
}

fn rect_bottom(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        bottom: Some(value),
        ..Default::default()
    }
}

fn rect_left(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        ..Default::default()
    }
}

fn apply_inset_all(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.left = Some(value);
    patch.right = Some(value);
    patch.top = Some(value);
    patch.bottom = Some(value);
}

fn apply_inset_x(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.left = Some(value);
    patch.right = Some(value);
}

fn apply_inset_y(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.top = Some(value);
    patch.bottom = Some(value);
}
