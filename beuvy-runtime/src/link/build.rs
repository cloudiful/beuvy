use super::{AddLink, DisabledLink, Link, LinkLabel, state};
use crate::build_pending::UiBuildPending;
use crate::focus::{UiFocusOutlineOnFocusOnly, UiFocusable, hidden_outline};
use crate::interaction_style::UiDisabled;
use crate::style::{
    apply_utility_patch, resolve_classes_with_fallback, root_visual_styles_from_patch,
    text_primary_color,
};
use crate::text::AddText;
use bevy::picking::Pickable;
use bevy::prelude::*;

const DEFAULT_LINK_CLASS: &str = "link-root";
const DEFAULT_LINK_LABEL_CLASS: &str = "link-label";

pub(super) fn add_link(mut commands: Commands, query: Query<(Entity, &AddLink)>) {
    for (entity, add_link) in &query {
        let add_link = add_link.clone();
        let root_patch = resolve_classes_with_fallback(
            DEFAULT_LINK_CLASS,
            add_link.class.as_deref(),
            "link root",
        );
        let label_patch = resolve_classes_with_fallback(
            DEFAULT_LINK_LABEL_CLASS,
            add_link.label_class.as_deref(),
            "link label",
        );
        let root_styles = root_visual_styles_from_patch(&root_patch);

        let mut node = Node::default();
        apply_utility_patch(&mut node, &root_patch);

        commands
            .entity(entity)
            .queue_silenced(move |mut entity_commands: EntityWorldMut| {
                entity_commands.insert((
                    Link {
                        name: add_link.name.clone(),
                        href: add_link.href.clone(),
                    },
                    Name::new(add_link.name.clone()),
                    node,
                    Visibility::Visible,
                    Pickable::default(),
                    hidden_outline(),
                    UiFocusOutlineOnFocusOnly,
                ));

                if let Some(styles) = root_styles.clone() {
                    entity_commands.insert(styles);
                }

                let label = entity_commands.world_scope(|world| {
                    let mut label_node = Node::default();
                    apply_utility_patch(&mut label_node, &label_patch);
                    world
                        .spawn((
                            label_node,
                            AddText {
                                text: add_link.text.clone(),
                                size: label_patch.text_size.unwrap_or(15.0),
                                color: label_patch
                                    .visual
                                    .text_color
                                    .as_deref()
                                    .and_then(crate::style::resolve_color_value)
                                    .unwrap_or_else(text_primary_color),
                                ..default()
                            },
                        ))
                        .id()
                });
                entity_commands.add_child(label);
                entity_commands.insert(LinkLabel { entity: label });

                if add_link.disabled {
                    entity_commands.insert((DisabledLink, UiDisabled));
                } else {
                    entity_commands.insert(UiFocusable);
                }

                entity_commands
                    .observe(state::link_click)
                    .remove::<AddLink>()
                    .remove::<UiBuildPending>();
            });
    }
}
