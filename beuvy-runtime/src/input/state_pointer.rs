use super::metrics::{node_logical_rect, text_byte_for_point, text_byte_for_x};
use super::{keep_caret_visible, shift_pressed};
use crate::input::{
    DisabledInput, InputClickState, InputField, InputValueChangedMessage, clear_input_focus,
    set_input_focus,
};
use crate::text::FontResource;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::TextLayoutInfo;

pub(crate) fn input_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut fields: Query<
        (&mut InputField, &mut InputClickState),
        (With<InputField>, Without<DisabledInput>),
    >,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<crate::input::InputText>>,
) {
    let Ok((mut field, mut click_state)) = fields.get_mut(event.entity) else {
        return;
    };

    set_input_focus(&mut input_focus, event.entity);
    let Some(text_entity) = field.text_entity else {
        return;
    };
    let Ok((layout, computed, transform)) = text_nodes.get(text_entity) else {
        return;
    };

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
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let display_text = field.edit_state.display_text_string(&field.placeholder);
    let byte = if field.is_multiline() {
        text_byte_for_point(layout, &display_text.text, local_x, local_y)
    } else {
        text_byte_for_x(layout, &display_text.text, local_x)
    };
    match click_state.click_count {
        1 => field.edit_state.set_caret(byte, shift_pressed(&keys)),
        2 => {
            field.edit_state.select_word_at(byte);
        }
        _ => {
            field.edit_state.select_all();
        }
    }
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag_start(
    mut event: On<Pointer<DragStart>>,
    mut input_focus: ResMut<InputFocus>,
    time: Res<Time>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<crate::input::InputText>>,
) {
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    set_input_focus(&mut input_focus, event.entity);
    let Some(text_entity) = field.text_entity else {
        return;
    };
    let Ok((layout, computed, transform)) = text_nodes.get(text_entity) else {
        return;
    };

    let logical_rect = node_logical_rect(computed, transform);
    let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let display_text = field.edit_state.display_text_string(&field.placeholder);
    let byte = if field.is_multiline() {
        text_byte_for_point(layout, &display_text.text, local_x, local_y)
    } else {
        text_byte_for_x(layout, &display_text.text, local_x)
    };
    field.edit_state.set_caret(byte, false);
    field.preferred_caret_x = Some(local_x);
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag(
    mut event: On<Pointer<Drag>>,
    time: Res<Time>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<crate::input::InputText>>,
) {
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    let Some(text_entity) = field.text_entity else {
        return;
    };
    let Ok((layout, computed, transform)) = text_nodes.get(text_entity) else {
        return;
    };

    let logical_rect = node_logical_rect(computed, transform);
    let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let display_text = field.edit_state.display_text_string(&field.placeholder);
    let byte = if field.is_multiline() {
        text_byte_for_point(layout, &display_text.text, local_x, local_y)
    } else {
        text_byte_for_x(layout, &display_text.text, local_x)
    };
    field.edit_state.set_caret(byte, true);
    field.preferred_caret_x = Some(local_x);
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag_end(mut event: On<Pointer<DragEnd>>) {
    event.propagate(false);
}

pub(crate) fn clear_input_focus_on_foreign_click(
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
            if super::commit_numeric_field(active, &mut field, &mut value_changed) {
                crate::input::text::update_input_text(&mut commands, &font_resource, &field, disabled);
            }
        }
        clear_input_focus(&mut input_focus);
        return;
    }
}
