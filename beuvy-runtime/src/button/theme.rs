use crate::style::{button_theme, ui_theme};
use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct ButtonColorTheme {
    pub idle: Color,
    pub hover: Color,
    pub active: Color,
    pub active_hover: Color,
    pub disabled: Color,
    pub idle_text: Color,
    pub hover_text: Color,
    pub active_text: Color,
    pub active_hover_text: Color,
    pub disabled_text: Color,
}

pub fn default_button_theme() -> ButtonColorTheme {
    let theme = button_theme(&ui_theme().buttons);

    ButtonColorTheme {
        idle: theme.background.idle,
        hover: theme.background.hover,
        active: theme.background.active,
        active_hover: theme.background.active_hover,
        disabled: theme.background.disabled,
        idle_text: theme.text.idle,
        hover_text: theme.text.hover,
        active_text: theme.text.active,
        active_hover_text: theme.text.active_hover,
        disabled_text: theme.text.disabled,
    }
}

impl Default for ButtonColorTheme {
    fn default() -> Self {
        default_button_theme()
    }
}

pub fn button_label_color(theme: ButtonColorTheme, disabled: bool) -> Color {
    if disabled {
        theme.disabled_text
    } else {
        theme.idle_text
    }
}

pub(super) fn button_active_state_text(theme: ButtonColorTheme, active: bool) -> Color {
    if active {
        theme.active_text
    } else {
        theme.idle_text
    }
}

pub(super) fn button_hover_text(theme: ButtonColorTheme, active: bool) -> Color {
    if active {
        theme.active_hover_text
    } else {
        theme.hover_text
    }
}
