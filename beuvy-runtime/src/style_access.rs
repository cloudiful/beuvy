use crate::style::{
    ButtonTheme, InteractionStatePalette, InteractionVisualTheme, PopupConfig, UiThemeConfig,
};
use crate::stylesheet::{
    StyleSheetError, default_style_sheet, parse_style_classes_with_sheet, runtime_style_sheet,
};
use crate::utility::UtilityStylePatch;
use bevy::prelude::*;
use bevy::ui::BoxShadow;
use bevy::ui::Val::Px;

fn palette(config: &crate::style::StatePaletteConfig) -> InteractionStatePalette {
    InteractionStatePalette {
        idle: config.idle.to_bevy(),
        hover: config.hover.to_bevy(),
        active: config.active.to_bevy(),
        active_hover: config.active_hover.to_bevy(),
        disabled: config.disabled.to_bevy(),
    }
}

pub fn button_theme(config: &crate::style::ButtonConfig) -> ButtonTheme {
    ButtonTheme {
        background: palette(&config.background),
        border: palette(&config.border),
        text: palette(&config.text),
    }
}

pub fn control_interaction_theme() -> InteractionVisualTheme {
    let sheet = runtime_style_sheet();
    let interaction = &sheet.config().control.interaction;
    InteractionVisualTheme {
        hover_outline: interaction.hover_outline.to_bevy(),
        focus_outline: interaction.focus_outline.to_bevy(),
        focus_hover_outline: interaction.focus_hover_outline.to_bevy(),
    }
}

pub fn ui_theme() -> &'static UiThemeConfig {
    default_style_sheet().config()
}

pub fn regular_border_width() -> f32 {
    runtime_style_sheet().config().border.regular
}

pub fn font_size_control() -> f32 {
    runtime_style_sheet().config().typography.control
}

pub fn font_size_control_compact() -> f32 {
    runtime_style_sheet().config().typography.control_compact
}

pub fn tab_border_width() -> f32 {
    runtime_style_sheet().config().border.tab
}

pub fn emphasis_border_width() -> f32 {
    runtime_style_sheet().config().border.emphasis
}

pub fn focus_outline_width() -> f32 {
    runtime_style_sheet().config().border.focus_outline
}

pub fn divider_width() -> f32 {
    runtime_style_sheet().config().border.divider
}

pub fn scrollbar_width() -> f32 {
    runtime_style_sheet().config().spacing.scrollbar_width
}

pub fn regular_border() -> UiRect {
    UiRect::all(Px(regular_border_width()))
}

pub fn tab_border() -> UiRect {
    UiRect::all(Px(tab_border_width()))
}

pub fn emphasis_border() -> UiRect {
    UiRect::all(Px(emphasis_border_width()))
}

pub fn control_radius() -> BorderRadius {
    BorderRadius::all(Px(runtime_style_sheet().config().radius.control))
}

pub fn panel_radius() -> BorderRadius {
    BorderRadius::all(Px(runtime_style_sheet().config().radius.panel))
}

pub fn pill_radius() -> BorderRadius {
    BorderRadius::all(Px(runtime_style_sheet().config().radius.pill))
}

pub fn prompt_shadow() -> BoxShadow {
    let sheet = runtime_style_sheet();
    popup_shadow(&sheet.config().panel.popup)
}

pub fn input_caret_width() -> f32 {
    runtime_style_sheet().config().control.input.caret_width
}

pub fn input_caret_color() -> Color {
    runtime_style_sheet()
        .config()
        .control
        .input
        .caret_color
        .to_bevy()
}

pub fn input_selection_color() -> Color {
    runtime_style_sheet()
        .config()
        .control
        .input
        .selection_color
        .to_bevy()
}

pub fn panel_surface_background() -> BackgroundColor {
    BackgroundColor(
        runtime_style_sheet()
            .config()
            .panel
            .main
            .background
            .to_bevy(),
    )
}

pub fn subtle_surface_background() -> BackgroundColor {
    BackgroundColor(
        runtime_style_sheet()
            .config()
            .panel
            .subtle
            .background
            .to_bevy(),
    )
}

pub fn prompt_surface_background() -> BackgroundColor {
    BackgroundColor(
        runtime_style_sheet()
            .config()
            .panel
            .prompt
            .background
            .to_bevy(),
    )
}

