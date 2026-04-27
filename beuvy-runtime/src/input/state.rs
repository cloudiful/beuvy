#[path = "state_ime.rs"]
mod ime;
#[path = "state_keyboard.rs"]
mod keyboard;
#[path = "state_metrics.rs"]
mod metrics;
#[path = "state_pointer.rs"]
mod pointer;
#[path = "state_visuals.rs"]
mod visuals;

use super::text::update_input_text;
use super::value::{can_insert_number_char, normalize_numeric_value};
use super::{InputField, InputType, InputValueChangedMessage, is_printable_char};
use crate::text::FontResource;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::prelude::*;

pub(crate) use ime::{handle_ime_input, sync_input_ime_state};
pub(crate) use keyboard::handle_keyboard_input;
pub(crate) use pointer::{
    clear_input_focus_on_foreign_click, input_click, input_drag, input_drag_end, input_drag_start,
};
pub(crate) use visuals::{sync_input_edit_visuals, sync_input_focus_visuals};

const CARET_BLINK_RESUME_DELAY_SECS: f64 = 0.6;

fn keep_caret_visible(field: &mut InputField, time: &Time) {
    field.caret_blink_resume_at = time.elapsed_secs_f64() + CARET_BLINK_RESUME_DELAY_SECS;
}

fn sync_display_change(
    commands: &mut Commands,
    font_resource: &FontResource,
    field: &mut InputField,
    disabled: bool,
    time: &Time,
) {
    keep_caret_visible(field, time);
    update_input_text(commands, font_resource, field, disabled);
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
