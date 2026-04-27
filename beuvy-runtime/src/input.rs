mod build;
mod edit;
mod range;
mod state;
mod text;
mod value;

pub use text::{set_input_disabled, set_input_value};

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::window::Ime;
use edit::TextEditState;

const MULTI_CLICK_INTERVAL_SECS: f64 = 0.5;

pub struct InputPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSet {
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputType {
    #[default]
    Text,
    Number,
    Range,
}

/// Declarative request to materialize an input field using the active UI theme.
#[derive(Component, Debug, Clone)]
pub struct AddInput {
    pub name: String,
    pub input_type: InputType,
    pub value: String,
    pub placeholder: String,
    pub size_chars: Option<usize>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub class: Option<String>,
    pub text_class: Option<String>,
    pub disabled: bool,
}

impl Default for AddInput {
    fn default() -> Self {
        Self {
            name: String::new(),
            input_type: InputType::Text,
            value: String::new(),
            placeholder: String::new(),
            size_chars: None,
            min: None,
            max: None,
            step: None,
            class: None,
            text_class: None,
            disabled: false,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct InputField {
    pub name: String,
    pub input_type: InputType,
    pub placeholder: String,
    pub text_entity: Entity,
    pub selection_entity: Entity,
    pub caret_entity: Entity,
    pub edit_state: TextEditState,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub step: Option<f32>,
    pub range_track: Option<Entity>,
    pub range_fill: Option<Entity>,
    pub range_thumb: Option<Entity>,
    pub drag_start_value: f32,
    pub focused: bool,
    pub dirty_since_focus: bool,
    pub value_on_focus: String,
    pub horizontal_scroll_px: f32,
    pub last_click_at: f64,
    pub click_count: u8,
}

impl InputField {
    pub fn value(&self) -> &str {
        self.edit_state.committed()
    }

    pub fn set_value(&mut self, value: impl Into<String>) {
        self.edit_state.set_text(value);
    }

    pub fn begin_focus_session(&mut self) {
        if self.focused {
            return;
        }
        self.focused = true;
        self.dirty_since_focus = false;
        self.value_on_focus = self.value().to_string();
    }

    pub fn end_focus_session(&mut self) {
        self.focused = false;
        self.click_count = 0;
        self.last_click_at = 0.0;
    }

    pub fn mark_dirty_from_value(&mut self) {
        self.dirty_since_focus = self.value() != self.value_on_focus;
    }

    pub fn numeric_value(&self) -> Option<f32> {
        value::parse_number_buffer(self.value())
    }

    pub fn step_by(&mut self, direction: f32) -> Option<String> {
        if !matches!(self.input_type, InputType::Number | InputType::Range) || direction == 0.0 {
            return None;
        }
        let current = self
            .numeric_value()
            .unwrap_or_else(|| self.min.unwrap_or(0.0));
        let step = self.step.unwrap_or(1.0);
        let next = value::snap_numeric_value(
            current + step * direction.signum(),
            self.min,
            self.max,
            self.step,
        );
        let next_value = value::format_numeric_value(next, self.step);
        if self.value() == next_value {
            return None;
        }
        self.set_value(next_value.clone());
        self.mark_dirty_from_value();
        Some(next_value)
    }

    pub fn register_click(&mut self, now: f64) -> u8 {
        if now - self.last_click_at <= MULTI_CLICK_INTERVAL_SECS {
            self.click_count = self.click_count.saturating_add(1).min(3);
        } else {
            self.click_count = 1;
        }
        self.last_click_at = now;
        self.click_count
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct InputText;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DisabledInput;

#[derive(Component, Debug, Clone, Copy)]
pub struct InputSelection;

#[derive(Component, Debug, Clone, Copy)]
pub struct InputCaret;

#[derive(Component, Debug, Clone, Copy)]
pub struct InputCursorPosition {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RangeTrack {
    pub input: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RangeFill;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct RangeThumb;

#[derive(Message, Debug, Clone)]
pub struct InputValueChangedMessage {
    pub entity: Entity,
    pub name: String,
    pub value: String,
}

#[derive(Message, Debug, Clone)]
pub struct InputValueCommittedMessage {
    pub entity: Entity,
    pub name: String,
    pub value: String,
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InputValueChangedMessage>()
            .add_message::<InputValueCommittedMessage>()
            .add_message::<KeyboardInput>()
            .add_message::<Ime>()
            .add_message::<Pointer<Click>>()
            .init_resource::<InputFocus>()
            .add_systems(
                Update,
                (
                    build::add_input.in_set(InputSet::Build),
                    state::clear_input_focus_on_foreign_click,
                    state::sync_input_focus_visuals,
                    range::sync_range_visuals,
                ),
            )
            .add_systems(
                Update,
                (
                    state::handle_keyboard_input,
                    state::handle_ime_input,
                    state::sync_input_ime_state,
                ),
            )
            .add_systems(PostUpdate, state::sync_input_edit_visuals);
    }
}

fn set_input_focus(input_focus: &mut InputFocus, entity: Entity) {
    input_focus.set(entity);
}

fn clear_input_focus(input_focus: &mut InputFocus) {
    input_focus.clear();
}

fn push_value_changed(
    value_changed: &mut MessageWriter<InputValueChangedMessage>,
    entity: Entity,
    field: &InputField,
) {
    value_changed.write(InputValueChangedMessage {
        entity,
        name: field.name.clone(),
        value: field.value().to_string(),
    });
}

fn push_value_committed(
    committed: &mut MessageWriter<InputValueCommittedMessage>,
    entity: Entity,
    field: &InputField,
) {
    committed.write(InputValueCommittedMessage {
        entity,
        name: field.name.clone(),
        value: field.value().to_string(),
    });
}

fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr);
    !chr.is_control() && !is_in_private_use_area
}

fn key_is_submit(key: &Key) -> bool {
    matches!(key, Key::Enter)
}

fn sync_window_ime(primary_window: &mut Window, enabled: bool, position: Vec2) {
    primary_window.ime_enabled = enabled;
    primary_window.ime_position = position;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::FontResource;
    use bevy::input::ButtonInput;
    use bevy::input::keyboard::KeyCode;
    use bevy::ui::UiScale;

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<UiScale>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(FontResource::default())
            .add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn pointer_group_initializes_without_conflicting_queries() {
        let mut app = test_app();
        app.add_message::<InputValueChangedMessage>()
            .add_message::<InputValueCommittedMessage>()
            .add_message::<KeyboardInput>()
            .add_message::<Ime>()
            .add_message::<Pointer<Click>>()
            .init_resource::<InputFocus>()
            .add_systems(
                Update,
                (
                    state::clear_input_focus_on_foreign_click,
                    state::sync_input_focus_visuals,
                    range::sync_range_visuals,
                ),
            );
        app.update();
    }

    #[test]
    fn keyboard_group_initializes_without_conflicting_queries() {
        let mut app = test_app();
        app.add_message::<InputValueChangedMessage>()
            .add_message::<InputValueCommittedMessage>()
            .add_message::<KeyboardInput>()
            .add_message::<Ime>()
            .init_resource::<InputFocus>()
            .add_systems(
                Update,
                (
                    state::handle_keyboard_input,
                    state::handle_ime_input,
                    state::sync_input_ime_state,
                ),
            );
        app.update();
    }

    #[test]
    fn postupdate_group_initializes_without_conflicting_queries() {
        let mut app = test_app();
        app.add_systems(PostUpdate, state::sync_input_edit_visuals);
        app.update();
    }
}
