use super::text::update_input_text;
use super::value::{can_insert_number_char, is_valid_number_buffer};
use super::{
    DisabledInput, InputCursorPosition, InputField, InputType, InputValueChangedMessage,
    InputValueCommittedMessage, clear_input_focus, is_printable_char, key_is_submit,
    push_value_changed, push_value_committed, set_input_focus, sync_window_ime,
};
use crate::focus::UiFocused;
use crate::text::FontResource;
use bevy::ecs::query::{QueryData, QueryFilter};
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
use std::process::{Command, Stdio};

const HORIZONTAL_SCROLL_PADDING: f32 = 12.0;

pub(super) fn input_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut fields: ParamSet<(
        Query<&mut InputField, With<InputField>>,
        Query<(&mut InputField, Has<DisabledInput>), With<InputField>>,
    )>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
    mut committed: MessageWriter<InputValueCommittedMessage>,
) {
    let active = active_input_entity(input_focus.as_ref(), &fields.p0());
    commit_previous_active_input(
        &mut input_focus,
        active,
        &mut fields.p0(),
        &mut committed,
        Some(event.entity),
    );

    let mut target_fields = fields.p1();
    let Ok((mut field, disabled)) = target_fields.get_mut(event.entity) else {
        return;
    };
    if disabled {
        return;
    }

    set_input_focus(&mut input_focus, event.entity);
    field.begin_focus_session();
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x + field.horizontal_scroll_px);
        let click_count = field.register_click(time.elapsed_secs_f64());
        if click_count >= 3 {
            field.edit_state.select_all();
        } else if click_count == 2 {
            field.edit_state.select_word_at(byte);
        } else {
            field.edit_state.set_caret(byte, shift_pressed(&keys));
        }
    }
    event.propagate(false);
}

pub(super) fn input_drag_start(
    mut event: On<Pointer<DragStart>>,
    mut input_focus: ResMut<InputFocus>,
    mut fields: ParamSet<(
        Query<&mut InputField, With<InputField>>,
        Query<(&mut InputField, Has<DisabledInput>), With<InputField>>,
    )>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
    mut committed: MessageWriter<InputValueCommittedMessage>,
) {
    let active = active_input_entity(input_focus.as_ref(), &fields.p0());
    commit_previous_active_input(
        &mut input_focus,
        active,
        &mut fields.p0(),
        &mut committed,
        Some(event.entity),
    );

    let mut target_fields = fields.p1();
    let Ok((mut field, disabled)) = target_fields.get_mut(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    set_input_focus(&mut input_focus, event.entity);
    field.begin_focus_session();
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x + field.horizontal_scroll_px);
        field.edit_state.set_caret(byte, false);
    }
    event.propagate(false);
}

