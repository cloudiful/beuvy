use super::model::{
    UiDisabled, UiStateStyleSource, UiStateVisualSnapshot, UiStateVisualStyles, UiStateVisualTarget,
};
use super::transition::{UiStateVisualTransition, transition_color};
use crate::focus::{UiFocused, UiHovered, UiPressed};
use crate::style::{UiThemeConfig, resolve_color_value_with_config};
use crate::stylesheet::runtime_style_sheet;
use crate::utility::{
    UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal, UtilityVisualStylePatch,
};
use bevy::prelude::*;

pub(super) fn capture_state_visual_snapshots(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&BackgroundColor>,
            Option<&BorderColor>,
            Option<&TextColor>,
            Option<&Outline>,
        ),
        (
            With<UiStateVisualStyles>,
            Or<(Added<UiStateVisualStyles>, Added<UiStateStyleSource>)>,
            Without<UiStateVisualSnapshot>,
        ),
    >,
) {
    for (entity, background, border, text, outline) in &query {
        let snapshot = UiStateVisualSnapshot {
            background: background.map(|value| value.0),
            border: border.map(|value| value.top),
            text: text.map(|value| value.0),
            outline_width: outline.map(|value| value.width),
            outline_offset: outline.map(|value| value.offset),
            outline_color: outline.map(|value| value.color),
        };
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.try_insert(snapshot);
        }
    }
}

#[allow(clippy::type_complexity)]
pub(super) fn resolve_state_visual_styles(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &UiStateVisualStyles,
        Option<&UiStateStyleSource>,
        &UiStateVisualSnapshot,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut TextColor>,
        Option<&mut Outline>,
        Option<&UiStateVisualTarget>,
    )>,
    states: Query<(
        Has<UiHovered>,
        Has<UiPressed>,
        Has<UiFocused>,
        Has<UiDisabled>,
    )>,
) {
    let runtime_sheet = runtime_style_sheet();
    let runtime_config = runtime_sheet.config();
    for (
        entity,
        styles,
        state_source,
        snapshot,
        background,
        border,
        text,
        outline,
        current_target,
    ) in &mut query
    {
        let has_background = background.is_some();
        let has_border = border.is_some();
        let has_text = text.is_some();
        let has_outline = outline.is_some();
        let state_entity = state_source.map(|value| value.0).unwrap_or(entity);
        let (hovered, pressed, focused, disabled) = states
            .get(state_entity)
            .unwrap_or((false, false, false, false));
        let patch = effective_visual_patch(styles, hovered, pressed, focused, disabled);
        let target = resolve_visual_target(runtime_config, &patch, snapshot);

        if current_target.copied() == Some(target) {
            continue;
        }

        let should_transition = matches!(
            patch.transition_property,
            Some(UtilityTransitionProperty::All | UtilityTransitionProperty::Colors)
        ) && patch.transition_duration_ms.unwrap_or_default() > 0.0;

        if should_transition {
            let transition = UiStateVisualTransition {
                background: transition_color(
                    background.as_deref().map(|value| value.0),
                    target.background,
                ),
                border: transition_color(border.as_deref().map(|value| value.top), target.border),
                text: transition_color(text.as_deref().map(|value| value.0), target.text),
                outline: transition_color(snapshot.outline_color, target.outline_color),
                duration_secs: patch.transition_duration_ms.unwrap_or_default() / 1000.0,
                elapsed_secs: 0.0,
                timing: patch
                    .transition_timing
                    .unwrap_or(UtilityTransitionTiming::EaseInOut),
            };

            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                ensure_visual_target_components(
                    &mut entity_commands,
                    &target,
                    has_background,
                    has_border,
                    has_text,
                    has_outline,
                );
                if transition.has_values() {
                    entity_commands.try_insert(transition);
                } else {
                    entity_commands.try_remove::<UiStateVisualTransition>();
                }
                entity_commands.try_insert(target);
            }

            if !transition.has_values() {
                apply_visual_target(background, border, text, outline, snapshot, &target);
            }
        } else {
            apply_visual_target(background, border, text, outline, snapshot, &target);
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                ensure_visual_target_components(
                    &mut entity_commands,
                    &target,
                    has_background,
                    has_border,
                    has_text,
                    has_outline,
                );
                entity_commands.try_remove::<UiStateVisualTransition>();
                entity_commands.try_insert(target);
            }
        }
    }
}

fn effective_visual_patch(
    styles: &UiStateVisualStyles,
    hovered: bool,
    pressed: bool,
    focused: bool,
    disabled: bool,
) -> UtilityVisualStylePatch {
    let mut patch = styles.base.clone();
    if disabled {
        merge_visual_patch(&mut patch, &styles.disabled);
    } else {
        if focused {
            merge_visual_patch(&mut patch, &styles.focus);
        }
        if hovered {
            merge_visual_patch(&mut patch, &styles.hover);
        }
        if pressed {
            merge_visual_patch(&mut patch, &styles.active);
        }
    }
    patch
}

