use super::{UiStateVisualTransition, eased_progress};
use crate::interaction_style::model::{UiStateVisualSnapshot, UiStateVisualTarget};
use crate::interaction_style::resolve::apply_visual_target;
use bevy::color::Mix;
use bevy::prelude::*;

pub(crate) fn animate_state_visual_styles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut UiStateVisualTransition,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut TextColor>,
        Option<&mut Outline>,
        &UiStateVisualTarget,
        &UiStateVisualSnapshot,
    )>,
) {
    for (
        entity,
        mut transition,
        mut background,
        mut border,
        mut text,
        mut outline,
        target,
        snapshot,
    ) in &mut query
    {
        transition.elapsed_secs =
            (transition.elapsed_secs + time.delta_secs()).min(transition.duration_secs);
        let progress = eased_progress(
            transition.elapsed_secs / transition.duration_secs.max(f32::EPSILON),
            transition.timing,
        );

        if let Some(background) = background.as_deref_mut()
            && let Some(animation) = transition.background
        {
            background.0 = animation.from.mix(&animation.to, progress);
        }
        if let Some(border) = border.as_deref_mut()
            && let Some(animation) = transition.border
        {
            border.set_all(animation.from.mix(&animation.to, progress));
        }
        if let Some(text) = text.as_deref_mut()
            && let Some(animation) = transition.text
        {
            text.0 = animation.from.mix(&animation.to, progress);
        }
        if let Some(outline) = outline.as_deref_mut() {
            if let Some(animation) = transition.outline {
                outline.color = animation.from.mix(&animation.to, progress);
            }
            if let Some(width) = target.outline_width.or(snapshot.outline_width) {
                outline.width = width;
            }
            if let Some(offset) = snapshot.outline_offset {
                outline.offset = offset;
            }
        }

        if transition.elapsed_secs >= transition.duration_secs {
            apply_visual_target(background, border, text, outline, snapshot, target);
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.try_remove::<UiStateVisualTransition>();
            }
        }
    }
}
