#[path = "style_access.rs"]
mod access;
#[path = "style_apply.rs"]
mod apply;
#[path = "style_state_patch.rs"]
mod state_patch;

use bevy::prelude::*;

pub use crate::theme_config::{
    BorderConfig, ButtonConfig, CheckboxConfig, ControlConfig, FieldConfig, FontConfig,
    InputConfig, InteractionConfig, PanelConfig, PopupConfig, RadiusConfig, ResponsiveConfig,
    SelectConfig, SliderConfig, SpacingConfig, StatePaletteConfig, SurfaceConfig as UiSurfaceConfig,
    TextConfig, ThemeColor, ThemeTokensConfig, TileConfig, TypographyConfig, UiThemeConfig,
};
pub use access::{
    button_theme, control_interaction_theme, control_radius, divider_width, emphasis_border,
    emphasis_border_width, focus_outline_width, font_asset_path, font_size_body,
    font_size_control, font_size_control_compact, font_size_display, font_size_hint,
    font_size_meta, font_size_title, form_item_compact_width, input_caret_color,
    input_caret_width, input_selection_color, merge_classes, panel_radius,
    panel_surface_background, pill_radius, prompt_shadow, prompt_surface_background,
    regular_border, regular_border_width, resolve_class_patch, resolve_class_patch_or_empty,
    resolve_classes_with_fallback, resolve_color_value, resolve_color_value_with_config,
    scrollbar_width, subtle_surface_background, tab_active_text_color, tab_border,
    tab_border_width, text_disabled_color, text_placeholder_color, text_primary_color,
    text_secondary_color, ui_focus_outline_color, ui_focus_outline_hover_color,
    ui_hover_outline_color, ui_theme,
};
pub use apply::apply_utility_patch;
pub use state_patch::{root_visual_styles_from_patch, text_visual_styles_from_patch, token_border_style};

#[derive(Debug, Clone, Copy)]
pub struct InteractionVisualTheme {
    pub hover_outline: Color,
    pub focus_outline: Color,
    pub focus_hover_outline: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct InteractionStatePalette {
    pub idle: Color,
    pub hover: Color,
    pub active: Color,
    pub active_hover: Color,
    pub disabled: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub background: InteractionStatePalette,
    pub border: InteractionStatePalette,
    pub text: InteractionStatePalette,
}
