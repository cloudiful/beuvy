use super::resolve::resolve_runtime_path;
use crate::runtime::state::{
    DeclarativeImageAltBinding, DeclarativeImageSrcBinding, DeclarativeLinkHrefBinding,
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use beuvy_runtime::link::Link;
use bevy::prelude::*;
use bevy::ui::widget::ImageNode;

pub(crate) fn sync_declarative_image_bindings(
    asset_server: Res<AssetServer>,
    values: Res<DeclarativeUiRuntimeValues>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    ref_rects: Res<DeclarativeRefRects>,
    mut images: Query<(
        Entity,
        Option<&DeclarativeImageSrcBinding>,
        Option<&DeclarativeImageAltBinding>,
        Option<&mut ImageNode>,
        Option<&mut Name>,
    )>,
) {
    for (entity, src_binding, alt_binding, image_node, name) in &mut images {
        if let Some(binding) = src_binding
            && let Some(src) = resolve_runtime_path(
                entity,
                &binding.0,
                &parents,
                &local_states,
                &computed,
                &roots,
                &values,
                &ref_rects,
            )
            .and_then(|value| value.text().map(str::to_string))
            && let Some(mut image_node) = image_node
        {
            image_node.image = asset_server.load(src);
        }

        if let Some(binding) = alt_binding
            && let Some(alt) = resolve_runtime_path(
                entity,
                &binding.0,
                &parents,
                &local_states,
                &computed,
                &roots,
                &values,
                &ref_rects,
            )
            .and_then(|value| value.text().map(str::to_string))
            && let Some(mut name) = name
        {
            *name = Name::new(if alt.trim().is_empty() {
                "image".to_string()
            } else {
                alt
            });
        }
    }
}

pub(crate) fn sync_declarative_link_bindings(
    values: Res<DeclarativeUiRuntimeValues>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    ref_rects: Res<DeclarativeRefRects>,
    mut links: Query<(Entity, &DeclarativeLinkHrefBinding, &mut Link)>,
) {
    for (entity, binding, mut link) in &mut links {
        if let Some(href) = resolve_runtime_path(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        )
        .and_then(|value| value.text().map(str::to_string))
        {
            link.href = href;
        }
    }
}
