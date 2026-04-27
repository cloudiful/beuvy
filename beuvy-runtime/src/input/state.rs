use super::text::update_input_text;
use super::value::{can_insert_number_char, normalize_numeric_value};
use super::{
    DisabledInput, InputClickState, InputCursorPosition, InputField, InputScrollOffset, InputType,
    InputValueChangedMessage, active_input_entity, clear_input_focus, is_printable_char,
    key_is_submit, push_value_changed, set_input_focus, sync_window_ime,
};
use crate::focus::UiFocused;
use crate::text::FontResource;
use bevy::input::ButtonState;
use bevy::input::{
    ButtonInput,
    keyboard::{Key, KeyCode, KeyboardInput},
};
use bevy::input_focus::InputFocus;
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::window::{Ime, PrimaryWindow};

pub(super) fn input_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut fields: Query<
        (&mut InputField, &mut InputClickState),
        (With<InputField>, Without<DisabledInput>),
    >,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
) {
    let Ok((mut field, mut click_state)) = fields.get_mut(event.entity) else {
        return;
    };

    set_input_focus(&mut input_focus, event.entity);
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let now = time.elapsed_secs_f64();
        let is_chained_click = now - click_state.last_click_time <= 0.5;
        click_state.click_count = if is_chained_click {
            click_state.click_count.saturating_add(1).min(3)
        } else {
            1
        };
        click_state.last_click_time = now;

        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x);
        match click_state.click_count {
            1 => field.edit_state.set_caret(byte, shift_pressed(&keys)),
            2 => {
                field.edit_state.select_word_at(byte);
            }
            _ => {
                field.edit_state.select_all();
            }
        }
    }
    event.propagate(false);
}

pub(super) fn input_drag_start(
    mut event: On<Pointer<DragStart>>,
    mut input_focus: ResMut<InputFocus>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
) {
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    set_input_focus(&mut input_focus, event.entity);
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x);
        field.edit_state.set_caret(byte, false);
    }
    event.propagate(false);
}

pub(super) fn input_drag(
    mut event: On<Pointer<Drag>>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
) {
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x);
        field.edit_state.set_caret(byte, true);
    }
    event.propagate(false);
}

pub(super) fn input_drag_end(mut event: On<Pointer<DragEnd>>) {
    event.propagate(false);
}

pub(super) fn clear_input_focus_on_foreign_click(
    mut commands: Commands,
    mut pointer_clicks: MessageReader<Pointer<Click>>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>)>,
    mut input_focus: ResMut<InputFocus>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    let Some(active) = input_focus.get().filter(|entity| fields.contains(*entity)) else {
        return;
    };

    for click in pointer_clicks.read() {
        if click.entity == active {
            return;
        }
        if let Ok((mut field, disabled)) = fields.get_mut(active) {
            if commit_numeric_field(active, &mut field, &mut value_changed) {
                update_input_text(&mut commands, &font_resource, &field, disabled);
            }
        }
        clear_input_focus(&mut input_focus);
        return;
    }
}

pub(super) fn sync_input_focus_visuals(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    fields: Query<(Entity, Option<&UiGlobalTransform>, Option<&ComputedNode>), With<InputField>>,
    added_fields: Query<(), Added<InputField>>,
    focused: Query<Entity, With<UiFocused>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if !input_focus.is_changed() && added_fields.is_empty() {
        return;
    }

    let focus_target = input_focus.get();

    for (entity, transform, computed) in &fields {
        let should_focus = focus_target == Some(entity);
        let is_focused = focused.contains(entity);

        if should_focus == is_focused {
            continue;
        }

        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };

        if should_focus {
            entity_commands.try_insert(UiFocused);
            if let (Some(transform), Some(computed)) = (transform, computed) {
                let (_, _, center) = transform.to_scale_angle_translation();
                let rect = Rect::from_center_size(center, computed.size());
                let inverse = computed.inverse_scale_factor();
                let content_inset = computed.content_inset();
                let content_left = rect.min.x + content_inset.min_inset.x;
                entity_commands.try_insert(InputCursorPosition {
                    x: content_left * inverse,
                    y: rect.center().y * inverse,
                });
            }
        } else {
            entity_commands.try_remove::<UiFocused>();
        }
    }

    let primary_window = primary_window.iter().next();
    if focus_target.is_some() && focus_target == primary_window {
        for entity in &focused {
            let Ok(mut entity_commands) = commands.get_entity(entity) else {
                continue;
            };
            entity_commands.try_remove::<UiFocused>();
        }
    }
}

