use super::size_spacing::{
    merge_rect, parse_numeric_value, rect_all, rect_bottom, rect_left, rect_right, rect_top,
    rect_x, rect_y, unwrap_arbitrary_value,
};
use crate::theme_config::{UiThemeConfig, resolve_theme_numeric_value_in};
use crate::utility::{ParseUtilityError, UtilityRect, UtilityStylePatch, UtilityVal};

pub(super) fn apply_border_radius_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityStylePatch,
) -> Result<bool, ParseUtilityError> {
    if apply_border_edge_utility_token(config, token, patch)? {
        return Ok(true);
    }

    match token {
        "border" => {
            merge_rect(&mut patch.border, rect_all(UtilityVal::Px(1.0)));
            return Ok(true);
        }
        "rounded" | "rounded-ui" => {
            patch.border_radius = Some(theme_radius(config, "ui"));
            return Ok(true);
        }
        _ => {}
    }

    if let Some(value) = token.strip_prefix("border-") {
        match parse_border_width(config, token, value) {
            Ok(width) => {
                merge_rect(&mut patch.border, rect_all(width));
                return Ok(true);
            }
            Err(_) => return Ok(false),
        }
    }
    if let Some(value) = token.strip_prefix("rounded-") {
        patch.border_radius = Some(parse_rounded_value(config, token, value)?);
        return Ok(true);
    }

    Ok(false)
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

pub(super) fn parse_border_width(
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

fn theme_radius(config: &UiThemeConfig, name: &str) -> UtilityVal {
    UtilityVal::Px(
        resolve_theme_numeric_value_in(config, &format!("var(--radius-{name})")).unwrap_or(0.0),
    )
}
