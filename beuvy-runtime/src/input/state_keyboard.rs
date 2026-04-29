use super::{
    can_insert_char, command_modifier_pressed, commit_numeric_field, control_pressed,
    set_checkable_state, step_number_field, sync_display_change, word_modifier_pressed,
};
use crate::input::{
    DisabledInput, InputClipboard, InputField, InputTextEngine, InputValueChangedMessage,
    active_input_entity, key_is_submit, push_value_changed,
};
use crate::text::FontResource;
use bevy::input::ButtonState;
use bevy::input::{
    ButtonInput,
    keyboard::{Key, KeyCode, KeyboardInput},
};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::text::ComputedTextBlock;

fn filter_pasted_text(field: &InputField, text: &str) -> String {
    let is_multiline = field.is_multiline();
    text.chars()
        .filter(|chr| {
            if *chr == '\n' {
                return is_multiline;
            }
            can_insert_char(field, *chr)
        })
        .collect()
}

pub(crate) fn handle_keyboard_input(
    mut commands: Commands,
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    fields_marker: Query<(), With<InputField>>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    radio_fields: Query<(Entity, &InputField), With<InputField>>,
    text_nodes: Query<&ComputedTextBlock, With<crate::input::InputText>>,
    viewports: Query<&ComputedNode, With<crate::input::InputViewport>>,
    input_focus: Res<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
    mut clipboard: NonSendMut<InputClipboard>,
    text_engine: Res<InputTextEngine>,
) {
    let Some(active) = active_input_entity(&input_focus, &fields_marker) else {
        return;
    };
    let mut display_changed = false;
    let mut pending_value_message = false;
    let mut pending_radio_group: Option<String> = None;
    let extend_selection = super::shift_pressed(&keys);
    let word_modifier = word_modifier_pressed(&keys);
    let command_modifier = command_modifier_pressed(&keys);
    let control_modifier = control_pressed(&keys);

    let disabled = {
        let Ok((entity, mut field, disabled)) = fields.get_mut(active) else {
            return;
        };
        if disabled {
            return;
        }

        for keyboard_input in keyboard_inputs.read() {
            if keyboard_input.state != ButtonState::Pressed {
                continue;
            }

            match (&keyboard_input.logical_key, &keyboard_input.text) {
                (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("a") => {
                    display_changed |= field.edit_state.select_all();
                }
                (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("c") => {
                    if let Some(range) = field.edit_state.selection_range() {
                        let selected = field.edit_state.committed()[range].to_string();
                        clipboard.set_text(&selected);
                    }
                }
                (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("x") => {
                    if let Some(range) = field.edit_state.selection_range() {
                        let selected = field.edit_state.committed()[range].to_string();
                        clipboard.set_text(&selected);
                        let current = field.value().to_string();
                        field.undo_stack.record(&current);
                        field.edit_state.backspace();
                        pending_value_message = true;
                        display_changed = true;
                    }
                }
                (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("v") => {
                    let text = clipboard.get_text();
                    if let Some(text) = text {
                        let filtered = filter_pasted_text(&field, &text);
                        if !filtered.is_empty() {
                            let current = field.value().to_string();
                            field.undo_stack.record(&current);
                            field.edit_state.insert_text(&filtered);
                            pending_value_message = true;
                            display_changed = true;
                        }
                    }
                }
                (Key::Character(key), _) if command_modifier && key.eq_ignore_ascii_case("z") => {
                    let shift = super::shift_pressed(&keys);
                    let current = field.value().to_string();
                    if shift {
                        if let Some(prev) = field.undo_stack.redo(&current) {
                            field.edit_state.set_text(prev);
                            pending_value_message = true;
                            display_changed = true;
                        }
                    } else if let Some(prev) = field.undo_stack.undo(&current) {
                        field.edit_state.set_text(prev);
                        pending_value_message = true;
                        display_changed = true;
                    }
                }
                (Key::Backspace, _) => {
                    let current = field.value().to_string();
                    field.undo_stack.record(&current);
                    let edited = if word_modifier {
                        field.edit_state.backspace_word()
                    } else {
                        field.edit_state.backspace()
                    };
                    pending_value_message |= edited;
                    display_changed |= edited;
                }
                (Key::Delete, _) => {
                    let current = field.value().to_string();
                    field.undo_stack.record(&current);
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
                    if field.is_multiline() {
                        if let (Some(viewport_entity), Some(text_entity)) =
                            (field.viewport_entity, field.text_entity)
                        {
                            if let (Ok(block), Ok(viewport)) =
                                (text_nodes.get(text_entity), viewports.get(viewport_entity))
                            {
                                if let Some((byte, preferred_x)) = text_engine.move_byte_vertically(
                                    block,
                                    viewport.inverse_scale_factor(),
                                    field.edit_state.display_caret_byte(),
                                    field.preferred_caret_x,
                                    -1,
                                ) {
                                    field.edit_state.set_caret(byte, extend_selection);
                                    field.preferred_caret_x = Some(preferred_x);
                                    display_changed = true;
                                }
                            }
                        }
                    } else {
                        let edited = step_number_field(&mut field, 1.0);
                        pending_value_message |= edited;
                        display_changed |= edited;
                    }
                }
                (Key::ArrowDown, _) => {
                    if field.is_multiline() {
                        if let (Some(viewport_entity), Some(text_entity)) =
                            (field.viewport_entity, field.text_entity)
                        {
                            if let (Ok(block), Ok(viewport)) =
                                (text_nodes.get(text_entity), viewports.get(viewport_entity))
                            {
                                if let Some((byte, preferred_x)) = text_engine.move_byte_vertically(
                                    block,
                                    viewport.inverse_scale_factor(),
                                    field.edit_state.display_caret_byte(),
                                    field.preferred_caret_x,
                                    1,
                                ) {
                                    field.edit_state.set_caret(byte, extend_selection);
                                    field.preferred_caret_x = Some(preferred_x);
                                    display_changed = true;
                                }
                            }
                        }
                    } else {
                        let edited = step_number_field(&mut field, -1.0);
                        pending_value_message |= edited;
                        display_changed |= edited;
                    }
                }
                (key, _) if key_is_submit(key) => {
                    if field.input_type == crate::input::InputType::Checkbox {
                        let next_checked = !field.checked;
                        let changed =
                            set_checkable_state(entity, &mut field, next_checked, &mut value_changed);
                        pending_value_message |= changed;
                        display_changed |= changed;
                    } else if field.input_type == crate::input::InputType::Radio {
                        let changed =
                            set_checkable_state(entity, &mut field, true, &mut value_changed);
                        pending_value_message |= changed;
                        display_changed |= changed;
                        pending_radio_group = Some(field.name.clone());
                    } else if field.is_multiline() {
                        let current = field.value().to_string();
                        field.undo_stack.record(&current);
                        let edited = field.edit_state.insert_text("\n");
                        pending_value_message |= edited;
                        display_changed |= edited;
                    } else {
                        let committed = commit_numeric_field(entity, &mut field, &mut value_changed);
                        display_changed |= committed;
                        if committed {
                            pending_value_message = false;
                        }
                    }
                }
                (_, Some(inserted_text)) if !control_modifier && !command_modifier => {
                    let filtered: String = inserted_text
                        .chars()
                        .filter(|chr| can_insert_char(&field, *chr))
                        .collect();
                    if !filtered.is_empty() {
                        let current = field.value().to_string();
                        field.undo_stack.record(&current);
                        let edited = field.edit_state.insert_text(&filtered);
                        pending_value_message |= edited;
                        display_changed |= edited;
                    }
                }
                _ => {}
            }
        }
        disabled
    };

    if let Some(group) = pending_radio_group {
        let targets = radio_fields
            .iter()
            .filter_map(|(candidate_entity, candidate)| {
                (candidate_entity != active
                    && candidate.input_type == crate::input::InputType::Radio
                    && candidate.name == group)
                    .then_some(candidate_entity)
            })
            .collect::<Vec<_>>();
        for candidate_entity in targets {
            if let Ok((_, mut candidate_field, _)) = fields.get_mut(candidate_entity) {
                let changed = set_checkable_state(
                    candidate_entity,
                    &mut candidate_field,
                    false,
                    &mut value_changed,
                );
                pending_value_message |= changed;
                display_changed |= changed;
            }
        }
    }

    if display_changed {
        if let Ok((_, mut field, _)) = fields.get_mut(active) {
            if !field.is_checkable() {
                sync_display_change(&mut commands, &font_resource, &mut field, disabled, &time);
            }
        }
    }
    if pending_value_message {
        if let Ok((entity, field, _)) = fields.get_mut(active) {
            if !field.is_checkable() {
                push_value_changed(&mut value_changed, entity, &field);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{InputType, TextEditState, UndoStack};

    fn field(input_type: InputType, value: &str) -> InputField {
        InputField {
            name: "input".to_string(),
            input_type,
            checked: false,
            input_value: None,
            placeholder: String::new(),
            viewport_entity: None,
            text_entity: None,
            selection_entity: None,
            caret_entity: None,
            edit_state: TextEditState::with_text(value),
            min: None,
            max: None,
            step: None,
            caret_blink_resume_at: 0.0,
            preferred_caret_x: None,
            undo_stack: UndoStack::default(),
        }
    }

    #[test]
    fn paste_preserves_newlines_for_textarea() {
        let field = field(InputType::Textarea, "");

        assert_eq!(filter_pasted_text(&field, "a\nb"), "a\nb");
    }

    #[test]
    fn paste_removes_newlines_for_single_line_input() {
        let field = field(InputType::Text, "");

        assert_eq!(filter_pasted_text(&field, "a\nb"), "ab");
    }
}