pub(super) fn input_drag(
    mut event: On<Pointer<Drag>>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>), With<InputField>>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<super::InputText>>,
) {
    let Ok((mut field, disabled)) = fields.get_mut(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    if let Ok((layout, computed, transform)) = text_nodes.get(field.text_entity) {
        let logical_rect = node_logical_rect(computed, transform);
        let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
        let byte = text_byte_for_x(layout, local_x + field.horizontal_scroll_px);
        field.edit_state.set_caret(byte, true);
    }
    event.propagate(false);
}

pub(super) fn input_drag_end(mut event: On<Pointer<DragEnd>>) {
    event.propagate(false);
}

pub(super) fn clear_input_focus_on_foreign_click(
    mut pointer_clicks: MessageReader<Pointer<Click>>,
    mut fields: Query<&mut InputField, With<InputField>>,
    mut input_focus: ResMut<InputFocus>,
    mut committed: MessageWriter<InputValueCommittedMessage>,
) {
    let Some(active) = active_input_entity(input_focus.as_ref(), &fields) else {
        return;
    };

    for click in pointer_clicks.read() {
        if click.entity == active || fields.contains(click.entity) {
            return;
        }
        commit_previous_active_input(
            &mut input_focus,
            Some(active),
            &mut fields,
            &mut committed,
            None,
        );
        return;
    }
}

pub(super) fn sync_input_focus_visuals(
    mut commands: Commands,
    input_focus: Res<InputFocus>,
    mut input_params: ParamSet<(
        Query<
            (
                Entity,
                &mut InputField,
                Option<&UiGlobalTransform>,
                Option<&ComputedNode>,
            ),
            With<InputField>,
        >,
        Query<(), Added<InputField>>,
    )>,
    focused: Query<Entity, With<UiFocused>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if !input_focus.is_changed() && input_params.p1().is_empty() {
        return;
    }

    let focus_target = input_focus.get();

    for (entity, mut field, transform, computed) in &mut input_params.p0() {
        let should_focus = focus_target == Some(entity);
        let is_focused = focused.contains(entity);

        if should_focus {
            field.begin_focus_session();
        } else {
            field.end_focus_session();
        }

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
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
    font_resource: Res<FontResource>,
    keys: Res<ButtonInput<KeyCode>>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
    mut committed: MessageWriter<InputValueCommittedMessage>,
) {
    let Some(active) = active_input_entity(input_focus.as_ref(), &fields) else {
        return;
    };
    let Ok((entity, mut field, disabled)) = fields.get_mut(active) else {
        return;
    };
    if disabled {
        return;
    }

    let modifiers = KeyboardModifiers::from_keys(&keys);
    let mut changed = false;
    let mut display_changed = false;
    let mut commit_requested = false;

    for keyboard_input in keyboard_inputs.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        if modifiers.shortcut && shortcut_copy(&keyboard_input.logical_key) {
            if let Some(selection) = selected_text(&field) {
                let _ = write_clipboard_text(&selection);
            }
            continue;
        }
        if modifiers.shortcut && shortcut_cut(&keyboard_input.logical_key) {
            if let Some(selection) = selected_text(&field) {
                let _ = write_clipboard_text(&selection);
                changed |= field.edit_state.replace_selection("");
                if changed {
                    field.mark_dirty_from_value();
                }
                display_changed |= changed;
            }
            continue;
        }
        if modifiers.shortcut && shortcut_paste(&keyboard_input.logical_key) {
            if let Some(text) = read_clipboard_text() {
                let inserted = insert_text_for_field(&mut field, &text);
                changed |= inserted;
                if inserted {
                    field.mark_dirty_from_value();
                }
                display_changed |= inserted;
            }
            continue;
        }
        if modifiers.shortcut && shortcut_select_all(&keyboard_input.logical_key) {
            display_changed |= field.edit_state.select_all();
            continue;
        }

        if matches!(field.input_type, InputType::Range) {
            let handled = handle_range_key(
                &keyboard_input.logical_key,
                entity,
                &mut field,
                &mut value_changed,
                &mut committed,
            );
            changed |= handled.changed;
            display_changed |= handled.display_changed;
            commit_requested |= handled.committed;
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Backspace, _) => {
                let delta = if modifiers.word_navigation {
                    field.edit_state.backspace_word()
                } else {
                    field.edit_state.backspace()
                };
                changed |= delta;
                if delta {
                    field.mark_dirty_from_value();
                }
                display_changed |= delta;
            }
            (Key::Delete, _) => {
                let delta = if modifiers.word_navigation {
                    field.edit_state.delete_word_forward()
                } else {
                    field.edit_state.delete_forward()
                };
                changed |= delta;
                if delta {
                    field.mark_dirty_from_value();
                }
                display_changed |= delta;
            }
            (Key::ArrowLeft, _) => {
                display_changed |= if modifiers.word_navigation {
                    field.edit_state.move_word_left(modifiers.shift)
                } else {
                    field.edit_state.move_left(modifiers.shift)
                };
            }
            (Key::ArrowRight, _) => {
                display_changed |= if modifiers.word_navigation {
                    field.edit_state.move_word_right(modifiers.shift)
                } else {
                    field.edit_state.move_right(modifiers.shift)
                };
            }
            (Key::Home, _) => {
                display_changed |= field.edit_state.move_home(modifiers.shift);
            }
            (Key::End, _) => {
                display_changed |= field.edit_state.move_end(modifiers.shift);
            }
            (key, _) if key_is_submit(key) => {
                commit_requested = true;
            }
            (_, Some(inserted_text)) => {
                let inserted = insert_text_for_field(&mut field, inserted_text);
                changed |= inserted;
                if inserted {
                    field.mark_dirty_from_value();
                }
                display_changed |= inserted;
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
    if commit_requested && field.dirty_since_focus {
        field.value_on_focus = field.value().to_string();
        field.dirty_since_focus = false;
        push_value_committed(&mut committed, entity, &field);
    }
}

pub(super) fn handle_ime_input(
    mut commands: Commands,
    mut ime_events: MessageReader<Ime>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    let Some(active) = active_input_entity(input_focus.as_ref(), &fields) else {
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
                let inserted = insert_text_for_field(&mut field, value);
                changed |= inserted;
                if inserted {
                    field.mark_dirty_from_value();
                }
                display_changed |= inserted;
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
    mut fields: Query<
        (
            Entity,
            &mut InputField,
            Has<DisabledInput>,
            Has<UiFocused>,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<InputField>,
    >,
    mut node_layers: ParamSet<(
        Query<
            (
                &TextLayoutInfo,
                &ComputedNode,
                &UiGlobalTransform,
                &mut Node,
            ),
            With<super::InputText>,
        >,
        Query<(&mut Node, &mut Visibility), With<super::InputSelection>>,
        Query<(&mut Node, &mut Visibility), With<super::InputCaret>>,
    )>,
) {
    for (entity, mut field, disabled, focused, input_computed, input_transform) in &mut fields {
        if matches!(field.input_type, InputType::Range) || field.text_entity == Entity::PLACEHOLDER
        {
            continue;
        }
        let input_rect = node_global_rect(input_computed, input_transform);
        let input_scale = input_computed.inverse_scale_factor();
        let (text_origin, text_height, caret_x, selection_bounds) = {
            let mut text_nodes = node_layers.p0();
            let Ok((layout, text_computed, text_transform, mut text_node)) =
                text_nodes.get_mut(field.text_entity)
            else {
                continue;
            };

            sync_horizontal_scroll(&mut field, layout, input_computed, focused && !disabled);
            text_node.left = Val::Px(-field.horizontal_scroll_px);

            let text_rect = node_global_rect(text_computed, text_transform);
            let text_scale = text_computed.inverse_scale_factor();
            let text_origin = text_rect.min * text_scale - input_rect.min * input_scale;
            let text_height = layout.size.y.max(text_computed.size().y * text_scale);
            let caret_x =
                text_origin.x + text_x_for_byte(layout, field.edit_state.display_caret_byte());
            let selection_bounds = field.edit_state.selection_range().map(|range| {
                (
                    text_origin.x + text_x_for_byte(layout, range.start),
                    text_origin.x + text_x_for_byte(layout, range.end),
                )
            });
            (text_origin, text_height, caret_x, selection_bounds)
        };

        let caret_width = 2.0;
        let caret_top = text_origin.y;
        let caret_center_y = caret_top + text_height * 0.5;

        if field.selection_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = node_layers.p1().get_mut(field.selection_entity)
            {
                if let Some((start_x, end_x)) = selection_bounds {
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
            if let Ok((mut node, mut visibility)) = node_layers.p2().get_mut(field.caret_entity) {
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

#[derive(Default)]
struct KeyboardModifiers {
    shift: bool,
    shortcut: bool,
    word_navigation: bool,
}

impl KeyboardModifiers {
    fn from_keys(keys: &ButtonInput<KeyCode>) -> Self {
        let shift = shift_pressed(keys);
        let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        let alt = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);
        let meta = keys.pressed(KeyCode::SuperLeft) || keys.pressed(KeyCode::SuperRight);
        Self {
            shift,
            shortcut: shortcut_modifier(control, meta),
            word_navigation: word_navigation_modifier(control, alt),
        }
    }
}

struct RangeKeyResult {
    changed: bool,
    display_changed: bool,
    committed: bool,
}

fn handle_range_key(
    key: &Key,
    entity: Entity,
    field: &mut InputField,
    value_changed: &mut MessageWriter<InputValueChangedMessage>,
    committed: &mut MessageWriter<InputValueCommittedMessage>,
) -> RangeKeyResult {
    let mut result = RangeKeyResult {
        changed: false,
        display_changed: false,
        committed: false,
    };
    let changed = match key {
        Key::ArrowLeft | Key::ArrowDown => field.step_by(-1.0).is_some(),
        Key::ArrowRight | Key::ArrowUp => field.step_by(1.0).is_some(),
        Key::Home => set_range_endpoint(field, field.min.unwrap_or(0.0)),
        Key::End => set_range_endpoint(field, field.max.unwrap_or(1.0)),
        _ => false,
    };
    if changed {
        field.mark_dirty_from_value();
        push_value_changed(value_changed, entity, field);
        if field.dirty_since_focus {
            field.value_on_focus = field.value().to_string();
            field.dirty_since_focus = false;
            push_value_committed(committed, entity, field);
        }
        result.changed = true;
        result.display_changed = true;
        result.committed = true;
    }
    result
}

fn set_range_endpoint(field: &mut InputField, value: f32) -> bool {
    let snapped = super::value::snap_numeric_value(value, field.min, field.max, field.step);
    let next = super::value::format_numeric_value(snapped, field.step);
    if field.value() == next {
        return false;
    }
    field.set_value(next);
    true
}

fn selected_text(field: &InputField) -> Option<String> {
    let range = field.edit_state.selection_range()?;
    field.value().get(range).map(str::to_string)
}

fn insert_text_for_field(field: &mut InputField, text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    match field.input_type {
        InputType::Text => {
            let filtered: String = text.chars().filter(|chr| is_printable_char(*chr)).collect();
            field.edit_state.insert_text(&filtered)
        }
        InputType::Number => {
            let filtered: String = text
                .chars()
                .filter(|chr| is_printable_char(*chr) && can_insert_number_char(*chr, "", field.min))
                .collect();
            if filtered.is_empty() {
                return false;
            }
            let mut preview = field.edit_state.clone();
            if !preview.commit_preedit_text(&filtered) {
                return false;
            }
            if !is_valid_number_buffer(preview.committed(), field.min) {
                return false;
            }
            field.edit_state.commit_preedit_text(&filtered)
        }
        InputType::Range => false,
    }
}

fn commit_previous_active_input(
    input_focus: &mut ResMut<InputFocus>,
    active: Option<Entity>,
    fields: &mut Query<&mut InputField, With<InputField>>,
    committed: &mut MessageWriter<InputValueCommittedMessage>,
    next_focus: Option<Entity>,
) {
    let Some(active) = active else {
        return;
    };
    if next_focus == Some(active) {
        return;
    }
    if let Ok(mut field) = fields.get_mut(active) {
        if field.dirty_since_focus {
            field.value_on_focus = field.value().to_string();
            field.dirty_since_focus = false;
            push_value_committed(committed, active, &field);
        }
        field.end_focus_session();
    }
    if next_focus.is_none() {
        clear_input_focus(input_focus);
    }
}

fn active_input_entity<T, F>(input_focus: &InputFocus, fields: &Query<T, F>) -> Option<Entity>
where
    T: QueryData,
    F: QueryFilter,
{
    input_focus.get().filter(|entity| fields.contains(*entity))
}

fn sync_horizontal_scroll(
    field: &mut InputField,
    layout: &TextLayoutInfo,
    input_computed: &ComputedNode,
    focused: bool,
) {
    if !focused || field.value().is_empty() {
        field.horizontal_scroll_px = 0.0;
        return;
    }
    let input_scale = input_computed.inverse_scale_factor();
    let content_inset = input_computed.content_inset();
    let viewport_width = (input_computed.size().x
        - content_inset.min_inset.x
        - content_inset.max_inset.x)
        * input_scale;
    if viewport_width <= 0.0 {
        field.horizontal_scroll_px = 0.0;
        return;
    }

    let target_byte = field
        .edit_state
        .selection_range()
        .map(|range| range.end)
        .unwrap_or_else(|| field.edit_state.display_caret_byte());
    let caret_x = text_x_for_byte(layout, target_byte);
    let total_width = layout.size.x / layout.scale_factor.max(f32::EPSILON);
    let max_scroll = (total_width - viewport_width).max(0.0);
    let visible_left = field.horizontal_scroll_px;
    let visible_right = visible_left + viewport_width;

    if caret_x + HORIZONTAL_SCROLL_PADDING > visible_right {
        field.horizontal_scroll_px =
            (caret_x + HORIZONTAL_SCROLL_PADDING - viewport_width).clamp(0.0, max_scroll);
    } else if caret_x - HORIZONTAL_SCROLL_PADDING < visible_left {
        field.horizontal_scroll_px =
            (caret_x - HORIZONTAL_SCROLL_PADDING).clamp(0.0, max_scroll);
    } else {
        field.horizontal_scroll_px = field.horizontal_scroll_px.clamp(0.0, max_scroll);
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

fn shortcut_select_all(key: &Key) -> bool {
    character_key_eq(key, "a")
}

fn shortcut_copy(key: &Key) -> bool {
    character_key_eq(key, "c")
}

fn shortcut_cut(key: &Key) -> bool {
    character_key_eq(key, "x")
}

fn shortcut_paste(key: &Key) -> bool {
    character_key_eq(key, "v")
}

fn character_key_eq(key: &Key, expected: &str) -> bool {
    matches!(key, Key::Character(value) if value.eq_ignore_ascii_case(expected))
}

#[cfg(target_os = "macos")]
fn shortcut_modifier(control: bool, meta: bool) -> bool {
    let _ = control;
    meta
}

#[cfg(not(target_os = "macos"))]
fn shortcut_modifier(control: bool, meta: bool) -> bool {
    let _ = meta;
    control
}

#[cfg(target_os = "macos")]
fn word_navigation_modifier(control: bool, alt: bool) -> bool {
    let _ = control;
    alt
}

#[cfg(not(target_os = "macos"))]
fn word_navigation_modifier(control: bool, alt: bool) -> bool {
    let _ = alt;
    control
}

fn read_clipboard_text() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pbpaste").output().ok()?;
        return String::from_utf8(output.stdout).ok();
    }
    #[cfg(target_os = "linux")]
    {
        for command in [["wl-paste", "--no-newline"], ["xclip", "-selection", "clipboard", "-o"]] {
            let output = Command::new(command[0]).args(&command[1..]).output().ok();
            if let Some(output) = output {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Some(text);
                }
            }
        }
        None
    }
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-Clipboard"])
            .output()
            .ok()?;
        return String::from_utf8(output.stdout).ok();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

fn write_clipboard_text(text: &str) -> bool {
    #[cfg(target_os = "macos")]
    {
        return Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                let Some(mut stdin) = child.stdin.take() else {
                    return Err(std::io::Error::other("pbcopy stdin unavailable"));
                };
                stdin.write_all(text.as_bytes())?;
                child.wait().map(|status| status.success())
            })
            .unwrap_or(false);
    }
    #[cfg(target_os = "linux")]
    {
        for command in [["wl-copy"], ["xclip", "-selection", "clipboard"]] {
            let result = Command::new(command[0])
                .args(&command[1..])
                .stdin(Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    let Some(mut stdin) = child.stdin.take() else {
                        return Err(std::io::Error::other("clipboard stdin unavailable"));
                    };
                    stdin.write_all(text.as_bytes())?;
                    child.wait().map(|status| status.success())
                });
            if result.unwrap_or(false) {
                return true;
            }
        }
        false
    }
    #[cfg(target_os = "windows")]
    {
        return Command::new("powershell")
            .args(["-NoProfile", "-Command", "Set-Clipboard"])
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                let Some(mut stdin) = child.stdin.take() else {
                    return Err(std::io::Error::other("Set-Clipboard stdin unavailable"));
                };
                stdin.write_all(text.as_bytes())?;
                child.wait().map(|status| status.success())
            })
            .unwrap_or(false);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = text;
        false
    }
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
