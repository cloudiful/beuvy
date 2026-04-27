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
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::{Ime, PrimaryWindow};

pub(super) fn input_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    fields: Query<(), (With<InputField>, Without<DisabledInput>)>,
) {
    if !fields.contains(event.entity) {
        return;
    }

    set_input_focus(&mut input_focus, event.entity);
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
    fields: Query<(Entity, Option<&GlobalTransform>, Option<&ComputedNode>), With<InputField>>,
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
                let translation = transform.translation();
                entity_commands.try_insert(InputCursorPosition {
                    x: translation.x,
                    y: translation.y + computed.size().y * 0.5,
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

    for keyboard_input in keyboard_inputs.read() {
        if keyboard_input.state != ButtonState::Pressed {
            continue;
        }

        match (&keyboard_input.logical_key, &keyboard_input.text) {
            (Key::Backspace, _) => {
                changed |= field.value.pop().is_some();
                field.preedit = None;
            }
            (key, _) if key_is_submit(key) => {}
            (_, Some(inserted_text)) => {
                let filtered: String = inserted_text
                    .chars()
                    .filter(|chr| can_insert_char(&field, *chr))
                    .collect();
                if !filtered.is_empty() {
                    field.value.push_str(&filtered);
                    field.preedit = None;
                    changed = true;
                }
            }
            _ => {}
        }
    }

    if changed {
        update_input_text(&mut commands, &font_resource, &field, disabled);
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
                field.preedit = Some(value.clone());
                display_changed = true;
            }
            Ime::Preedit { cursor, .. } if cursor.is_none() => {
                field.preedit = None;
                display_changed = true;
            }
            Ime::Commit { value, .. } => {
                let value: String = value
                    .chars()
                    .filter(|chr| can_insert_char(&field, *chr))
                    .collect();
                field.value.push_str(&value);
                field.preedit = None;
                changed |= !value.is_empty();
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

fn can_insert_char(field: &InputField, chr: char) -> bool {
    match field.input_type {
        InputType::Text => is_printable_char(chr),
        InputType::Number => can_insert_number_char(chr, &field.value, field.min),
        InputType::Range => false,
    }
}
