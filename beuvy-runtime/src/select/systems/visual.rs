use crate::button::sync_button_active_state;
use crate::select::model::{
    SELECT_CHEVRON_CLOSED, SELECT_CHEVRON_OPEN, Select, SelectChevronGlyph,
};
use bevy::prelude::*;

pub(crate) fn sync_select_visual_state(
    mut commands: Commands,
    selects: Query<&Select, Changed<Select>>,
    mut chevrons: Query<&mut Text, With<SelectChevronGlyph>>,
) {
    for select in &selects {
        sync_button_active_state(&mut commands, select.trigger, select.open);

        if let Ok(mut text) = chevrons.get_mut(select.chevron_glyph) {
            text.0 = if select.open {
                SELECT_CHEVRON_OPEN.to_string()
            } else {
                SELECT_CHEVRON_CLOSED.to_string()
            };
        }
    }
}
