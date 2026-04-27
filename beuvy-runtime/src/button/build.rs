use super::{AddButton, Button, ButtonInner, ButtonLabel, DisabledButton, state};
use crate::build_pending::UiBuildPending;
use crate::focus::{UiFocusable, hidden_outline};
use crate::interaction_style::{
    UiDisabled, UiStateStyleSource, pointer_cancel, pointer_drag_end, pointer_release,
};
use crate::style::{
    apply_utility_patch, resolve_classes_with_fallback, root_visual_styles_from_patch,
    text_primary_color, text_visual_styles_from_patch,
};
use crate::text::AddText;
use bevy::prelude::*;
use bevy::text::TextLayout;

const DEFAULT_BUTTON_CLASS: &str = "button-root";
const DEFAULT_BUTTON_LABEL_CLASS: &str = "button-label";

pub(super) fn add_button(mut commands: Commands, query: Query<(Entity, &AddButton)>) {
    for (entity, add_button) in query {
        let visibility = if add_button.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let button_patch = resolve_classes_with_fallback(
            DEFAULT_BUTTON_CLASS,
            add_button.class.as_deref(),
            "button root",
        );
        let mut button_node = default_button_node();
        apply_utility_patch(&mut button_node, &button_patch);
        let root_styles = root_visual_styles_from_patch(&button_patch);

        let label_patch = resolve_classes_with_fallback(
            DEFAULT_BUTTON_LABEL_CLASS,
            add_button
                .label_class
                .as_deref()
                .or(add_button.class.as_deref()),
            "button label",
        );
        let label_styles = text_visual_styles_from_patch(&label_patch);

        let add_button = add_button.clone();
        commands
            .entity(entity)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                entity.insert((
                    Button {
                        name: add_button.name.clone(),
                    },
                    Interaction::None,
                    button_node,
                    BackgroundColor(Color::NONE),
                    UiFocusable,
                    hidden_outline(),
                    visibility,
                ));
                if let Some(styles) = root_styles.clone() {
                    entity.insert(styles);
                }

                let source = entity.id();
                let child = entity.world_scope(|world| {
                    let mut child = world.spawn((
                        ButtonInner,
                        Node::default(),
                        TextLayout::new_with_no_wrap(),
                        build_button_text(&add_button, &label_patch),
                    ));
                    if let Some(styles) = label_styles.clone() {
                        child.insert((styles, UiStateStyleSource(source)));
                    }
                    child.id()
                });

                entity.add_child(child);
                entity.insert(ButtonLabel { entity: child });

                if add_button.disabled {
                    entity.insert((DisabledButton, UiDisabled));
                } else {
                    entity.remove::<DisabledButton>().remove::<UiDisabled>();
                }

                entity
                    .observe(state::button_hover_over)
                    .observe(state::button_hover_out)
                    .observe(state::button_press)
                    .observe(pointer_release)
                    .observe(pointer_cancel)
                    .observe(pointer_drag_end)
                    .observe(state::button_click);

                entity.remove::<AddButton>().remove::<UiBuildPending>();
            });
    }
}

pub fn default_button_node() -> Node {
    Node::default()
}

fn build_button_text(
    add_button: &AddButton,
    label_patch: &crate::utility::UtilityStylePatch,
) -> AddText {
    let base = AddText {
        text: add_button.text.clone(),
        size: label_patch
            .text_size
            .unwrap_or_else(crate::style::font_size_control),
        color: label_patch
            .visual
            .text_color
            .as_deref()
            .and_then(crate::style::resolve_color_value)
            .unwrap_or_else(text_primary_color),
        ..default()
    };

    match (
        add_button.localized_text_format.clone(),
        add_button.localized_text,
    ) {
        (Some(localized_text_format), _) => base.with_localized_format(localized_text_format),
        (None, Some(localized_text)) => base.with_localized(localized_text),
        (None, None) => base,
    }
}
