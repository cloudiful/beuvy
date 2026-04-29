mod build;
mod state;

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinkSet {
    Build,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Link {
    pub name: String,
    pub href: String,
}

#[derive(Component, Debug, Clone, Default)]
pub struct DisabledLink;

#[derive(Component, Debug, Clone, Copy)]
pub struct LinkLabel {
    pub entity: Entity,
}

#[derive(Component, Debug, Clone)]
pub struct AddLink {
    pub name: String,
    pub href: String,
    pub text: String,
    pub class: Option<String>,
    pub label_class: Option<String>,
    pub visible: bool,
    pub disabled: bool,
}

impl Default for AddLink {
    fn default() -> Self {
        Self {
            name: String::new(),
            href: String::new(),
            text: String::new(),
            class: None,
            label_class: None,
            visible: true,
            disabled: false,
        }
    }
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct LinkActivatedMessage {
    pub entity: Entity,
    pub name: String,
    pub href: String,
}

pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<LinkActivatedMessage>()
            .add_systems(
                Update,
                (build::add_link.in_set(LinkSet::Build), state::link_keyboard_activate),
            );
    }
}
