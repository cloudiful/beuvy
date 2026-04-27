use super::builtin::parse_builtin_utility_tokens;
use super::types::{ParseUtilityError, UtilityStylePatch};
use crate::stylesheet::{default_style_sheet, parse_style_classes_with_sheet};
use crate::theme_config::UiThemeConfig;

/// Parses a Tailwind-like utility class string into a style patch using the
/// currently active UI theme config.
///
/// This parser intentionally supports a curated subset of Tailwind focused on
/// common Bevy UI layout and visual utilities. Unknown tokens fail fast rather
/// than being ignored so DSL assets cannot silently drift out of support.
pub fn parse_utility_classes(input: &str) -> Result<UtilityStylePatch, ParseUtilityError> {
    parse_style_classes_with_sheet(default_style_sheet(), input)
        .map_err(|error| ParseUtilityError::new(input, error.reason))
}

#[doc(hidden)]
pub fn parse_utility_classes_with_config(
    config: &UiThemeConfig,
    input: &str,
) -> Result<UtilityStylePatch, ParseUtilityError> {
    let tokens = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    parse_builtin_utility_tokens(config, &tokens)
}