pub fn text_primary_color() -> Color {
    runtime_style_sheet().config().text.primary.to_bevy()
}

pub fn text_secondary_color() -> Color {
    runtime_style_sheet().config().text.secondary.to_bevy()
}

pub fn text_disabled_color() -> Color {
    runtime_style_sheet().config().text.disabled.to_bevy()
}

pub fn text_placeholder_color() -> Color {
    runtime_style_sheet().config().text.placeholder.to_bevy()
}

pub fn checkbox_border_color() -> Color {
    runtime_style_sheet()
        .config()
        .control
        .checkbox
        .border
        .to_bevy()
}

pub fn checkbox_indicator_color() -> Color {
    runtime_style_sheet()
        .config()
        .control
        .checkbox
        .indicator
        .to_bevy()
}

pub fn tab_active_text_color() -> Color {
    runtime_style_sheet().config().text.tab_active.to_bevy()
}

pub fn ui_hover_outline_color() -> Color {
    control_interaction_theme().hover_outline
}

pub fn ui_focus_outline_color() -> Color {
    control_interaction_theme().focus_outline
}

pub fn ui_focus_outline_hover_color() -> Color {
    control_interaction_theme().focus_hover_outline
}

pub fn font_size_hint() -> f32 {
    runtime_style_sheet().config().typography.hint
}

pub fn font_size_meta() -> f32 {
    runtime_style_sheet().config().typography.meta
}

pub fn font_size_body() -> f32 {
    runtime_style_sheet().config().typography.body
}

pub fn font_size_title() -> f32 {
    runtime_style_sheet().config().typography.title
}

pub fn font_size_display() -> f32 {
    runtime_style_sheet().config().typography.display
}

pub fn resolve_color_value(raw: &str) -> Option<Color> {
    let sheet = runtime_style_sheet();
    resolve_color_value_with_config(sheet.config(), raw)
}

pub fn resolve_color_value_with_config(config: &UiThemeConfig, raw: &str) -> Option<Color> {
    crate::theme_config::resolve_theme_color_value_in(config, raw).map(|color| color.to_bevy())
}

pub fn merge_classes(default_classes: &str, extra_classes: Option<&str>) -> String {
    let extra_classes = extra_classes.unwrap_or_default().trim();
    if extra_classes.is_empty() {
        default_classes.trim().to_string()
    } else {
        format!("{} {}", default_classes.trim(), extra_classes)
    }
}

pub fn resolve_class_patch(classes: &str) -> Result<UtilityStylePatch, StyleSheetError> {
    let sheet = runtime_style_sheet();
    parse_style_classes_with_sheet(&sheet, classes)
}

pub fn font_asset_path() -> String {
    let sheet = runtime_style_sheet();
    crate::theme_config::ui_theme_asset_path(&sheet.config().font.path)
}

pub fn form_item_compact_width() -> f32 {
    runtime_style_sheet()
        .config()
        .responsive
        .form_item_compact_width
}

pub fn resolve_classes_with_fallback(
    default_classes: &str,
    extra_classes: Option<&str>,
    context: &str,
) -> UtilityStylePatch {
    let merged = merge_classes(default_classes, extra_classes);
    match resolve_class_patch(&merged) {
        Ok(patch) => patch,
        Err(error) => {
            let extra_classes = extra_classes.unwrap_or_default().trim();
            if !extra_classes.is_empty() {
                warn!(
                    "failed to resolve {context} classes `{merged}`: {}; falling back to `{default_classes}`",
                    error.reason
                );
                return resolve_class_patch_or_empty(default_classes, context);
            }

            warn!(
                "failed to resolve default {context} classes `{default_classes}`: {}; using empty patch",
                error.reason
            );
            UtilityStylePatch::default()
        }
    }
}

pub fn resolve_class_patch_or_empty(classes: &str, context: &str) -> UtilityStylePatch {
    match resolve_class_patch(classes) {
        Ok(patch) => patch,
        Err(error) => {
            warn!(
                "failed to resolve {context} classes `{classes}`: {}; using empty patch",
                error.reason
            );
            UtilityStylePatch::default()
        }
    }
}

fn popup_shadow(config: &PopupConfig) -> BoxShadow {
    BoxShadow::new(
        config.shadow_color.to_bevy(),
        Px(0.0),
        Px(0.0),
        Px(0.0),
        Px(config.shadow_blur),
    )
}
