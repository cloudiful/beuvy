use super::value::{format_numeric_value, range_progress, snap_numeric_value};
use super::{
    DisabledInput, InputField, InputValueChangedMessage, RangeFill, RangeState, RangeThumb,
    RangeTrack, push_range_value_changed,
};
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, ComputedUiRenderTargetInfo, UiGlobalTransform, UiScale, Val::Px};

pub(crate) const RANGE_TRACK_WIDTH: f32 = 240.0;
pub(crate) const RANGE_TRACK_HEIGHT: f32 = 32.0;
const RANGE_THUMB_VISUAL_NUDGE_Y: f32 = -1.0;
const RANGE_FILL_PADDING_Y: f32 = 3.0;
const RANGE_THUMB_PADDING: f32 = 4.0;

pub(crate) fn range_track_inset() -> f32 {
    crate::style::regular_border_width()
}

fn range_fill_inset_y() -> f32 {
    range_track_inset() + RANGE_FILL_PADDING_Y
}

pub(crate) fn range_thumb_size() -> f32 {
    (RANGE_TRACK_HEIGHT - range_track_inset() * 2.0 - RANGE_THUMB_PADDING * 2.0).max(12.0)
}

fn centered_thumb_top(track_height: f32, thumb_size: f32) -> f32 {
    ((track_height - thumb_size) * 0.5 + RANGE_THUMB_VISUAL_NUDGE_Y).max(0.0)
}

fn slider_theme() -> &'static crate::style::UiThemeConfig {
    crate::style::ui_theme()
}

pub(crate) fn spawn_range_track(world: &mut World, input: Entity) -> Entity {
    let slider = &slider_theme().control.slider;
    let mut node = Node {
        min_width: Px(120.0),
        max_width: Px(RANGE_TRACK_WIDTH),
        height: Px(RANGE_TRACK_HEIGHT),
        flex_grow: 1.0,
        overflow: Overflow::clip(),
        ..default()
    };
    node.border = UiRect::all(Px(range_track_inset()));
    node.border_radius = BorderRadius::MAX;
    world
        .spawn((
            RangeTrack { input },
            node,
            BorderColor::all(slider.track.border.to_bevy()),
            BackgroundColor(slider.track.background.to_bevy()),
            Visibility::Visible,
        ))
        .observe(range_track_press)
        .observe(range_track_drag_start)
        .observe(range_track_drag)
        .observe(range_track_drag_end)
        .id()
}

pub(crate) fn spawn_range_fill(world: &mut World) -> Entity {
    let slider = &slider_theme().control.slider;
    world
        .spawn((
            RangeFill,
            Pickable::IGNORE,
            {
                let mut node = Node {
                    position_type: PositionType::Absolute,
                    left: Px(range_track_inset()),
                    top: Px(range_fill_inset_y()),
                    bottom: Px(range_fill_inset_y()),
                    width: Px(0.0),
                    ..default()
                };
                node.border_radius = BorderRadius::MAX;
                node
            },
            BackgroundColor(slider.fill_background.to_bevy()),
        ))
        .id()
}

pub(crate) fn spawn_range_thumb(world: &mut World) -> Entity {
    let slider = &slider_theme().control.slider;
    let size = range_thumb_size();
    world
        .spawn((
            RangeThumb,
            Pickable::IGNORE,
            {
                let mut node = Node {
                    position_type: PositionType::Absolute,
                    left: Px(range_track_inset()),
                    top: Px(centered_thumb_top(RANGE_TRACK_HEIGHT, size)),
                    width: Px(size),
                    height: Px(size),
                    ..default()
                };
                node.border_radius = BorderRadius::MAX;
                node
            },
            BorderColor::all(slider.thumb_border.to_bevy()),
            BackgroundColor(slider.thumb_background.to_bevy()),
        ))
        .id()
}

pub(crate) fn sync_range_visuals(
    fields: Query<(Entity, &InputField, &RangeState), Or<(Changed<InputField>, Added<InputField>)>>,
    track_nodes: Query<&ComputedNode, With<RangeTrack>>,
    thumb_nodes: Query<&ComputedNode, With<RangeThumb>>,
    mut nodes: Query<&mut Node>,
) {
    for (_entity, field, range_state) in &fields {
        let Ok(track_node) = track_nodes.get(range_state.track) else {
            continue;
        };
        let thumb_size = thumb_nodes
            .get(range_state.thumb)
            .map(computed_node_logical_width)
            .unwrap_or_else(|_| range_thumb_size());
        let thumb_height = thumb_nodes
            .get(range_state.thumb)
            .map(computed_node_logical_height)
            .unwrap_or(thumb_size);
        let min = field.min.unwrap_or(0.0);
        let max = field.max.unwrap_or(1.0);
        let value = field.numeric_value().unwrap_or(min);
        let progress = range_progress(value, min, max);
        let inset = range_track_inset();
        let track_width = computed_node_logical_width(track_node);
        let track_height = computed_node_logical_height(track_node);
        let travel = (track_width - inset * 2.0 - thumb_size).max(1.0);
        let fill_width = inset + thumb_size * 0.5 + travel * progress;
        let thumb_left = inset + travel * progress;
        let thumb_top = centered_thumb_top(track_height, thumb_height);

        if let Ok(mut fill_node) = nodes.get_mut(range_state.fill) {
            fill_node.width = Px(snap_slider_pixel(fill_width.max(0.0)));
        }
        if let Ok(mut thumb_node) = nodes.get_mut(range_state.thumb) {
            thumb_node.left = Px(snap_slider_pixel(thumb_left.max(0.0)));
            thumb_node.top = Px(snap_slider_pixel(thumb_top));
        }
    }
}