fn merge_visual_patch(target: &mut UtilityVisualStylePatch, patch: &UtilityVisualStylePatch) {
    if let Some(value) = &patch.background_color {
        target.background_color = Some(value.clone());
    }
    if let Some(value) = &patch.text_color {
        target.text_color = Some(value.clone());
    }
    if let Some(value) = &patch.border_color {
        target.border_color = Some(value.clone());
    }
    if let Some(value) = patch.outline_width {
        target.outline_width = Some(value);
    }
    if let Some(value) = &patch.outline_color {
        target.outline_color = Some(value.clone());
    }
    if let Some(value) = patch.opacity {
        target.opacity = Some(value);
    }
    if let Some(value) = patch.transition_property {
        target.transition_property = Some(value);
    }
    if let Some(value) = patch.transition_duration_ms {
        target.transition_duration_ms = Some(value);
    }
    if let Some(value) = patch.transition_timing {
        target.transition_timing = Some(value);
    }
}

fn resolve_visual_target(
    config: &UiThemeConfig,
    patch: &UtilityVisualStylePatch,
    snapshot: &UiStateVisualSnapshot,
) -> UiStateVisualTarget {
    UiStateVisualTarget {
        background: resolve_color_target(
            config,
            patch.background_color.as_deref(),
            snapshot.background,
            patch.opacity,
        ),
        border: resolve_color_target(
            config,
            patch.border_color.as_deref(),
            snapshot.border,
            patch.opacity,
        ),
        text: resolve_color_target(
            config,
            patch.text_color.as_deref(),
            snapshot.text,
            patch.opacity,
        ),
        outline_width: patch
            .outline_width
            .and_then(utility_val_to_val)
            .or(snapshot.outline_width),
        outline_color: resolve_color_target(
            config,
            patch.outline_color.as_deref(),
            snapshot.outline_color,
            patch.opacity,
        ),
    }
}

fn resolve_color_target(
    config: &UiThemeConfig,
    raw: Option<&str>,
    fallback: Option<Color>,
    opacity: Option<f32>,
) -> Option<Color> {
    let color = raw
        .and_then(|value| resolve_color_value_with_config(config, value))
        .or(fallback)?;
    Some(apply_opacity(color, opacity))
}

fn apply_opacity(color: Color, opacity: Option<f32>) -> Color {
    let Some(opacity) = opacity else {
        return color;
    };
    let [red, green, blue, alpha] = color.to_srgba().to_f32_array();
    Color::srgba(red, green, blue, alpha * opacity.clamp(0.0, 1.0))
}

fn utility_val_to_val(value: UtilityVal) -> Option<Val> {
    match value {
        UtilityVal::Auto => Some(Val::Auto),
        UtilityVal::Px(value) => Some(Val::Px(value)),
        UtilityVal::Percent(value) => Some(Val::Percent(value)),
        UtilityVal::Vw(value) => Some(Val::Vw(value)),
        UtilityVal::Vh(value) => Some(Val::Vh(value)),
    }
}

pub(super) fn apply_visual_target(
    background: Option<Mut<BackgroundColor>>,
    border: Option<Mut<BorderColor>>,
    text: Option<Mut<TextColor>>,
    outline: Option<Mut<Outline>>,
    snapshot: &UiStateVisualSnapshot,
    target: &UiStateVisualTarget,
) {
    if let Some(mut background) = background
        && let Some(color) = target.background
    {
        background.0 = color;
    }
    if let Some(mut border) = border
        && let Some(color) = target.border
    {
        border.set_all(color);
    }
    if let Some(mut text) = text
        && let Some(color) = target.text
    {
        text.0 = color;
    }
    if let Some(mut outline) = outline {
        if let Some(width) = target.outline_width.or(snapshot.outline_width) {
            outline.width = width;
        }
        if let Some(color) = target.outline_color {
            outline.color = color;
        }
        if let Some(offset) = snapshot.outline_offset {
            outline.offset = offset;
        }
    }
}

fn ensure_visual_target_components(
    entity: &mut EntityCommands,
    target: &UiStateVisualTarget,
    has_background: bool,
    has_border: bool,
    has_text: bool,
    has_outline: bool,
) {
    if !has_background && let Some(color) = target.background {
        entity.try_insert(BackgroundColor(color));
    }
    if !has_border && let Some(color) = target.border {
        entity.try_insert(BorderColor::all(color));
    }
    if !has_text && let Some(color) = target.text {
        entity.try_insert(TextColor(color));
    }
    if !has_outline && (target.outline_width.is_some() || target.outline_color.is_some()) {
        entity.try_insert(Outline::new(
            target.outline_width.unwrap_or(Val::Px(0.0)),
            Val::Px(0.0),
            target.outline_color.unwrap_or(Color::NONE),
        ));
    }
}
