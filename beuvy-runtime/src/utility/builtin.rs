#[path = "builtin_border_radius.rs"]
mod border_radius;
#[path = "builtin_layout.rs"]
mod layout;
#[path = "builtin_size_spacing.rs"]
mod size_spacing;
#[path = "builtin_text.rs"]
mod text;
#[path = "builtin_variant.rs"]
mod variant;
#[path = "builtin_visual.rs"]
mod visual;

use super::types::{ParseUtilityError, UtilityStylePatch};
use crate::theme_config::UiThemeConfig;

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
        if let Some((utility_variant, inner)) = variant::parse_variant_token(token)? {
            visual::apply_visual_utility_token(
                config,
                inner,
                variant::state_patch_mut(&mut patch, utility_variant),
                true,
                token,
            )?;
            continue;
        }

        apply_utility_token(config, token, &mut patch)?;
    }

    Ok(patch)
}

fn apply_utility_token(
    config: &UiThemeConfig,
    token: &str,
    patch: &mut UtilityStylePatch,
) -> Result<(), ParseUtilityError> {
    if let Some(size) = text::parse_text_size_token(config, token)? {
        patch.text_size = Some(size);
        return Ok(());
    }
    if border_radius::apply_border_radius_utility_token(config, token, patch)? {
        return Ok(());
    }
    if visual::apply_visual_utility_token(config, token, &mut patch.visual, false, token)? {
        return Ok(());
    }
    if layout::apply_layout_utility_token(token, patch) {
        return Ok(());
    }
    if size_spacing::apply_size_spacing_utility_token(config, token, patch)? {
        return Ok(());
    }

    Err(ParseUtilityError::new(token, "unknown utility class"))
}