pub(super) fn handle_keyboard_input(
    mut commands: Commands,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    fields_marker: Query<(), With<InputField>>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    let Some(active) = active_input_entity(&input_focus, &fields_marker) else {
        return;
    };
    let Ok((entity, mut field, disabled)) = fields.get_mut(active) else {
        return;
    };
    if disabled {
        return;
    }

    let mut display_changed = false;
    let mut pending_value_message = false;
    let extend_selection = shift_pressed(&keys);
    let word_modifier = word_modifier_pressed(&keys);
    let command_modifier = command_modifier_pressed(&keys);
    let control_modifier = control_pressed(&keys);

    for keyboard_input in keyboard_inputs.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("a") => {
                display_changed |= field.edit_state.select_all();
            }
            (Key::Backspace, _) => {
                let edited = if word_modifier {
                    field.edit_state.backspace_word()
                } else {
                    field.edit_state.backspace()
                };
                pending_value_message |= edited;
                display_changed |= edited;
            }
            (Key::Delete, _) => {
                let edited = if word_modifier {
                    field.edit_state.delete_word_forward()
                } else {
                    field.edit_state.delete_forward()
                };
                pending_value_message |= edited;
                display_changed |= edited;
            }
            (Key::ArrowLeft, _) => {
                display_changed |= if word_modifier {
                    field.edit_state.move_word_left(extend_selection)
                } else {
                    field.edit_state.move_left(extend_selection)
                };
            }
            (Key::ArrowRight, _) => {
                display_changed |= if word_modifier {
                    field.edit_state.move_word_right(extend_selection)
                } else {
                    field.edit_state.move_right(extend_selection)
                };
            }
            (Key::Home, _) => {
                display_changed |= field.edit_state.move_home(extend_selection);
            }
            (Key::End, _) => {
                display_changed |= field.edit_state.move_end(extend_selection);
            }
            (Key::ArrowUp, _) => {
                let edited = step_number_field(&mut field, 1.0);
                pending_value_message |= edited;
                display_changed |= edited;
            }
            (Key::ArrowDown, _) => {
                let edited = step_number_field(&mut field, -1.0);
                pending_value_message |= edited;
                display_changed |= edited;
            }
            (key, _) if key_is_submit(key) => {
                let committed = commit_numeric_field(entity, &mut field, &mut value_changed);
                display_changed |= committed;
                if committed {
                    pending_value_message = false;
                }
            }
            (_, Some(inserted_text)) if !control_modifier && !command_modifier => {
                let filtered: String = inserted_text
                    .chars()
                    .filter(|chr| can_insert_char(&field, *chr))
                    .collect();
                if !filtered.is_empty() {
                    let edited = field.edit_state.insert_text(&filtered);
                    pending_value_message |= edited;
                    display_changed |= edited;
                }
            }
            _ => {}
        }
    }

    if display_changed {
        update_input_text(&mut commands, &font_resource, &field, disabled);
    }
    if pending_value_message {
        push_value_changed(&mut value_changed, entity, &field);
    }
}

pub(super) fn handle_ime_input(
    mut commands: Commands,
    mut ime_events: MessageReader<Ime>,
    fields_marker: Query<(), With<InputField>>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    let Some(active) = active_input_entity(&input_focus, &fields_marker) else {
        return;
    };
    let Ok((entity, mut field, disabled)) = fields.get_mut(active) else {
        return;
    };
    if disabled {
        return;
    }

    let mut changed = false;
    let mut display_changed = false;

    for ime in ime_events.read() {
        match ime {
            Ime::Preedit { value, cursor, .. } if cursor.is_some() => {
                field.edit_state.set_preedit(value.clone(), *cursor);
                display_changed = true;
            }
            Ime::Preedit { cursor, .. } if cursor.is_none() => {
                field.edit_state.clear_preedit();
                display_changed = true;
            }
            Ime::Commit { value, .. } => {
                let value: String = value
                    .chars()
                    .filter(|chr| can_insert_char(&field, *chr))
                    .collect();
                changed |= field.edit_state.commit_preedit_text(&value);
                display_changed |= !value.is_empty();
            }
            _ => {}
        }
    }

    if display_changed {
        update_input_text(&mut commands, &font_resource, &field, disabled);
    }
    if changed {
        push_value_changed(&mut value_changed, entity, &field);
    }
}

pub(super) fn sync_input_ime_state(
    input_focus: Res<InputFocus>,
    fields: Query<(&InputCursorPosition, Has<DisabledInput>), With<InputField>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(mut primary_window) = primary_window.iter_mut().next() else {
        return;
    };

    let Some(entity) = input_focus.get() else {
        sync_window_ime(&mut primary_window, false, Vec2::ZERO);
        return;
    };

    let Ok((cursor_position, disabled)) = fields.get(entity) else {
        sync_window_ime(&mut primary_window, false, Vec2::ZERO);
        return;
    };

    sync_window_ime(
        &mut primary_window,
        !disabled,
        Vec2::new(cursor_position.x.max(0.0), cursor_position.y.max(0.0)),
    );
}

