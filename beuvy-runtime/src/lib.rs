#![allow(clippy::default_constructed_unit_structs, clippy::type_complexity)]
#![doc = r#"
`beuvy-runtime` is a compact UI kit for Bevy.

It provides a small set of reusable controls and utility-class parsing for
declarative styling. The stable v1 surface is:

- [`UiKitPlugin`]
- [`text::AddText`]
- [`button::AddButton`]
- [`input::AddInput`]
- [`utility::parse_utility_classes`]

The crate also exposes a few lower-level modules used by GPMO. Those remain
public for compatibility, but they are intentionally hidden from the main docs
and should be treated as less stable.
"#]

#[doc(hidden)]
pub mod backdrop;
#[doc(hidden)]
pub mod build_pending;
pub mod button;
#[doc(hidden)]
pub mod focus;
#[doc(hidden)]
pub mod form_item;
pub mod input;
#[path = "state_style/mod.rs"]
#[doc(hidden)]
pub mod interaction_style;
pub mod scroll;
pub mod select;
#[doc(hidden)]
pub mod style;
pub mod stylesheet;
pub mod text;
mod theme_config;
pub mod utility;

pub use button::AddButton;
pub use input::AddInput;
pub use interaction_style as state_style;
pub use scroll::{MouseWheelScroll, scroll_container_node};
pub use select::{AddSelect, AddSelectOption};
pub use select::{
    Select, SelectPanel, SelectValueChangedMessage, default_select_node, selected_option,
    sync_select_label, trigger_label_entity,
};
pub use stylesheet::{
    RuntimeStyleSource, StyleSheetError, UiStyleSheet, compose_style_sheet, default_style_sheet,
    font_size_for_tag as stylesheet_font_size_for_tag, parse_style_classes_with_sheet,
    parse_style_sheet, replace_runtime_style_source, runtime_style_sheet, runtime_style_source,
};
pub use text::AddText;
pub use utility::parse_utility_classes;

use bevy::prelude::*;

/// Installs the core UI kit systems: text, buttons, inputs, focus handling,
/// and interaction-driven state styling.
pub struct UiKitPlugin;

impl Plugin for UiKitPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(backdrop::BackdropPlugin)
            .add_plugins(focus::FocusableUiPlugin)
            .add_plugins(text::TextPlugin)
            .add_plugins(form_item::FormItem::default())
            .add_plugins(button::ButtonPlugin::default())
            .add_plugins(input::InputPlugin)
            .add_plugins(select::SelectPlugin)
            .add_plugins(interaction_style::UiStateStylePlugin)
            .register_required_components::<AddText, build_pending::UiBuildPending>()
            .register_required_components::<AddButton, build_pending::UiBuildPending>()
            .register_required_components::<AddInput, build_pending::UiBuildPending>()
            .register_required_components::<select::AddSelect, build_pending::UiBuildPending>()
            .register_required_components::<form_item::AddFormItem, build_pending::UiBuildPending>(
            );
    }
}
