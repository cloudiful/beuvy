use super::config_types::UiThemeConfig;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl ThemeColor {
    pub const fn hex(value: &str) -> Self {
        let bytes = value.as_bytes();
        let start = if !bytes.is_empty() && bytes[0] == b'#' {
            1
        } else {
            0
        };
        let len = bytes.len() - start;
        let alpha = if len == 8 {
            decode_hex(bytes[start + 6], bytes[start + 7])
        } else {
            255
        };

        Self {
            red: decode_hex(bytes[start], bytes[start + 1]),
            green: decode_hex(bytes[start + 2], bytes[start + 3]),
            blue: decode_hex(bytes[start + 4], bytes[start + 5]),
            alpha,
        }
    }

    pub fn to_bevy(self) -> Color {
        Color::srgba_u8(self.red, self.green, self.blue, self.alpha)
    }
}

impl Default for ThemeColor {
    fn default() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }
    }
}

#[allow(dead_code)]
pub fn resolve_theme_color_value(raw: &str) -> Option<ThemeColor> {
    resolve_theme_color_value_in(super::ui_theme_config(), raw)
}

#[allow(dead_code)]
pub fn resolve_theme_numeric_value(raw: &str) -> Option<f32> {
    resolve_theme_numeric_value_in(super::ui_theme_config(), raw)
}

pub fn resolve_theme_color_value_in(config: &UiThemeConfig, raw: &str) -> Option<ThemeColor> {
    let value = raw.trim();
    if let Some(name) = parse_theme_color_var(value) {
        return theme_color_variable(config, name);
    }
    parse_theme_color(value).ok()
}

pub fn resolve_theme_numeric_value_in(config: &UiThemeConfig, raw: &str) -> Option<f32> {
    let token = raw
        .trim()
        .strip_prefix("var(")
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)?;
    match token {
        "--text-hint" => Some(config.typography.hint),
        "--text-meta" => Some(config.typography.meta),
        "--text-body" => Some(config.typography.body),
        "--text-control" => Some(config.typography.control),
        "--text-control-compact" => Some(config.typography.control_compact),
        "--text-title" => Some(config.typography.title),
        "--text-display" => Some(config.typography.display),
        "--spacing-scrollbar-width" => Some(config.spacing.scrollbar_width),
        "--spacing-panel-padding" => Some(config.spacing.panel_padding),
        "--spacing-panel-gap" => Some(config.spacing.panel_gap),
        "--spacing-section-gap" => Some(config.spacing.section_gap),
        "--spacing-section-grid-gap" => Some(config.spacing.section_grid_gap),
        "--spacing-content-padding" => Some(config.spacing.content_padding),
        "--spacing-form-padding-x" => Some(config.spacing.form_padding_x),
        "--spacing-form-padding-y" => Some(config.spacing.form_padding_y),
        "--spacing-form-gap" => Some(config.spacing.form_gap),
        "--spacing-button-padding-x" => Some(config.spacing.button_padding_x),
        "--spacing-button-padding-y" => Some(config.spacing.button_padding_y),
        "--spacing-button-compact-padding-x" => Some(config.spacing.button_compact_padding_x),
        "--spacing-button-compact-padding-y" => Some(config.spacing.button_compact_padding_y),
        "--border-regular" => Some(config.border.regular),
        "--border-tab" => Some(config.border.tab),
        "--border-emphasis" => Some(config.border.emphasis),
        "--border-focus-outline" => Some(config.border.focus_outline),
        "--border-divider" => Some(config.border.divider),
        "--radius-ui" => Some(config.radius.ui),
        "--radius-panel" => Some(config.radius.panel),
        "--radius-control" => Some(config.radius.control),
        "--radius-pill" => Some(config.radius.pill),
        _ => None,
    }
}

pub(super) fn parse_theme_color(raw: &str) -> Result<ThemeColor, String> {
    let value = raw.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 && hex.len() != 8 {
        return Err(format!("expected color '{}' to use 6 or 8 hex digits", raw));
    }

    let red = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|error| format!("invalid red channel in '{raw}': {error}"))?;
    let green = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|error| format!("invalid green channel in '{raw}': {error}"))?;
    let blue = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|error| format!("invalid blue channel in '{raw}': {error}"))?;
    let alpha = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16)
            .map_err(|error| format!("invalid alpha channel in '{raw}': {error}"))?
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

fn parse_theme_color_var(raw: &str) -> Option<&str> {
    raw.strip_prefix("var(")
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
        .or_else(|| raw.strip_prefix("--color-").map(|_| raw))
}

