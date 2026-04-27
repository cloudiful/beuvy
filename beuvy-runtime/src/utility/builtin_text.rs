use super::size_spacing::parse_numeric_value;
use crate::utility::{ParseUtilityError, UtilityVal};
use crate::theme_config::{UiThemeConfig, resolve_theme_numeric_value_in};

pub(super) fn parse_text_size_token(
    config: &UiThemeConfig,
    token: &str,
) -> Result<Option<f32>, ParseUtilityError> {
    let Some(raw) = token.strip_prefix("text-") else {
        return Ok(None);
    };

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
                return Ok(Some(parse_text_size_value(config, token, raw)?));
            }
            Ok(None)
        }
    }
}

pub(super) fn parse_text_size_token_value(
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
                return Ok(Some(parse_text_size_value(config, token, raw)?));
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
    match parse_numeric_value(config, token, super::size_spacing::unwrap_arbitrary_value(raw))? {
        UtilityVal::Px(value) => Ok(value),
        UtilityVal::Percent(_) | UtilityVal::Vw(_) | UtilityVal::Vh(_) | UtilityVal::Auto => {
            Err(ParseUtilityError::new(token, "invalid text size value"))
        }
    }
}

fn theme_text_size(config: &UiThemeConfig, name: &str) -> f32 {
    resolve_theme_numeric_value_in(config, &format!("var(--text-{name})")).unwrap_or(0.0)
}
