mod build;
mod sync;

use crate::style::{font_size_control, text_primary_color};
use bevy::prelude::*;
use bevy::text::LineHeight;
use bevy_localization::{Localization, TextKey};

/// Materializes [`AddText`] components and keeps localized text in sync.
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build::setup)
            .add_systems(Update, build::add_text)
            .add_systems(
                PostUpdate,
                (
                    sync::sync_localized_text_on_binding_change,
                    sync::sync_localized_text_on_locale_change,
                    sync::sync_localized_text_format_on_binding_change,
                    sync::sync_localized_text_format_on_locale_change,
                ),
            );
    }
}

#[derive(Resource)]
pub struct FontResource {
    pub primary_font: Handle<Font>,
}

/// Declarative request to materialize a text entity using the active UI theme.
#[derive(Component, Debug, Clone)]
pub struct AddText {
    pub text: String,
    pub line_height: LineHeight,
    pub size: f32,
    pub color: Color,
    pub localized_text: Option<TextKey>,
    pub localized_text_format: Option<LocalizedTextFormat>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalizedText {
    pub key: TextKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedArg {
    pub name: &'static str,
    pub value: String,
}

impl LocalizedArg {
    pub fn new(name: &'static str, value: impl std::fmt::Display) -> Self {
        Self {
            name,
            value: value.to_string(),
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct LocalizedTextFormat {
    pub key: TextKey,
    pub args: Vec<LocalizedArg>,
}

impl LocalizedTextFormat {
    pub fn new(key: TextKey) -> Self {
        Self {
            key,
            args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, name: &'static str, value: impl std::fmt::Display) -> Self {
        self.args.push(LocalizedArg::new(name, value));
        self
    }
}

impl Default for AddText {
    fn default() -> Self {
        Self {
            text: "[missing text]".to_string(),
            line_height: LineHeight::RelativeToFont(1.5),
            size: font_size_control(),
            color: text_primary_color(),
            localized_text: None,
            localized_text_format: None,
        }
    }
}

impl AddText {
    /// Uses a localized text key instead of the raw `text` field.
    pub fn with_localized(mut self, key: TextKey) -> Self {
        self.localized_text = Some(key);
        self.localized_text_format = None;
        self
    }

    /// Uses a localized format string instead of the raw `text` field.
    pub fn with_localized_format(mut self, localized_text_format: LocalizedTextFormat) -> Self {
        self.localized_text = None;
        self.localized_text_format = Some(localized_text_format);
        self
    }
}

/// Replaces the text content with plain text and clears localization bindings.
pub fn set_plain_text(commands: &mut Commands, entity: Entity, text: impl Into<String>) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    entity_commands
        .try_insert(Text::new(text.into()))
        .try_remove::<LocalizedText>()
        .try_remove::<LocalizedTextFormat>();
}

/// Replaces the text content with a localized string resolved from `key`.
pub fn set_localized_text(
    commands: &mut Commands,
    entity: Entity,
    localization: &Localization,
    key: TextKey,
) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    entity_commands
        .try_insert((Text::new(localization.text(key)), LocalizedText { key }))
        .try_remove::<LocalizedTextFormat>();
}

/// Replaces the text content with a localized format string.
pub fn set_localized_text_format(
    commands: &mut Commands,
    entity: Entity,
    localization: &Localization,
    localized_text_format: LocalizedTextFormat,
) {
    let text = localization.format_text(
        localized_text_format.key,
        localized_text_format
            .args
            .iter()
            .map(|arg| (arg.name, arg.value.as_str())),
    );
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    entity_commands
        .try_insert((Text::new(text), localized_text_format))
        .try_remove::<LocalizedText>();
}