fn theme_color_variable(config: &UiThemeConfig, name: &str) -> Option<ThemeColor> {
    let token = name.strip_prefix("--color-")?;
    if let Some(color) = config.tokens.color.get(token) {
        return Some(*color);
    }

    match token {
        "primary" | "text-primary" => Some(config.text.primary),
        "secondary" | "text-secondary" => Some(config.text.secondary),
        "disabled" | "text-disabled" => Some(config.text.disabled),
        "muted" | "text-muted" => Some(config.text.muted),
        "placeholder" | "text-placeholder" => Some(config.text.placeholder),
        "subtle-glyph" => Some(config.text.subtle_glyph),
        "selection-indicator-glyph" => Some(config.text.selection_indicator_glyph),
        "option-enabled" | "settings-option-enabled" => Some(config.text.option_enabled),
        "option-disabled" | "settings-option-disabled" => Some(config.text.option_disabled),
        "tab-active" => Some(config.text.tab_active),
        "panel-app-bg" => Some(config.panel.app.background),
        "panel-app-border" => Some(config.panel.app.border),
        "panel-main-bg" => Some(config.panel.main.background),
        "panel-main-border" => Some(config.panel.main.border),
        "panel-subtle-bg" => Some(config.panel.subtle.background),
        "panel-subtle-border" => Some(config.panel.subtle.border),
        "panel-prompt-bg" => Some(config.panel.prompt.background),
        "panel-prompt-border" => Some(config.panel.prompt.border),
        "panel-popup-bg" => Some(config.panel.popup.background),
        "panel-popup-border" => Some(config.panel.popup.border),
        "panel-detail-popup-bg" | "panel-inventory-popup-bg" => {
            Some(config.panel.detail_popup.background)
        }
        "panel-detail-popup-border" | "panel-inventory-popup-border" => {
            Some(config.panel.detail_popup.border)
        }
        "panel-media-preview-bg" | "panel-inventory-preview-bg" => {
            Some(config.panel.media_preview.background)
        }
        "panel-media-preview-border" | "panel-inventory-preview-border" => {
            Some(config.panel.media_preview.border)
        }
        "panel-media-preview-fallback-bg" | "panel-inventory-preview-fallback-bg" => {
            Some(config.panel.media_preview_fallback.background)
        }
        "panel-media-preview-fallback-border" | "panel-inventory-preview-fallback-border" => {
            Some(config.panel.media_preview_fallback.border)
        }
        "panel-count-badge-bg" | "panel-inventory-count-badge-bg" => {
            Some(config.panel.count_badge.background)
        }
        "panel-count-badge-border" | "panel-inventory-count-badge-border" => {
            Some(config.panel.count_badge.border)
        }
        "panel-list-item-idle-bg" | "panel-instant-message-idle-bg" => {
            Some(config.panel.list_item_idle.background)
        }
        "panel-list-item-idle-border" | "panel-instant-message-idle-border" => {
            Some(config.panel.list_item_idle.border)
        }
        "panel-list-item-selected-bg" | "panel-instant-message-selected-bg" => {
            Some(config.panel.list_item_selected.background)
        }
        "panel-list-item-selected-border" | "panel-instant-message-selected-border" => {
            Some(config.panel.list_item_selected.border)
        }
        "tile-label-bg" | "inventory-tile-label-bg" => {
            Some(config.panel.tile.tile_label_background)
        }
        "control-hover-outline" => Some(config.control.interaction.hover_outline),
        "control-focus-outline" => Some(config.control.interaction.focus_outline),
        "control-focus-hover-outline" => Some(config.control.interaction.focus_hover_outline),
        "control-active-bg" => Some(config.control.interaction.active_background),
        "control-active-border" => Some(config.control.interaction.active_border),
        "select-chip-bg" => Some(config.control.select.chip.background),
        "select-chip-border" => Some(config.control.select.chip.border),
        "select-indicator-bg" => Some(config.control.select.indicator.background),
        "select-indicator-border" => Some(config.control.select.indicator.border),
        "select-panel-bg" => Some(config.control.select.panel.background),
        "select-panel-border" => Some(config.control.select.panel.border),
        "select-glyph" => Some(config.control.select.glyph_color),
        "select-indicator-glyph" => Some(config.control.select.indicator_glyph_color),
        "slider-field-bg" => Some(config.control.slider.field.background),
        "slider-field-border" => Some(config.control.slider.field.border),
        "slider-field-active-bg" => Some(config.control.slider.field.active_background),
        "slider-field-active-border" => Some(config.control.slider.field.active_border),
        "slider-track-bg" => Some(config.control.slider.track.background),
        "slider-track-border" => Some(config.control.slider.track.border),
        "slider-fill-bg" => Some(config.control.slider.fill_background),
        "slider-thumb-bg" => Some(config.control.slider.thumb_background),
        "slider-thumb-border" => Some(config.control.slider.thumb_border),
        "button-bg" => Some(config.buttons.background.idle),
        "button-bg-hover" => Some(config.buttons.background.hover),
        "button-bg-active" => Some(config.buttons.background.active),
        "button-bg-active-hover" => Some(config.buttons.background.active_hover),
        "button-bg-disabled" => Some(config.buttons.background.disabled),
        "button-text" => Some(config.buttons.text.idle),
        "button-text-hover" => Some(config.buttons.text.hover),
        "button-text-active" => Some(config.buttons.text.active),
        "button-text-active-hover" => Some(config.buttons.text.active_hover),
        "button-text-disabled" => Some(config.buttons.text.disabled),
        _ => None,
    }
}

const fn decode_hex(high: u8, low: u8) -> u8 {
    (decode_nibble(high) << 4) | decode_nibble(low)
}

const fn decode_nibble(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => 0,
    }
}
