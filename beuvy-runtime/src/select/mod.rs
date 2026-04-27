mod build;
mod model;
mod systems;

pub use model::{
    AddSelect, AddSelectOption, Select, SelectOptionState, SelectPanel, SelectValueChangedMessage,
    default_select_node, selected_option, sync_select_label, trigger_label_entity,
};

use crate::button::ButtonSet;
use crate::form_item::FormItemSet;
use bevy::prelude::*;
use build::add_select;
use systems::{
    close_selects_on_foreign_click, sync_select_button_layouts, sync_select_option_indicators,
    sync_select_panel_placement, sync_select_semantics, sync_select_visual_state,
};

pub struct SelectPlugin;

impl Plugin for SelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Pointer<Click>>()
            .add_message::<SelectValueChangedMessage>()
            .add_systems(Update, add_select.after(FormItemSet::Build))
            .add_systems(
                Update,
                (
                    close_selects_on_foreign_click,
                    sync_select_visual_state,
                    sync_select_semantics,
                    sync_select_panel_placement,
                    sync_select_button_layouts,
                    sync_select_option_indicators,
                )
                    .after(ButtonSet::Build),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_plugin_schedule_initializes() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(SelectPlugin);
        app.update();
    }
}
