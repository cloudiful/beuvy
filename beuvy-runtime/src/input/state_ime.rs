use super::{can_insert_char, sync_display_change};
use crate::input::{
    DisabledInput, InputCursorPosition, InputField, InputValueChangedMessage, active_input_entity,
    push_value_changed, sync_window_ime,
};
use crate::text::FontResource;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::{Ime, PrimaryWindow};

pub(crate) fn handle_ime_input(
    mut commands: Commands,
    mut ime_events: MessageReader<Ime>,
    fields_marker: Query<(), With<InputField>>,
    mut fields: Query<(Entity, &mut InputField, Has<DisabledInput>)>,
    input_focus: Res<InputFocus>,
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
        sync_display_change(&mut commands, &font_resource, &mut field, disabled, &time);
    }
    if changed {
        push_value_changed(&mut value_changed, entity, &field);
    }
}

pub(crate) fn sync_input_ime_state(
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
