use super::text::update_input_text;
use super::value::can_insert_number_char;
use super::{
    DisabledInput, InputCursorPosition, InputField, InputType, InputValueChangedMessage,
    active_input_entity, clear_input_focus, is_printable_char, key_is_submit, push_value_changed,
    set_input_focus, sync_window_ime,
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
        field.edit_state.set_caret(byte, shift_pressed(&keys));
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
    mut pointer_clicks: MessageReader<Pointer<Click>>,
    fields: Query<(), With<InputField>>,
    mut input_focus: ResMut<InputFocus>,
) {
    let Some(active) = active_input_entity(&input_focus, &fields) else {
        return;
    };

    for click in pointer_clicks.read() {
        if click.entity == active {
            return;
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

    for keyboard_input in keyboard_inputs.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Backspace, _) => {
                changed |= field.edit_state.backspace();
                display_changed |= changed;
            }
            (Key::Delete, _) => {
                changed |= field.edit_state.delete_forward();
                display_changed |= changed;
            }
            (Key::ArrowLeft, _) => {
                display_changed |= field.edit_state.move_left(false);
            }
            (Key::ArrowRight, _) => {
                display_changed |= field.edit_state.move_right(false);
            }
            (Key::Home, _) => {
                display_changed |= field.edit_state.move_home(false);
            }
            (Key::End, _) => {
                display_changed |= field.edit_state.move_end(false);
            }
            (key, _) if key_is_submit(key) => {}
            (_, Some(inserted_text)) => {
                let filtered: String = inserted_text
                    .chars()
                    .filter(|chr| can_insert_char(&field, *chr))
                    .collect();
                if !filtered.is_empty() {
                    changed |= field.edit_state.insert_text(&filtered);
                    display_changed |= changed;
                }
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
    mut overlays: ParamSet<(
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
        let caret_x =
            text_origin.x + text_x_for_byte(layout, field.edit_state.display_caret_byte());
        let caret_top = text_origin.y;
        let caret_center_y = caret_top + text_height * 0.5;

        if field.selection_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = overlays.p0().get_mut(field.selection_entity) {
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
            if let Ok((mut node, mut visibility)) = overlays.p1().get_mut(field.caret_entity) {
                node.left = Val::Px((caret_x - caret_width * 0.5).max(0.0));
                node.top = Val::Px(caret_top);
                node.height = Val::Px(text_height.max(0.0));
                *visibility = if focused && !disabled {
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
        let glyph_x = glyph.position.x / glyph_scale;
        let glyph_width = glyph.size.x / glyph_scale;
        if byte <= glyph.byte_index {
            return glyph_x;
        }
        current_x = glyph_x + glyph_width;
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
        let start = glyph.position.x / glyph_scale;
        let width = glyph.size.x / glyph_scale;
        let end = start + width;
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

fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn can_insert_char(field: &InputField, chr: char) -> bool {
    match field.input_type {
        InputType::Text => is_printable_char(chr),
        InputType::Number => can_insert_number_char(chr, field.value(), field.min),
        InputType::Range => false,
    }
}
