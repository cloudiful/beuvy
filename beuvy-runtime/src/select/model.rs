use crate::button::ButtonLabel;
use crate::text::{
    LocalizedTextFormat, set_localized_text, set_localized_text_format, set_plain_text,
};
use bevy::prelude::*;
use bevy_localization::{Localization, TextKey};

pub const SELECT_CHEVRON_CLOSED: &str = "⌄";
pub const SELECT_CHEVRON_OPEN: &str = "⌃";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectOptionState {
    pub entity: Entity,
    pub value: String,
    pub text: String,
    pub localized_text: Option<TextKey>,
    pub localized_text_format: Option<LocalizedTextFormat>,
    pub disabled: bool,
}

#[derive(Component, Debug, Clone)]
pub struct Select {
    pub name: String,
    pub value: String,
    pub initial_value: String,
    pub options: Vec<SelectOptionState>,
    pub panel: Entity,
    pub trigger: Entity,
    pub chevron_glyph: Entity,
    pub open: bool,
    pub disabled: bool,
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct SelectValueChangedMessage {
    pub entity: Entity,
    pub name: String,
    pub value: String,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct SelectTrigger {
    pub(crate) select: Entity,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct SelectOptionButton {
    pub(crate) select: Entity,
    pub(crate) value: String,
}

#[derive(Component, Debug, Clone)]
pub struct SelectPanel;

#[derive(Component, Debug, Clone)]
pub(crate) struct SelectOptionIndicator;

#[derive(Component, Debug, Clone)]
pub(crate) struct SelectChevron;

#[derive(Component, Debug, Clone)]
pub(crate) struct SelectChevronGlyph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddSelectOption {
    pub name: String,
    pub value: String,
    pub text: String,
    pub localized_text: Option<TextKey>,
    pub localized_text_format: Option<LocalizedTextFormat>,
    pub disabled: bool,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct AddSelect {
    pub name: String,
    pub value: String,
    pub options: Vec<AddSelectOption>,
    pub class: Option<String>,
    pub trigger_class: Option<String>,
    pub label_class: Option<String>,
    pub panel_class: Option<String>,
    pub chevron_class: Option<String>,
    pub option_class: Option<String>,
    pub indicator_class: Option<String>,
    pub disabled: bool,
}

impl Default for AddSelect {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            options: Vec::new(),
            class: None,
            trigger_class: None,
            label_class: None,
            panel_class: None,
            chevron_class: None,
            option_class: None,
            indicator_class: None,
            disabled: false,
        }
    }
}

pub fn default_select_node() -> Node {
    Node::default()
}

pub fn selected_option(select: &Select) -> Option<&SelectOptionState> {
    select
        .options
        .iter()
        .find(|option| option.value == select.value)
}

pub fn trigger_label_entity(
    trigger_labels: &Query<&ButtonLabel>,
    select: &Select,
) -> Option<Entity> {
    trigger_labels
        .get(select.trigger)
        .ok()
        .map(|label| label.entity)
}

pub fn sync_select_label(
    commands: &mut Commands,
    localization: Option<&Localization>,
    label_entity: Entity,
    option: Option<&SelectOptionState>,
    fallback_value: &str,
) {
    let Some(option) = option else {
        set_plain_text(commands, label_entity, fallback_value);
        return;
    };

    if let (Some(localization), Some(format)) = (localization, option.localized_text_format.clone())
    {
        set_localized_text_format(commands, label_entity, localization, format);
        return;
    }
    if let (Some(localization), Some(key)) = (localization, option.localized_text) {
        set_localized_text(commands, label_entity, localization, key);
        return;
    }
    set_plain_text(commands, label_entity, option.text.clone());
}
