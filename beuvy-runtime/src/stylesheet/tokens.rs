use super::model::StyleSheetError;
use crate::style::{ThemeColor, UiThemeConfig};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub(super) struct ParsedTokenSets {
    pub(super) color: HashSet<String>,
    pub(super) text: HashSet<String>,
    pub(super) radius: HashSet<String>,
}

pub(super) fn parse_theme_block(
    config: &mut UiThemeConfig,
    tokens: &mut ParsedTokenSets,
    body: &str,
) -> Result<(), StyleSheetError> {
    for declaration in body.split(';') {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let Some((name, value)) = declaration.split_once(':') else {
            return Err(StyleSheetError::new(format!(
                "invalid @theme declaration `{declaration}`"
            )));
        };
        apply_theme_variable(config, tokens, name.trim(), value.trim())?;
    }
    Ok(())
}

fn apply_theme_variable(
    config: &mut UiThemeConfig,
    tokens: &mut ParsedTokenSets,
    name: &str,
    value: &str,
) -> Result<(), StyleSheetError> {
    if let Some(token) = name.strip_prefix("--color-") {
        let color = parse_theme_color(value)?;
        tokens.color.insert(token.to_string());
        config.tokens.color.insert(token.to_string(), color);
        match token {
            "primary" => config.text.primary = color,
            "secondary" => config.text.secondary = color,
            "disabled" => config.text.disabled = color,
            "muted" => config.text.muted = color,
            "placeholder" => config.text.placeholder = color,
            "subtle-glyph" => config.text.subtle_glyph = color,
            "selection-indicator-glyph" => config.text.selection_indicator_glyph = color,
            "settings-option-enabled" | "option-enabled" => config.text.option_enabled = color,
            "settings-option-disabled" | "option-disabled" => config.text.option_disabled = color,
            "tab-active" => config.text.tab_active = color,
            "panel-app-bg" => config.panel.app.background = color,
            "panel-app-border" => config.panel.app.border = color,
            "panel-main-bg" => config.panel.main.background = color,
            "panel-main-border" => config.panel.main.border = color,
            "panel-subtle-bg" => config.panel.subtle.background = color,
            "panel-subtle-border" => config.panel.subtle.border = color,
            "panel-prompt-bg" => config.panel.prompt.background = color,
            "panel-prompt-border" => config.panel.prompt.border = color,
            "panel-popup-bg" => config.panel.popup.background = color,
            "panel-popup-border" => config.panel.popup.border = color,
            "panel-popup-shadow" => config.panel.popup.shadow_color = color,
            "panel-inventory-popup-bg" | "panel-detail-popup-bg" => {
                config.panel.detail_popup.background = color
            }
            "panel-inventory-popup-border" | "panel-detail-popup-border" => {
                config.panel.detail_popup.border = color
            }
            "panel-inventory-popup-shadow" | "panel-detail-popup-shadow" => {
                config.panel.detail_popup.shadow_color = color
            }
            "panel-inventory-preview-bg" | "panel-media-preview-bg" => {
                config.panel.media_preview.background = color
            }
            "panel-inventory-preview-border" | "panel-media-preview-border" => {
                config.panel.media_preview.border = color
            }
            "panel-inventory-preview-fallback-bg" | "panel-media-preview-fallback-bg" => {
                config.panel.media_preview_fallback.background = color
            }
            "panel-inventory-preview-fallback-border" | "panel-media-preview-fallback-border" => {
                config.panel.media_preview_fallback.border = color
            }
            "panel-inventory-count-badge-bg" | "panel-count-badge-bg" => {
                config.panel.count_badge.background = color
            }
            "panel-inventory-count-badge-border" | "panel-count-badge-border" => {
                config.panel.count_badge.border = color
            }
            "panel-instant-message-idle-bg" | "panel-list-item-idle-bg" => {
                config.panel.list_item_idle.background = color
            }
            "panel-instant-message-idle-border" | "panel-list-item-idle-border" => {
                config.panel.list_item_idle.border = color
            }
            "panel-instant-message-selected-bg" | "panel-list-item-selected-bg" => {
                config.panel.list_item_selected.background = color
            }
            "panel-instant-message-selected-border" | "panel-list-item-selected-border" => {
                config.panel.list_item_selected.border = color
            }
            "inventory-tile-label-bg" | "tile-label-bg" => {
                config.panel.tile.tile_label_background = color
            }
            "control-hover-outline" => config.control.interaction.hover_outline = color,
            "control-focus-outline" => config.control.interaction.focus_outline = color,
            "control-focus-hover-outline" => config.control.interaction.focus_hover_outline = color,
            "control-active-bg" => config.control.interaction.active_background = color,
            "control-active-border" => config.control.interaction.active_border = color,
            "checkbox-border" => config.control.checkbox.border = color,
            "checkbox-indicator" => config.control.checkbox.indicator = color,
            "select-chip-bg" => config.control.select.chip.background = color,
            "select-chip-border" => config.control.select.chip.border = color,
            "select-indicator-bg" => config.control.select.indicator.background = color,
            "select-indicator-border" => config.control.select.indicator.border = color,
            "select-panel-bg" => config.control.select.panel.background = color,
            "select-panel-border" => config.control.select.panel.border = color,
            "select-panel-shadow" => config.control.select.panel.shadow_color = color,
            "select-glyph" => config.control.select.glyph_color = color,
            "select-indicator-glyph" => config.control.select.indicator_glyph_color = color,
            "slider-field-bg" => config.control.slider.field.background = color,
            "slider-field-border" => config.control.slider.field.border = color,
            "slider-field-active-bg" => config.control.slider.field.active_background = color,
            "slider-field-active-border" => config.control.slider.field.active_border = color,
            "slider-track-bg" => config.control.slider.track.background = color,
            "slider-track-border" => config.control.slider.track.border = color,
            "slider-fill-bg" => config.control.slider.fill_background = color,
            "slider-thumb-bg" => config.control.slider.thumb_background = color,
            "slider-thumb-border" => config.control.slider.thumb_border = color,
            "button-bg" => config.buttons.background.idle = color,
            "button-bg-hover" => config.buttons.background.hover = color,
            "button-bg-active" => config.buttons.background.active = color,
            "button-bg-active-hover" => config.buttons.background.active_hover = color,
            "button-bg-disabled" => config.buttons.background.disabled = color,
            "button-border" => config.buttons.border.idle = color,
            "button-border-hover" => config.buttons.border.hover = color,
            "button-border-active" => config.buttons.border.active = color,
            "button-border-active-hover" => config.buttons.border.active_hover = color,
            "button-border-disabled" => config.buttons.border.disabled = color,
            "button-text" => config.buttons.text.idle = color,
            "button-text-hover" => config.buttons.text.hover = color,
            "button-text-active" => config.buttons.text.active = color,
            "button-text-active-hover" => config.buttons.text.active_hover = color,
            "button-text-disabled" => config.buttons.text.disabled = color,
            _ => {}
        }
        return Ok(());
    }

    if let Some(token) = name.strip_prefix("--text-") {
        let numeric = parse_px_value(name, value)?;
        tokens.text.insert(token.to_string());
        match token {
            "hint" => config.typography.hint = numeric,
            "meta" => config.typography.meta = numeric,
            "body" => config.typography.body = numeric,
            "control" => config.typography.control = numeric,
            "control-compact" => config.typography.control_compact = numeric,
            "title" => config.typography.title = numeric,
            "display" => config.typography.display = numeric,
            _ => {}
        }
        return Ok(());
    }

    if let Some(token) = name.strip_prefix("--radius-") {
        let numeric = parse_px_value(name, value)?;
        tokens.radius.insert(token.to_string());
        match token {
            "ui" => config.radius.ui = numeric,
            "panel" => config.radius.panel = numeric,
            "control" => config.radius.control = numeric,
            "pill" => config.radius.pill = numeric,
            _ => {}
        }
        return Ok(());
    }

    match name {
        "--font-ui" => config.font.path = unquote(value).to_string(),
        "--border-regular" => config.border.regular = parse_px_value(name, value)?,
        "--border-tab" => config.border.tab = parse_px_value(name, value)?,
        "--border-emphasis" => config.border.emphasis = parse_px_value(name, value)?,
        "--border-focus-outline" => config.border.focus_outline = parse_px_value(name, value)?,
        "--border-divider" => config.border.divider = parse_px_value(name, value)?,
        "--spacing-scrollbar-width" => {
            config.spacing.scrollbar_width = parse_px_value(name, value)?
        }
        "--spacing-panel-padding" => config.spacing.panel_padding = parse_px_value(name, value)?,
        "--spacing-panel-gap" => config.spacing.panel_gap = parse_px_value(name, value)?,
        "--spacing-section-gap" => config.spacing.section_gap = parse_px_value(name, value)?,
        "--spacing-section-grid-gap" => {
            config.spacing.section_grid_gap = parse_px_value(name, value)?
        }
        "--spacing-content-padding" => {
            config.spacing.content_padding = parse_px_value(name, value)?
        }
        "--spacing-form-padding-x" => config.spacing.form_padding_x = parse_px_value(name, value)?,
        "--spacing-form-padding-y" => config.spacing.form_padding_y = parse_px_value(name, value)?,
        "--spacing-form-gap" => config.spacing.form_gap = parse_px_value(name, value)?,
        "--spacing-button-padding-x" => {
            config.spacing.button_padding_x = parse_px_value(name, value)?
        }
        "--spacing-button-padding-y" => {
            config.spacing.button_padding_y = parse_px_value(name, value)?
        }
        "--spacing-button-compact-padding-x" => {
            config.spacing.button_compact_padding_x = parse_px_value(name, value)?
        }
        "--spacing-button-compact-padding-y" => {
            config.spacing.button_compact_padding_y = parse_px_value(name, value)?
        }
        "--breakpoint-form-item-compact" => {
            config.responsive.form_item_compact_width = parse_px_value(name, value)?
        }
        "--breakpoint-setting-shell-compact" => {
            config.responsive.panel_shell_compact_width = parse_px_value(name, value)?
        }
        "--breakpoint-setting-grid-single-column" => {
            config.responsive.panel_grid_single_column_width = parse_px_value(name, value)?
        }
        "--shadow-panel-popup-blur" => {
            config.panel.popup.shadow_blur = parse_px_value(name, value)?
        }
        "--shadow-panel-inventory-popup-blur" | "--shadow-panel-detail-popup-blur" => {
            config.panel.detail_popup.shadow_blur = parse_px_value(name, value)?
        }
        "--shadow-select-panel-blur" => {
            config.control.select.panel.shadow_blur = parse_px_value(name, value)?
        }
        _ => {}
    }

    Ok(())
}

fn parse_theme_color(raw: &str) -> Result<ThemeColor, StyleSheetError> {
    let value = raw.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 && hex.len() != 8 {
        return Err(StyleSheetError::new(format!(
            "expected color `{raw}` to use 6 or 8 hex digits"
        )));
    }

    let red = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| StyleSheetError::new(format!("invalid red channel in `{raw}`")))?;
    let green = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| StyleSheetError::new(format!("invalid green channel in `{raw}`")))?;
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| StyleSheetError::new(format!("invalid blue channel in `{raw}`")))?;
    let alpha = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16)
            .map_err(|_| StyleSheetError::new(format!("invalid alpha channel in `{raw}`")))?
    } else {
        255
    };

    Ok(ThemeColor {
        red,
        green,
        blue,
        alpha,
    })
}

fn parse_px_value(name: &str, raw: &str) -> Result<f32, StyleSheetError> {
    let raw = raw.trim();
    let value = raw.strip_suffix("px").unwrap_or(raw);
    value
        .parse::<f32>()
        .map_err(|_| StyleSheetError::new(format!("{name} expected px or unitless number")))
}

fn unquote(raw: &str) -> &str {
    raw.trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            raw.trim()
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(raw.trim())
}
