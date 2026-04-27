use crate::utility::UtilityVisualStylePatch;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UiStateVisualStyles {
    pub base: UtilityVisualStylePatch,
    pub hover: UtilityVisualStylePatch,
    pub active: UtilityVisualStylePatch,
    pub focus: UtilityVisualStylePatch,
    pub disabled: UtilityVisualStylePatch,
}

impl UiStateVisualStyles {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct UiStateStyleSource(pub Entity);

#[derive(Component, Debug, Clone, Copy)]
pub struct UiDisabled;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct UiStateVisualSnapshot {
    pub(super) background: Option<Color>,
    pub(super) border: Option<Color>,
    pub(super) text: Option<Color>,
    pub(super) outline_width: Option<Val>,
    pub(super) outline_offset: Option<Val>,
    pub(super) outline_color: Option<Color>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct UiStateVisualTarget {
    pub(super) background: Option<Color>,
    pub(super) border: Option<Color>,
    pub(super) text: Option<Color>,
    pub(super) outline_width: Option<Val>,
    pub(super) outline_color: Option<Color>,
}
