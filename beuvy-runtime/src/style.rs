use crate::interaction_style::UiStateVisualStyles;
use crate::stylesheet::{
    StyleSheetError, default_style_sheet, parse_style_classes_with_sheet, runtime_style_sheet,
};
use crate::utility::{
    UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay, UtilityFlexDirection,
    UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis, UtilityPositionType, UtilityRect,
    UtilityStylePatch, UtilityVal, UtilityVisualStylePatch,
};
use bevy::prelude::*;
use bevy::ui::BoxShadow;
use bevy::ui::OverflowAxis;
use bevy::ui::Val::Px;

pub use crate::theme_config::{
    BorderConfig, ButtonConfig, CheckboxConfig, ControlConfig, FieldConfig, FontConfig,
    InteractionConfig, PanelConfig, PopupConfig, RadiusConfig, ResponsiveConfig, SelectConfig,
    SliderConfig, SpacingConfig, StatePaletteConfig, SurfaceConfig as UiSurfaceConfig, TextConfig,
    ThemeColor, ThemeTokensConfig, TileConfig, TypographyConfig, UiThemeConfig,
};

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

fn palette(config: &StatePaletteConfig) -> InteractionStatePalette {
    InteractionStatePalette {
        idle: config.idle.to_bevy(),
        hover: config.hover.to_bevy(),
        active: config.active.to_bevy(),
        active_hover: config.active_hover.to_bevy(),
        disabled: config.disabled.to_bevy(),
    }
}

