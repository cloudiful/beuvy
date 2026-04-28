use super::border_radius::parse_border_width;
use super::text::parse_text_size_token_value;
use crate::theme_config::UiThemeConfig;
use crate::utility::{
    ParseUtilityError, UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal,
    UtilityVisualStylePatch,
};

pub(super) fn apply_visual_utility_token(
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
        if variant && parse_text_size_token_value(config, token, value)?.is_some() {
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
        patch.opacity = Some(parse_opacity(value, token)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("duration-") {
        patch.transition_duration_ms = Some(parse_duration(value, token)?);
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
    let value = super::size_spacing::unwrap_arbitrary_value(raw);
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
        let inner = super::size_spacing::unwrap_arbitrary_value(raw);
        if inner.starts_with("var(") || inner.starts_with('#') {
            return Ok(None);
        }
        return super::size_spacing::parse_numeric_value(config, token, inner).map(Some);
    }
    Ok(None)
}

fn parse_opacity(raw: &str, token: &str) -> Result<f32, ParseUtilityError> {
    let raw = super::size_spacing::unwrap_arbitrary_value(raw);
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

fn parse_duration(raw: &str, token: &str) -> Result<f32, ParseUtilityError> {
    let raw = super::size_spacing::unwrap_arbitrary_value(raw);
    let raw = raw.strip_suffix("ms").unwrap_or(raw);
    raw.parse::<f32>()
        .map_err(|_| ParseUtilityError::new(token, "invalid transition duration"))
}
