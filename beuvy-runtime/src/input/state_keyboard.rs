use super::{
    can_insert_char, command_modifier_pressed, commit_numeric_field, control_pressed,
    step_number_field, sync_display_change, word_modifier_pressed,
};
use crate::input::{
    DisabledInput, InputField, InputValueChangedMessage, active_input_entity, key_is_submit,
    push_value_changed,
};
use crate::text::FontResource;
use bevy::input::ButtonState;
use bevy::input::{
    ButtonInput,
    keyboard::{Key, KeyCode, KeyboardInput},
};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

pub(crate) fn handle_keyboard_input(
    mut commands: Commands,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    fields_marker: Query<(), With<InputField>>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
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
    let extend_selection = super::shift_pressed(&keys);
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
        sync_display_change(&mut commands, &font_resource, &mut field, disabled, &time);
    }
    if pending_value_message {
        push_value_changed(&mut value_changed, entity, &field);
    }
}