fn range_track_press(
    mut event: On<Pointer<Press>>,
    tracks: Query<(
        &RangeTrack,
        &ComputedNode,
        &ComputedUiRenderTargetInfo,
        &UiGlobalTransform,
    )>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>)>,
    mut range_states: Query<&mut RangeState>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
    thumbs: Query<&ComputedNode, With<RangeThumb>>,
    ui_scale: Res<UiScale>,
) {
    let Ok((track, node, node_target, transform)) = tracks.get(event.entity) else {
        return;
    };
    event.propagate(false);
    let Ok((mut field, disabled)) = fields.get_mut(track.input) else {
        return;
    };
    if disabled {
        return;
    }
    let mut range_state = range_states
        .get_mut(track.input)
        .expect("range state missing");
    range_state.drag_start_value = field.numeric_value().unwrap_or(field.min.unwrap_or(0.0));
    let thumb_size = thumbs
        .get(range_state.thumb)
        .map(computed_node_logical_width)
        .unwrap_or_else(|_| range_thumb_size());
    let value = pointer_position_to_range_value(
        event.pointer_location.position,
        node,
        node_target.scale_factor(),
        transform,
        &field,
        thumb_size,
        &ui_scale,
    );
    commit_range_value(track.input, &mut field, value, &mut value_changed);
}

fn range_track_drag_start(
    mut event: On<Pointer<DragStart>>,
    tracks: Query<&RangeTrack>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>)>,
    mut range_states: Query<&mut RangeState>,
) {
    let Ok(track) = tracks.get(event.entity) else {
        return;
    };
    event.propagate(false);
    let Ok((field, disabled)) = fields.get_mut(track.input) else {
        return;
    };
    if disabled {
        return;
    }
    let mut range_state = range_states
        .get_mut(track.input)
        .expect("range state missing");
    range_state.drag_start_value = field.numeric_value().unwrap_or(field.min.unwrap_or(0.0));
}

fn range_track_drag(
    mut event: On<Pointer<Drag>>,
    tracks: Query<(&RangeTrack, &ComputedNode)>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>)>,
    range_states: Query<&RangeState>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
    thumbs: Query<&ComputedNode, With<RangeThumb>>,
    ui_scale: Res<UiScale>,
) {
    let Ok((track, node)) = tracks.get(event.entity) else {
        return;
    };
    event.propagate(false);
    let Ok((mut field, disabled)) = fields.get_mut(track.input) else {
        return;
    };
    if disabled {
        return;
    }
    let range_state = range_states.get(track.input).expect("range state missing");
    let thumb_size = thumbs
        .get(range_state.thumb)
        .map(computed_node_logical_width)
        .unwrap_or_else(|_| range_thumb_size());
    let value = drag_distance_to_range_value(
        &field,
        range_state.drag_start_value,
        computed_node_logical_width(node),
        thumb_size,
        event.distance.x,
        &ui_scale,
    );
    commit_range_value(track.input, &mut field, value, &mut value_changed);
}

fn range_track_drag_end(mut event: On<Pointer<DragEnd>>) {
    event.propagate(false);
}

fn commit_range_value(
    entity: Entity,
    field: &mut InputField,
    value: f32,
    value_changed: &mut MessageWriter<InputValueChangedMessage>,
) {
    let snapped = snap_numeric_value(value, field.min, field.max, field.step);
    let next_value = format_numeric_value(snapped, field.step);
    if field.value() == next_value {
        return;
    }
    field.set_value(next_value.clone());
    push_range_value_changed(value_changed, entity, &field.name, next_value);
}

fn pointer_position_to_range_value(
    pointer_position: Vec2,
    node: &ComputedNode,
    target_scale_factor: f32,
    transform: &UiGlobalTransform,
    field: &InputField,
    thumb_size: f32,
    ui_scale: &UiScale,
) -> f32 {
    let Some(inverse_transform) = transform.try_inverse() else {
        return field.numeric_value().unwrap_or(field.min.unwrap_or(0.0));
    };
    let local_pos =
        inverse_transform.transform_point2(pointer_position * target_scale_factor / ui_scale.0);
    let track_width = computed_node_logical_width(node);
    let x_from_left = local_pos.x * node.inverse_scale_factor() + track_width * 0.5;
    let adjusted_x = x_from_left - range_track_inset() - thumb_size * 0.5;
    let travel = (track_width - range_track_inset() * 2.0 - thumb_size).max(1.0);
    let min = field.min.unwrap_or(0.0);
    let max = field.max.unwrap_or(1.0);
    min + (adjusted_x / travel).clamp(0.0, 1.0) * (max - min)
}

