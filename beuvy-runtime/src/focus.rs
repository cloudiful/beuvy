use crate::button::Button;
use crate::style::{ui_focus_outline_color, ui_focus_outline_hover_color, ui_hover_outline_color};
use bevy::prelude::*;
use bevy::ui::Val::Px;

pub struct FocusableUiPlugin;

#[derive(Component, Debug, Clone, Copy)]
pub struct UiFocusable;

#[derive(Component, Debug, Clone, Copy)]
pub struct UiFocusOutlineOnFocusOnly;

#[derive(Component, Debug, Clone, Copy)]
pub struct UiHovered;

#[derive(Component, Debug, Clone, Copy)]
pub struct UiFocused;

#[derive(Component, Debug, Clone, Copy)]
pub struct UiPressed;

pub const UI_FOCUS_OUTLINE_WIDTH: f32 = 3.0;
pub const UI_FOCUS_OUTLINE_OFFSET: f32 = 2.0;

impl Plugin for FocusableUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                sync_focusable_outlines_on_state_change,
                sync_focusable_outlines_on_state_removal,
            ),
        );
    }
}

pub fn hidden_outline() -> Outline {
    Outline::new(
        Px(UI_FOCUS_OUTLINE_WIDTH),
        Px(UI_FOCUS_OUTLINE_OFFSET),
        Color::NONE,
    )
}

fn sync_focusable_outlines_on_state_change(
    mut focusables: Query<
        (
            Has<UiHovered>,
            Has<UiFocused>,
            Has<UiFocusOutlineOnFocusOnly>,
            &mut Outline,
        ),
        (
            With<UiFocusable>,
            Without<Button>,
            Or<(Added<UiFocusable>, Changed<UiHovered>, Changed<UiFocused>)>,
        ),
    >,
) {
    for (hovered, focused, focus_only, mut outline) in &mut focusables {
        outline.color = outline_color(hovered, focused, focus_only);
    }
}

fn sync_focusable_outlines_on_state_removal(
    mut removed_hovered: RemovedComponents<UiHovered>,
    mut removed_focused: RemovedComponents<UiFocused>,
    mut focusables: Query<
        (
            Has<UiHovered>,
            Has<UiFocused>,
            Has<UiFocusOutlineOnFocusOnly>,
            &mut Outline,
        ),
        (With<UiFocusable>, Without<Button>),
    >,
) {
    for entity in removed_hovered.read() {
        let Ok((hovered, focused, focus_only, mut outline)) = focusables.get_mut(entity) else {
            continue;
        };
        outline.color = outline_color(hovered, focused, focus_only);
    }

    for entity in removed_focused.read() {
        let Ok((hovered, focused, focus_only, mut outline)) = focusables.get_mut(entity) else {
            continue;
        };
        outline.color = outline_color(hovered, focused, focus_only);
    }
}

pub fn outline_color(hovered: bool, focused: bool, focus_only: bool) -> Color {
    if focus_only {
        return if focused {
            ui_focus_outline_color()
        } else {
            Color::NONE
        };
    }

    match (hovered, focused) {
        (true, true) => ui_focus_outline_hover_color(),
        (true, false) => ui_hover_outline_color(),
        (false, true) => ui_focus_outline_color(),
        (false, false) => Color::NONE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_only_outline_ignores_hover_state() {
        assert_eq!(outline_color(true, false, true), Color::NONE);
        assert_eq!(
            outline_color(true, true, true),
            ui_focus_outline_color()
        );
    }
}
