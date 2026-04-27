#[path = "systems/open.rs"]
mod open;
#[path = "systems/placement.rs"]
mod placement;
#[path = "systems/sync.rs"]
mod sync;
#[path = "systems/visual.rs"]
mod visual;

pub(crate) use open::{close_selects_on_foreign_click, select_option_click, select_trigger_click};
pub(crate) use placement::sync_select_panel_placement;
pub(crate) use sync::{
    sync_select_button_layouts, sync_select_option_indicators, sync_select_semantics,
};
pub(crate) use visual::sync_select_visual_state;

#[cfg(test)]
#[path = "systems/tests.rs"]
mod tests;