pub fn button_theme(config: &ButtonConfig) -> ButtonTheme {
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

pub fn token_border_style(token: &str) -> UiStateVisualStyles {
    UiStateVisualStyles {
        base: UtilityVisualStylePatch {
            border_color: Some(format!("var(--color-{token})")),
            ..default()
        },
        ..default()
    }
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

pub fn apply_utility_patch(node: &mut Node, patch: &UtilityStylePatch) {
    if let Some(value) = patch.width {
        node.width = utility_val_to_val(value);
    }
    if let Some(value) = patch.height {
        node.height = utility_val_to_val(value);
    }
    if let Some(value) = patch.min_width {
        node.min_width = utility_val_to_val(value);
    }
    if let Some(value) = patch.min_height {
        node.min_height = utility_val_to_val(value);
    }
    if let Some(value) = patch.max_width {
        node.max_width = utility_val_to_val(value);
    }
    if let Some(value) = patch.max_height {
        node.max_height = utility_val_to_val(value);
    }
    if let Some(value) = patch.flex_direction {
        node.flex_direction = match value {
            UtilityFlexDirection::Row => FlexDirection::Row,
            UtilityFlexDirection::Column => FlexDirection::Column,
            UtilityFlexDirection::RowReverse => FlexDirection::RowReverse,
            UtilityFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
        };
    }
    if let Some(value) = patch.justify_content {
        node.justify_content = match value {
            UtilityJustifyContent::FlexStart => JustifyContent::FlexStart,
            UtilityJustifyContent::FlexEnd => JustifyContent::FlexEnd,
            UtilityJustifyContent::Center => JustifyContent::Center,
            UtilityJustifyContent::SpaceBetween => JustifyContent::SpaceBetween,
            UtilityJustifyContent::SpaceAround => JustifyContent::SpaceAround,
            UtilityJustifyContent::SpaceEvenly => JustifyContent::SpaceEvenly,
        };
    }
    if let Some(value) = patch.align_items {
        node.align_items = match value {
            UtilityAlignItems::FlexStart => AlignItems::FlexStart,
            UtilityAlignItems::FlexEnd => AlignItems::FlexEnd,
            UtilityAlignItems::Center => AlignItems::Center,
            UtilityAlignItems::Baseline => AlignItems::Baseline,
            UtilityAlignItems::Stretch => AlignItems::Stretch,
        };
    }
    if let Some(value) = patch.align_content {
        node.align_content = match value {
            UtilityAlignContent::FlexStart => AlignContent::FlexStart,
            UtilityAlignContent::FlexEnd => AlignContent::FlexEnd,
            UtilityAlignContent::Center => AlignContent::Center,
            UtilityAlignContent::Stretch => AlignContent::Stretch,
            UtilityAlignContent::SpaceBetween => AlignContent::SpaceBetween,
            UtilityAlignContent::SpaceAround => AlignContent::SpaceAround,
            UtilityAlignContent::SpaceEvenly => AlignContent::SpaceEvenly,
        };
    }
    if let Some(value) = patch.align_self {
        node.align_self = match value {
            UtilityAlignSelf::Auto => AlignSelf::Auto,
            UtilityAlignSelf::FlexStart => AlignSelf::FlexStart,
            UtilityAlignSelf::FlexEnd => AlignSelf::FlexEnd,
            UtilityAlignSelf::Center => AlignSelf::Center,
            UtilityAlignSelf::Baseline => AlignSelf::Baseline,
            UtilityAlignSelf::Stretch => AlignSelf::Stretch,
        };
    }
    if let Some(value) = patch.flex_wrap {
        node.flex_wrap = match value {
            UtilityFlexWrap::NoWrap => FlexWrap::NoWrap,
            UtilityFlexWrap::Wrap => FlexWrap::Wrap,
            UtilityFlexWrap::WrapReverse => FlexWrap::WrapReverse,
        };
    }
    if let Some(value) = patch.flex_grow {
        node.flex_grow = value;
    }
    if let Some(value) = patch.flex_shrink {
        node.flex_shrink = value;
    }
    if let Some(value) = patch.flex_basis {
        node.flex_basis = utility_val_to_val(value);
    }
    if let Some(value) = patch.row_gap {
        node.row_gap = utility_val_to_val(value);
    }
    if let Some(value) = patch.column_gap {
        node.column_gap = utility_val_to_val(value);
    }
    if let Some(value) = &patch.padding {
        node.padding = utility_padding_rect_to_rect(value);
    }
    if let Some(value) = &patch.margin {
        node.margin = utility_margin_rect_to_rect(value);
    }
    if let Some(value) = &patch.border {
        node.border = utility_border_rect_to_rect(value);
    }
    if let Some(value) = patch.border_radius {
        node.border_radius = BorderRadius::all(utility_val_to_val(value));
    }
    if patch.overflow_x.is_some() || patch.overflow_y.is_some() {
        node.overflow = Overflow {
            x: patch
                .overflow_x
                .map(utility_overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
            y: patch
                .overflow_y
                .map(utility_overflow_axis)
                .unwrap_or(OverflowAxis::Visible),
        };
    }
    if let Some(value) = patch.display {
        node.display = match value {
            UtilityDisplay::Flex => Display::Flex,
            UtilityDisplay::Grid => Display::Grid,
            UtilityDisplay::None => Display::None,
            UtilityDisplay::Block => Display::Block,
        };
    }
    if let Some(value) = patch.position_type {
        node.position_type = match value {
            UtilityPositionType::Relative => PositionType::Relative,
            UtilityPositionType::Absolute => PositionType::Absolute,
        };
    }
    if let Some(value) = patch.left {
        node.left = utility_val_to_val(value);
    }
    if let Some(value) = patch.right {
        node.right = utility_val_to_val(value);
    }
    if let Some(value) = patch.top {
        node.top = utility_val_to_val(value);
    }
    if let Some(value) = patch.bottom {
        node.bottom = utility_val_to_val(value);
    }
}

pub fn root_visual_styles_from_patch(patch: &UtilityStylePatch) -> Option<UiStateVisualStyles> {
    let styles = UiStateVisualStyles {
        base: visual_patch_without_text(&patch.visual),
        hover: visual_patch_without_text(&patch.hover),
        active: visual_patch_without_text(&patch.active),
        focus: visual_patch_without_text(&patch.focus),
        disabled: visual_patch_without_text(&patch.disabled),
    };
    (!styles.is_empty()).then_some(styles)
}

pub fn text_visual_styles_from_patch(patch: &UtilityStylePatch) -> Option<UiStateVisualStyles> {
    let styles = UiStateVisualStyles {
        base: text_visual_patch_only(&patch.visual),
        hover: text_visual_patch_only(&patch.hover),
        active: text_visual_patch_only(&patch.active),
        focus: text_visual_patch_only(&patch.focus),
        disabled: text_visual_patch_only(&patch.disabled),
    };
    (!styles.is_empty()).then_some(styles)
}

fn visual_patch_without_text(patch: &UtilityVisualStylePatch) -> UtilityVisualStylePatch {
    UtilityVisualStylePatch {
        background_color: patch.background_color.clone(),
        border_color: patch.border_color.clone(),
        outline_width: patch.outline_width,
        outline_color: patch.outline_color.clone(),
        opacity: patch.opacity,
        transition_property: patch.transition_property,
        transition_duration_ms: patch.transition_duration_ms,
        transition_timing: patch.transition_timing,
        ..default()
    }
}

fn text_visual_patch_only(patch: &UtilityVisualStylePatch) -> UtilityVisualStylePatch {
    UtilityVisualStylePatch {
        text_color: patch.text_color.clone(),
        opacity: patch.opacity,
        transition_property: patch.transition_property,
        transition_duration_ms: patch.transition_duration_ms,
        transition_timing: patch.transition_timing,
        ..default()
    }
}

fn utility_val_to_val(value: UtilityVal) -> Val {
    match value {
        UtilityVal::Auto => Val::Auto,
        UtilityVal::Px(value) => Val::Px(value),
        UtilityVal::Percent(value) => Val::Percent(value),
        UtilityVal::Vw(value) => Val::Vw(value),
        UtilityVal::Vh(value) => Val::Vh(value),
    }
}

fn utility_margin_rect_to_rect(value: &UtilityRect) -> UiRect {
    UiRect {
        left: value.left.map(utility_val_to_val).unwrap_or(Val::ZERO),
        right: value.right.map(utility_val_to_val).unwrap_or(Val::ZERO),
        top: value.top.map(utility_val_to_val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(utility_val_to_val).unwrap_or(Val::ZERO),
    }
}

fn utility_padding_rect_to_rect(value: &UtilityRect) -> UiRect {
    utility_zero_default_rect(value)
}

fn utility_border_rect_to_rect(value: &UtilityRect) -> UiRect {
    utility_zero_default_rect(value)
}

fn utility_zero_default_rect(value: &UtilityRect) -> UiRect {
    UiRect {
        left: value.left.map(utility_val_to_val).unwrap_or(Val::ZERO),
        right: value.right.map(utility_val_to_val).unwrap_or(Val::ZERO),
        top: value.top.map(utility_val_to_val).unwrap_or(Val::ZERO),
        bottom: value.bottom.map(utility_val_to_val).unwrap_or(Val::ZERO),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margin_rect_defaults_missing_edges_to_zero() {
        let rect = utility_margin_rect_to_rect(&UtilityRect {
            left: Some(UtilityVal::Auto),
            right: Some(UtilityVal::Auto),
            ..default()
        });

        assert_eq!(rect.left, Val::Auto);
        assert_eq!(rect.right, Val::Auto);
        assert_eq!(rect.top, Val::ZERO);
        assert_eq!(rect.bottom, Val::ZERO);
    }

    #[test]
    fn padding_rect_defaults_missing_edges_to_zero() {
        let rect = utility_padding_rect_to_rect(&UtilityRect {
            left: Some(UtilityVal::Px(10.0)),
            right: Some(UtilityVal::Px(10.0)),
            ..default()
        });

        assert_eq!(rect.left, Val::Px(10.0));
        assert_eq!(rect.right, Val::Px(10.0));
        assert_eq!(rect.top, Val::ZERO);
        assert_eq!(rect.bottom, Val::ZERO);
    }

    #[test]
    fn border_rect_defaults_missing_edges_to_zero() {
        let rect = utility_border_rect_to_rect(&UtilityRect {
            top: Some(UtilityVal::Px(1.0)),
            ..default()
        });

        assert_eq!(rect.left, Val::ZERO);
        assert_eq!(rect.right, Val::ZERO);
        assert_eq!(rect.top, Val::Px(1.0));
        assert_eq!(rect.bottom, Val::ZERO);
    }
}

fn utility_overflow_axis(value: UtilityOverflowAxis) -> OverflowAxis {
    match value {
        UtilityOverflowAxis::Visible => OverflowAxis::Visible,
        UtilityOverflowAxis::Clip => OverflowAxis::Clip,
        UtilityOverflowAxis::Hidden => OverflowAxis::Hidden,
        UtilityOverflowAxis::Scroll => OverflowAxis::Scroll,
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