pub(super) fn sync_input_edit_visuals(
    mut commands: Commands,
    time: Res<Time>,
    fields: Query<
        (
            Entity,
            &InputField,
            Has<DisabledInput>,
            Has<UiFocused>,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<InputField>,
    >,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
    mut visuals: ParamSet<(
        Query<(&mut InputScrollOffset, &mut Node)>,
        Query<(&mut Node, &mut Visibility), With<super::InputSelection>>,
        Query<(&mut Node, &mut Visibility), With<super::InputCaret>>,
    )>,
) {
    for (entity, field, disabled, focused, input_computed, input_transform) in &fields {
        if matches!(field.input_type, InputType::Range) || field.text_entity == Entity::PLACEHOLDER
        {
            continue;
        }
        let Ok((layout, text_computed, text_transform)) = text_nodes.get(field.text_entity) else {
            continue;
        };

        let input_rect = node_global_rect(input_computed, input_transform);
        let text_rect = node_global_rect(text_computed, text_transform);
        let input_scale = input_computed.inverse_scale_factor();
        let text_scale = text_computed.inverse_scale_factor();
        let text_origin = text_rect.min * text_scale - input_rect.min * input_scale;
        let text_height = layout.size.y.max(text_computed.size().y * text_scale);
        let caret_width = 2.0;
        let input_inset = input_computed.content_inset();
        let input_content_width = (input_computed.size().x * input_scale
            - input_inset.min_inset.x
            - input_inset.max_inset.x)
            .max(0.0);
        let raw_caret_x = text_x_for_byte(layout, field.edit_state.display_caret_byte());
        if focused && !disabled {
            if let Ok((mut scroll_offset, mut text_node)) = visuals.p0().get_mut(field.text_entity)
            {
                let max_offset = (layout.size.x - input_content_width).max(0.0);
                if raw_caret_x < scroll_offset.x {
                    scroll_offset.x = raw_caret_x.max(0.0);
                } else if raw_caret_x > scroll_offset.x + input_content_width {
                    scroll_offset.x = (raw_caret_x - input_content_width).max(0.0);
                }
                scroll_offset.x = scroll_offset.x.min(max_offset);
                text_node.left = Val::Px(-scroll_offset.x);
            }
        }
        let caret_x = text_origin.x + raw_caret_x;
        let caret_top = text_origin.y;
        let caret_center_y = caret_top + text_height * 0.5;

        if field.selection_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p1().get_mut(field.selection_entity) {
                if let Some(range) = field.edit_state.selection_range() {
                    let start_x = text_origin.x + text_x_for_byte(layout, range.start);
                    let end_x = text_origin.x + text_x_for_byte(layout, range.end);
                    node.left = Val::Px(start_x);
                    node.top = Val::Px(caret_top);
                    node.width = Val::Px((end_x - start_x).max(0.0));
                    node.height = Val::Px(text_height.max(0.0));
                    *visibility = if end_x > start_x {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }

        if field.caret_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p2().get_mut(field.caret_entity) {
                node.left = Val::Px((caret_x - caret_width * 0.5).max(0.0));
                node.top = Val::Px(caret_top);
                node.height = Val::Px(text_height.max(0.0));
                let blink_visible = (time.elapsed_secs_f64() * 2.0).floor() as i64 % 2 == 0;
                *visibility = if focused && !disabled && blink_visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        if focused && !disabled {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.try_insert(InputCursorPosition {
                    x: (input_rect.min.x * input_scale + caret_x).max(0.0),
                    y: caret_center_y.max(0.0),
                });
            }
        }
    }
}

fn node_global_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let (_, _, center) = transform.to_scale_angle_translation();
    Rect::from_center_size(center, computed.size())
}

fn node_logical_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let physical = node_global_rect(computed, transform);
    let inverse = computed.inverse_scale_factor();
    Rect {
        min: physical.min * inverse,
        max: physical.max * inverse,
    }
}

fn text_x_for_byte(layout: &TextLayoutInfo, byte: usize) -> f32 {
    if byte == 0 || layout.glyphs.is_empty() {
        return 0.0;
    }
    let glyph_scale = layout.scale_factor.max(f32::EPSILON);

    let mut current_x = 0.0;
    for glyph in &layout.glyphs {
        let glyph_x = glyph_left_x(glyph, glyph_scale);
        if byte <= glyph.byte_index {
            return glyph_x;
        }
        current_x = glyph_right_x(glyph, glyph_scale);
        if byte <= glyph.byte_index + glyph.byte_length {
            return current_x;
        }
    }

    current_x
}

fn text_byte_for_x(layout: &TextLayoutInfo, x: f32) -> usize {
    if layout.glyphs.is_empty() {
        return 0;
    }
    if x <= 0.0 {
        return 0;
    }
    let glyph_scale = layout.scale_factor.max(f32::EPSILON);

    for glyph in &layout.glyphs {
        let start = glyph_left_x(glyph, glyph_scale);
        let end = glyph_right_x(glyph, glyph_scale);
        let width = end - start;
        let midpoint = start + width * 0.5;
        if x < midpoint {
            return glyph.byte_index;
        }
        if x <= end {
            return glyph.byte_index + glyph.byte_length;
        }
    }

    layout
        .glyphs
        .last()
        .map(|glyph| glyph.byte_index + glyph.byte_length)
        .unwrap_or(0)
}

fn glyph_left_x(glyph: &bevy::text::PositionedGlyph, scale: f32) -> f32 {
    (glyph.position.x - glyph.size.x * 0.5) / scale
}

fn glyph_right_x(glyph: &bevy::text::PositionedGlyph, scale: f32) -> f32 {
    (glyph.position.x + glyph.size.x * 0.5) / scale
}

fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn control_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight)
}

