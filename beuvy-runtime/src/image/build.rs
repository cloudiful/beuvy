use super::AddImage;
use crate::build_pending::UiBuildPending;
use crate::style::{apply_utility_patch, resolve_classes_with_fallback};
use bevy::prelude::*;

const DEFAULT_IMAGE_CLASS: &str = "image-root";

pub(super) fn add_image(
    mut commands: Commands,
    query: Query<(Entity, &AddImage)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, add_image) in &query {
        let root_patch = resolve_classes_with_fallback(
            DEFAULT_IMAGE_CLASS,
            add_image.class.as_deref(),
            "image root",
        );
        let mut node = Node::default();
        apply_utility_patch(&mut node, &root_patch);

        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };
        entity_commands
            .insert((
                Name::new(add_image.alt.clone().if_empty_then("image")),
                node,
                ImageNode::new(asset_server.load(add_image.src.clone())),
            ))
            .remove::<AddImage>()
            .remove::<UiBuildPending>();
    }
}

trait IfEmptyThen {
    fn if_empty_then(self, fallback: &str) -> String;
}

impl IfEmptyThen for String {
    fn if_empty_then(self, fallback: &str) -> String {
        if self.trim().is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}
