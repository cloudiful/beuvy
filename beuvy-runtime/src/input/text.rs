use super::{AddInput, DisabledInput, InputField, InputText};
use crate::style::{
    font_size_control, text_disabled_color, text_placeholder_color, text_primary_color,
};
use crate::text::{AddText, FontResource, set_plain_text};
use bevy::prelude::*;

pub(crate) fn default_input_node(size_chars: Option<usize>) -> Node {
    let width = size_chars.map(input_width_for_chars).unwrap_or(96.0);
    Node {
        width: size_chars.map_or(Val::Auto, |chars| Val::Px(input_width_for_chars(chars))),
        min_width: Val::Px(width),
        ..default()
    }
}

fn input_width_for_chars(size_chars: usize) -> f32 {
    let content_width = size_chars.max(1) as f32 * (font_size_control() * 0.6);
    content_width + 20.0
}

pub(crate) fn input_text_node() -> Node {
    Node {
        display: Display::Block,
        flex_grow: 1.0,
        min_width: Val::Px(0.0),
        ..default()
    }
}

fn input_display_text(field: &InputField) -> (&str, Color) {
    if let Some(preedit) = field.preedit.as_deref() {
        return (preedit, text_primary_color());
    }
    if !field.value.is_empty() {
        return (&field.value, text_primary_color());
    }
    (&field.placeholder, text_placeholder_color())
}

pub(crate) fn input_text_bundle(add_input: &AddInput) -> AddText {
    let preview = if add_input.value.is_empty() {
        add_input.placeholder.clone()
    } else {
        add_input.value.clone()
    };
    let color = if add_input.disabled {
        text_disabled_color()
    } else if add_input.value.is_empty() {
        text_placeholder_color()
    } else {
        text_primary_color()
    };

    AddText {
        text: preview,
        size: font_size_control(),
        color,
        ..default()
    }
}

pub(crate) fn update_input_text(
    commands: &mut Commands,
    font_resource: &FontResource,
    field: &InputField,
    disabled: bool,
) {
    if field.text_entity == Entity::PLACEHOLDER {
        return;
    }

    let text = if disabled {
        if field.value.is_empty() {
            field.placeholder.as_str()
        } else {
            field.value.as_str()
        }
    } else {
        input_display_text(field).0
    };
    set_plain_text(commands, field.text_entity, text);

    let Ok(mut entity_commands) = commands.get_entity(field.text_entity) else {
        return;
    };
    let color = if disabled {
        text_disabled_color()
    } else {
        input_display_text(field).1
    };
    entity_commands.try_insert((
        TextFont {
            font: font_resource.primary_font.clone(),
            font_size: font_size_control(),
            ..default()
        },
        TextColor(color),
    ));
}

pub fn set_input_value(
    commands: &mut Commands,
    font_resource: &FontResource,
    field: &mut InputField,
    disabled: bool,
    value: impl Into<String>,
) -> bool {
    let value = value.into();
    if field.value == value && field.preedit.is_none() {
        return false;
    }

    field.value = value;
    field.preedit = None;
    update_input_text(commands, font_resource, field, disabled);
    true
}

pub fn set_input_disabled(
    commands: &mut Commands,
    font_resource: &FontResource,
    entity: Entity,
    field: &InputField,
    disabled: bool,
) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    if disabled {
        entity_commands.try_insert((DisabledInput, crate::interaction_style::UiDisabled));
    } else {
        entity_commands
            .try_remove::<DisabledInput>()
            .try_remove::<crate::interaction_style::UiDisabled>();
    }

    update_input_text(commands, font_resource, field, disabled);
}

pub(crate) fn input_text_marker() -> InputText {
    InputText
}