fn alt_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
}

fn command_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight)
}

fn command_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    control_pressed(keys) || command_pressed(keys)
}

fn word_modifier_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    control_pressed(keys) || alt_pressed(keys)
}

fn can_insert_char(field: &InputField, chr: char) -> bool {
    match field.input_type {
        InputType::Text => is_printable_char(chr),
        InputType::Number => can_insert_number_char(chr, field.value(), field.min),
        InputType::Range => false,
    }
}

fn step_number_field(field: &mut InputField, direction: f32) -> bool {
    field.step_by(direction).is_some()
}

fn commit_numeric_field(
    entity: Entity,
    field: &mut InputField,
    value_changed: &mut MessageWriter<InputValueChangedMessage>,
) -> bool {
    if !matches!(field.input_type, InputType::Number) {
        return false;
    }
    let next = normalize_numeric_value(field.value(), field.min, field.max, field.step);
    if !field.edit_state.normalize_text(next) {
        return false;
    }
    value_changed.write(InputValueChangedMessage {
        entity,
        name: field.name.clone(),
        value: field.value().to_string(),
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::AssetId;
    use bevy::image::Image;
    use bevy::math::{IVec2, Vec2};
    use bevy::text::{GlyphAtlasInfo, GlyphAtlasLocation, PositionedGlyph};

    #[test]
    fn text_x_for_byte_uses_glyph_edges_not_centers() {
        let layout = layout_with_glyphs(&[(0, 1, 15.0, 10.0), (1, 1, 25.0, 10.0)]);

        assert_eq!(text_x_for_byte(&layout, 0), 0.0);
        assert_eq!(text_x_for_byte(&layout, 1), 20.0);
        assert_eq!(text_x_for_byte(&layout, 2), 30.0);
    }

    #[test]
    fn text_byte_for_x_uses_glyph_edge_midpoints() {
        let layout = layout_with_glyphs(&[(0, 1, 15.0, 10.0), (1, 1, 25.0, 10.0)]);

        assert_eq!(text_byte_for_x(&layout, 9.9), 0);
        assert_eq!(text_byte_for_x(&layout, 15.0), 1);
        assert_eq!(text_byte_for_x(&layout, 24.9), 1);
        assert_eq!(text_byte_for_x(&layout, 25.0), 2);
    }

    fn layout_with_glyphs(glyphs: &[(usize, usize, f32, f32)]) -> TextLayoutInfo {
        TextLayoutInfo {
            scale_factor: 1.0,
            glyphs: glyphs
                .iter()
                .map(
                    |(byte_index, byte_length, center_x, width)| PositionedGlyph {
                        position: Vec2::new(*center_x, 12.0),
                        size: Vec2::new(*width, 16.0),
                        atlas_info: GlyphAtlasInfo {
                            texture: AssetId::<Image>::invalid(),
                            texture_atlas: AssetId::invalid(),
                            location: GlyphAtlasLocation {
                                glyph_index: 0,
                                offset: IVec2::ZERO,
                            },
                        },
                        span_index: 0,
                        line_index: 0,
                        byte_index: *byte_index,
                        byte_length: *byte_length,
                    },
                )
                .collect(),
            run_geometry: Vec::new(),
            size: Vec2::new(30.0, 16.0),
        }
    }
}
