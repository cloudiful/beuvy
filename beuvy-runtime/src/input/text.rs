use super::{AddInput, DisabledInput, InputField, InputText, InputType};
use crate::style::{
    control_radius, font_size_control, regular_border, text_disabled_color,
    text_placeholder_color, text_primary_color,
};
use crate::text::{AddText, FontResource, set_plain_text};
use bevy::prelude::*;
use bevy::text::{LineBreak, TextLayout};

pub(crate) fn default_input_node(size_chars: Option<usize>) -> Node {
    let width = size_chars.map(input_width_for_chars).unwrap_or(96.0);
    Node {
        width: size_chars.map_or(Val::Auto, |chars| Val::Px(input_width_for_chars(chars))),
        min_width: Val::Px(width),
        ..default()
    }
}

pub(crate) fn default_textarea_node(size_chars: Option<usize>, rows: Option<usize>) -> Node {
    let width = size_chars.map(input_width_for_chars).unwrap_or(180.0);
    let rows = rows.unwrap_or(3).max(1) as f32;
    let line_height = font_size_control() * 1.5;
    let content_height = line_height * rows;
    Node {
        width: size_chars.map_or(Val::Auto, |chars| Val::Px(input_width_for_chars(chars))),
        min_width: Val::Px(width),
        min_height: Val::Px(content_height + 20.0),
        height: Val::Px(content_height + 20.0),
        align_items: AlignItems::Start,
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
        position_type: PositionType::Absolute,
        left: Val::Px(0.0),
        top: Val::Px(0.0),
        min_width: Val::Px(0.0),
        ..default()
    }
}

pub(crate) fn input_text_bundle(add_input: &AddInput) -> AddText {
    let preview = if add_input.input_type == InputType::Password && !add_input.value.is_empty() {
        mask_password(&add_input.value)
    } else if add_input.value.is_empty() {
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
        layout: if add_input.input_type == InputType::Textarea {
            TextLayout::new_with_linebreak(LineBreak::WordBoundary)
        } else {
            TextLayout::new_with_no_wrap()
        },
        ..default()
    }
}

pub(crate) fn update_input_text(
    commands: &mut Commands,
    font_resource: &FontResource,
    field: &InputField,
    disabled: bool,
) {
    let Some(text_entity) = field.text_entity else {
        return;
    };

    let display_text = field.edit_state.display_text_string(&field.placeholder);
    let text = if matches!(field.input_type, InputType::Password) {
        if field.value().is_empty() {
            if disabled || display_text.is_placeholder {
                field.placeholder.clone()
            } else {
                String::new()
            }
        } else {
            mask_password(field.value())
        }
    } else if disabled && field.edit_state.preedit().is_some() {
        field.value().to_string()
    } else if disabled {
        if field.value().is_empty() {
            field.placeholder.clone()
        } else {
            field.value().to_string()
        }
    } else {
        display_text.text
    };
    set_plain_text(commands, text_entity, text);

    let Ok(mut entity_commands) = commands.get_entity(text_entity) else {
        return;
    };
    let color = if disabled {
        text_disabled_color()
    } else if display_text.is_placeholder {
        text_placeholder_color()
    } else {
        text_primary_color()
    };
    let text_font = font_resource
        .primary_font
        .clone()
        .map(TextFont::from)
        .unwrap_or_default()
        .with_font_size(font_size_control());
    entity_commands.try_insert((text_font, TextColor(color)));
}

pub fn set_input_value(
    commands: &mut Commands,
    font_resource: &FontResource,
    field: &mut InputField,
    disabled: bool,
    value: impl Into<String>,
) -> bool {
    let value = value.into();
    if field.value() == value && field.edit_state.preedit().is_none() {
        return false;
    }

    field.set_value(value);
    update_input_text(commands, font_resource, field, disabled);
    true
}

pub(crate) fn default_check_input_node() -> Node {
    Node {
        min_width: Val::Px(18.0),
        min_height: Val::Px(18.0),
        width: Val::Px(18.0),
        height: Val::Px(18.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: regular_border(),
        border_radius: control_radius(),
        ..default()
    }
}

pub(crate) fn apply_check_input_shape(node: &mut Node, input_type: InputType) {
    if matches!(input_type, InputType::Radio) {
        node.border_radius = BorderRadius::all(Val::Px(999.0));
    } else {
        node.border_radius = BorderRadius::ZERO;
    }
}

pub(crate) fn default_check_indicator_node(input_type: InputType) -> Node {
    let mut node = Node {
        width: Val::Px(10.0),
        height: Val::Px(10.0),
        ..default()
    };
    if matches!(input_type, InputType::Radio) {
        node.border_radius = BorderRadius::all(Val::Px(999.0));
    } else {
        node.width = Val::Px(9.0);
        node.height = Val::Px(9.0);
    }
    node
}

fn mask_password(value: &str) -> String {
    value.chars().map(|_| '*').collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::InputType;
    use bevy::text::LineBreak;

    #[test]
    fn input_text_bundle_disables_soft_wrap() {
        let add_input = AddInput {
            value: "Long single-line value".to_string(),
            ..Default::default()
        };

        let bundle = input_text_bundle(&add_input);

        assert_eq!(bundle.layout.linebreak, LineBreak::NoWrap);
    }

    #[test]
    fn empty_input_text_bundle_uses_placeholder_color() {
        let add_input = AddInput {
            placeholder: "Hint".to_string(),
            ..Default::default()
        };

        let bundle = input_text_bundle(&add_input);

        assert_eq!(bundle.text, "Hint");
        assert_eq!(bundle.color, text_placeholder_color());
    }

    #[test]
    fn textarea_text_bundle_enables_soft_wrap() {
        let add_input = AddInput {
            input_type: InputType::Textarea,
            value: "Long multi-line value".to_string(),
            ..Default::default()
        };

        let bundle = input_text_bundle(&add_input);

        assert_eq!(bundle.layout.linebreak, LineBreak::WordBoundary);
    }
}
