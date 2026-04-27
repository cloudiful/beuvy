mod model;
mod pointer;
mod resolve;
mod transition;

use bevy::prelude::*;

pub use model::{UiDisabled, UiStateStyleSource, UiStateVisualStyles};
pub use pointer::{
    pointer_cancel, pointer_drag_end, pointer_hover_out, pointer_hover_over, pointer_press,
    pointer_release,
};

pub struct UiStateStylePlugin;

impl Plugin for UiStateStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                resolve::capture_state_visual_snapshots,
                resolve::resolve_state_visual_styles,
                transition::animate_state_visual_styles,
            )
                .chain(),
        );
    }
}

#[cfg(test)]
mod tests;
