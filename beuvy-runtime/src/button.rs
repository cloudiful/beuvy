mod build;
mod state;

use crate::text::LocalizedTextFormat;
use bevy::{input_focus::InputFocus, prelude::*};
use bevy_localization::TextKey;

pub use self::state::sync_button_active_state;
pub use build::default_button_node;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonSet {
    Build,
}

#[derive(Component, Default, Debug, Clone)]
pub struct Button {
    pub name: String,
}

#[derive(Component, Default, Debug, Clone)]
pub struct ButtonInner;

#[derive(Component, Debug, Clone, Copy)]
pub struct ButtonLabel {
    pub entity: Entity,
}

#[derive(Component, Default, Debug, Clone)]
pub struct DisabledButton;

#[derive(Component, Default, Debug, Clone)]
pub struct ActiveButton;

#[derive(Default)]
pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>()
            .add_message::<ButtonClickMessage>()
            .add_systems(Update, build::add_button.in_set(ButtonSet::Build));
    }
}

/// Declarative request to materialize a themed button.
#[derive(Component, Debug, Clone)]
pub struct AddButton {
    pub name: String,
    pub text: String,
    pub localized_text: Option<TextKey>,
    pub localized_text_format: Option<LocalizedTextFormat>,
    pub class: Option<String>,
    pub label_class: Option<String>,
    pub visible: bool,
    pub disabled: bool,
}

impl Default for AddButton {
    fn default() -> Self {
        Self {
            name: String::new(),
            text: String::new(),
            localized_text: None,
            localized_text_format: None,
            class: None,
            label_class: None,
            visible: true,
            disabled: false,
        }
    }
}

#[derive(Message, Debug, Clone)]
pub struct ButtonClickMessage {
    pub button: Button,
    pub entity: Entity,
}