fn computed_node_logical_width(node: &ComputedNode) -> f32 {
    node.size().x * node.inverse_scale_factor()
}

fn computed_node_logical_height(node: &ComputedNode) -> f32 {
    node.size().y * node.inverse_scale_factor()
}

fn drag_distance_to_range_value(
    field: &InputField,
    drag_start_value: f32,
    track_width: f32,
    thumb_size: f32,
    drag_distance_x: f32,
    ui_scale: &UiScale,
) -> f32 {
    let travel = (track_width - range_track_inset() * 2.0 - thumb_size).max(1.0);
    let min = field.min.unwrap_or(0.0);
    let max = field.max.unwrap_or(1.0);
    let span = max - min;
    if span <= f32::EPSILON {
        return min;
    }
    let logical_drag_distance = drag_distance_x / ui_scale.0;
    (drag_start_value + (logical_drag_distance / travel) * span).clamp(min, max)
}

fn snap_slider_pixel(value: f32) -> f32 {
    value.round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::edit::TextEditState;

    fn range_field(min: f32, max: f32, step: f32, value: &str) -> InputField {
        InputField {
            name: "range".to_string(),
            input_type: super::super::InputType::Range,
            checked: false,
            input_value: None,
            placeholder: String::new(),
            viewport_entity: None,
            text_entity: None,
            selection_entity: None,
            caret_entity: None,
            edit_state: TextEditState::with_text(value),
            min: Some(min),
            max: Some(max),
            step: Some(step),
            caret_blink_resume_at: 0.0,
            preferred_caret_x: None,
            undo_stack: crate::input::UndoStack::default(),
        }
    }

    fn computed_node(logical_width: f32, scale: f32) -> ComputedNode {
        ComputedNode {
            size: Vec2::new(logical_width * scale, RANGE_TRACK_HEIGHT * scale),
            inverse_scale_factor: scale.recip(),
            ..default()
        }
    }

    fn pointer_for_logical_x(x_from_left: f32, logical_width: f32, scale: f32) -> Vec2 {
        Vec2::new((x_from_left - logical_width * 0.5) * scale, 0.0)
    }

    fn value_at_logical_x(x_from_left: f32, scale: f32) -> f32 {
        let logical_width = RANGE_TRACK_WIDTH;
        let node = computed_node(logical_width, scale);
        let field = range_field(0.0, 100.0, 10.0, "0");
        pointer_position_to_range_value(
            pointer_for_logical_x(x_from_left, logical_width, scale),
            &node,
            scale,
            &UiGlobalTransform::default(),
            &field,
            range_thumb_size(),
            &UiScale(scale),
        )
    }

    #[test]
    fn pointer_position_to_range_value_maps_midpoint_at_default_scale() {
        assert_eq!(
            value_at_logical_x(RANGE_TRACK_WIDTH * 0.5, 1.0).round(),
            50.0
        );
    }

    #[test]
    fn pointer_position_to_range_value_keeps_logical_progress_under_ui_scale() {
        let x_from_left = RANGE_TRACK_WIDTH * 0.7;

        assert_eq!(
            value_at_logical_x(x_from_left, 1.0).round(),
            value_at_logical_x(x_from_left, 1.3).round()
        );
    }

    #[test]
    fn pointer_position_to_range_value_snaps_after_commit() {
        let travel = RANGE_TRACK_WIDTH - range_track_inset() * 2.0 - range_thumb_size();
        let x_from_left = range_track_inset() + range_thumb_size() * 0.5 + travel * 0.74;
        let value = value_at_logical_x(x_from_left, 1.3);

        assert_eq!(
            format_numeric_value(
                snap_numeric_value(value, Some(0.0), Some(100.0), Some(10.0)),
                Some(10.0)
            ),
            "70"
        );
    }

    #[test]
    fn centered_thumb_top_applies_visual_nudge() {
        let top = centered_thumb_top(RANGE_TRACK_HEIGHT, range_thumb_size());
        assert!(top < (RANGE_TRACK_HEIGHT - range_thumb_size()) * 0.5);
        assert!(top >= 0.0);
    }

    #[test]
    fn drag_distance_to_range_value_uses_total_drag_distance() {
        let field = InputField {
            ..range_field(0.0, 100.0, 10.0, "40")
        };
        let track_width = RANGE_TRACK_WIDTH;
        let thumb_size = range_thumb_size();
        let travel = track_width - range_track_inset() * 2.0 - thumb_size;
        let value = drag_distance_to_range_value(
            &field,
            40.0,
            track_width,
            thumb_size,
            travel * 0.2,
            &UiScale(1.0),
        );
        assert!((value - 60.0).abs() < 0.001);
    }
}
