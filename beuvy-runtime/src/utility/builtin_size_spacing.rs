use crate::utility::{ParseUtilityError, UtilityRect, UtilityStylePatch, UtilityVal};
use crate::theme_config::{UiThemeConfig, resolve_theme_numeric_value_in};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Width,
    Height,
}

pub(super) fn apply_size_spacing_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityStylePatch,
) -> Result<bool, ParseUtilityError> {
    if let Some(value) = token.strip_prefix("w-") {
        patch.width = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("h-") {
        patch.height = Some(parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("min-w-") {
        patch.min_width = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("min-h-") {
        patch.min_height = Some(parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("max-w-") {
        patch.max_width = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("max-h-") {
        patch.max_height = Some(parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("basis-") {
        patch.flex_basis = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("gap-x-") {
        patch.column_gap = Some(parse_spacing_value(config, token, value)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("gap-y-") {
        patch.row_gap = Some(parse_spacing_value(config, token, value)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("gap-") {
        let val = parse_spacing_value(config, token, value)?;
        patch.row_gap = Some(val);
        patch.column_gap = Some(val);
        return Ok(true);
    }
    if let Some((target, rect)) = padding_token(token) {
        merge_rect(&mut patch.padding, rect(parse_spacing_value(config, token, target)?));
        return Ok(true);
    }
    if let Some((target, rect)) = margin_token(token) {
        merge_rect(&mut patch.margin, rect(parse_margin_value(config, token, target)?));
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("inset-x-") {
        apply_inset_x(patch, parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("inset-y-") {
        apply_inset_y(patch, parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("inset-") {
        apply_inset_all(patch, parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("top-") {
        patch.top = Some(parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("right-") {
        patch.right = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("bottom-") {
        patch.bottom = Some(parse_size_value(config, token, value, Axis::Height)?);
        return Ok(true);
    }
    if let Some(value) = token.strip_prefix("left-") {
        patch.left = Some(parse_size_value(config, token, value, Axis::Width)?);
        return Ok(true);
    }

    Ok(false)
}

type RectFactory = fn(UtilityVal) -> UtilityRect;

fn padding_token(token: &str) -> Option<(&str, RectFactory)> {
    match_prefix(
        token,
        &[
            ("px-", rect_x as RectFactory),
            ("py-", rect_y as RectFactory),
            ("pt-", rect_top as RectFactory),
            ("pr-", rect_right as RectFactory),
            ("pb-", rect_bottom as RectFactory),
            ("pl-", rect_left as RectFactory),
            ("p-", rect_all as RectFactory),
        ],
    )
}

fn margin_token(token: &str) -> Option<(&str, RectFactory)> {
    match_prefix(
        token,
        &[
            ("mx-", rect_x as RectFactory),
            ("my-", rect_y as RectFactory),
            ("mt-", rect_top as RectFactory),
            ("mr-", rect_right as RectFactory),
            ("mb-", rect_bottom as RectFactory),
            ("ml-", rect_left as RectFactory),
            ("m-", rect_all as RectFactory),
        ],
    )
}

fn match_prefix<'a>(token: &'a str, pairs: &[(&'static str, RectFactory)]) -> Option<(&'a str, RectFactory)> {
    for (prefix, rect) in pairs {
        if let Some(value) = token.strip_prefix(prefix) {
            return Some((value, *rect));
        }
    }
    None
}

pub(super) fn unwrap_arbitrary_value(raw: &str) -> &str {
    raw.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(raw)
}

pub(super) fn parse_numeric_value(
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

pub(super) fn merge_rect(target: &mut Option<UtilityRect>, patch: UtilityRect) {
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

pub(super) fn rect_all(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        right: Some(value),
        top: Some(value),
        bottom: Some(value),
    }
}

pub(super) fn rect_x(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        right: Some(value),
        ..Default::default()
    }
}

pub(super) fn rect_y(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        top: Some(value),
        bottom: Some(value),
        ..Default::default()
    }
}

pub(super) fn rect_top(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        top: Some(value),
        ..Default::default()
    }
}

pub(super) fn rect_right(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        right: Some(value),
        ..Default::default()
    }
}

pub(super) fn rect_bottom(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        bottom: Some(value),
        ..Default::default()
    }
}

pub(super) fn rect_left(value: UtilityVal) -> UtilityRect {
    UtilityRect {
        left: Some(value),
        ..Default::default()
    }
}

pub(super) fn apply_inset_all(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.left = Some(value);
    patch.right = Some(value);
    patch.top = Some(value);
    patch.bottom = Some(value);
}

pub(super) fn apply_inset_x(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.left = Some(value);
    patch.right = Some(value);
}

pub(super) fn apply_inset_y(patch: &mut UtilityStylePatch, value: UtilityVal) {
    patch.top = Some(value);
    patch.bottom = Some(value);
}
