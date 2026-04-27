use super::bindings::{apply_common_bindings_to_entity, conditional_matches};
use super::context::DeclarativeUiBuildContext;
use super::spawn::{spawn_declarative_child_nodes_in_world, sync_conditional_subtree_component};
use super::state::{
    DeclarativeClassBindings, DeclarativeLocalState, DeclarativeNodeId,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiSlot, DeclarativeUiSlots,
};
use super::style::{apply_node_style, insert_runtime_visuals};
use crate::ast::{DeclarativeUiAsset, DeclarativeUiNode};
use crate::value::UiValue;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn load_internal_declarative_ui_shell<'a>(
    asset_server: &AssetServer,
    assets: &'a Assets<DeclarativeUiAsset>,
    handle: &mut Option<Handle<DeclarativeUiAsset>>,
    path: &'static str,
) -> Option<&'a DeclarativeUiAsset> {
    let handle = handle.get_or_insert_with(|| asset_server.load(path));
    assets.get(handle.id())
}

pub fn materialize_declarative_ui_shell_on_entity_in_world(
    entity: &mut EntityWorldMut,
    asset: &DeclarativeUiAsset,
    context: DeclarativeUiBuildContext,
) -> HashMap<String, Entity> {
    let mut slots = Vec::new();
    let context = context.with_merged_local_state(
        asset
            .root_state
            .iter()
            .map(|assignment| {
                (
                    assignment.name.clone(),
                    UiValue::from_literal(&assignment.value),
                )
            })
            .collect::<Vec<_>>(),
    );
    materialize_declarative_ui_shell_on_entity_inner(
        entity,
        asset,
        &asset.root,
        context,
        &mut slots,
    );
    slots.into_iter().collect()
}

pub fn materialize_internal_declarative_ui_shell_on_entity_in_world(
    entity: &mut EntityWorldMut,
    asset: &DeclarativeUiAsset,
    context: DeclarativeUiBuildContext,
    shell_path: &'static str,
) -> DeclarativeUiSlots {
    DeclarativeUiSlots::new(
        shell_path,
        materialize_declarative_ui_shell_on_entity_in_world(entity, asset, context),
    )
}

fn materialize_declarative_ui_shell_on_entity_inner(
    entity: &mut EntityWorldMut,
    asset: &DeclarativeUiAsset,
    node: &DeclarativeUiNode,
    context: DeclarativeUiBuildContext,
    slots: &mut Vec<(String, Entity)>,
) {
    match node {
        DeclarativeUiNode::Container {
            class,
            class_bindings,
            node: style_node,
            style_binding,
            outlet,
            conditional,
            show_expr,
            visual_style,
            state_visual_styles,
            ref_binding,
            event_bindings,
            children,
            ..
        } => {
            if !conditional_matches(conditional, &context) {
                return;
            }
            entity.insert(DeclarativeNodeId(node.node_id().to_string()));
            entity.insert(DeclarativeRootViewModel(context.root().clone()));
            entity.insert((
                apply_node_style(Node::default(), style_node),
                Visibility::Visible,
            ));
            if !asset.root_state.is_empty() && !entity.contains::<DeclarativeLocalState>() {
                entity.insert(DeclarativeLocalState(context.local_state().clone()));
            }
            if !asset.root_computed.is_empty()
                && !entity.contains::<DeclarativeRootComputedLocals>()
            {
                entity.insert(DeclarativeRootComputedLocals::from(
                    asset.root_computed.as_slice(),
                ));
            }
            if let Some(outlet) = outlet {
                entity.insert(DeclarativeUiSlot);
                slots.push((outlet.clone(), entity.id()));
            }
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                None,
                None,
                None,
                ref_binding.as_ref(),
                style_binding.as_ref(),
                event_bindings,
                &context,
            );
            insert_runtime_visuals(entity, visual_style, state_visual_styles);
            if !class_bindings.is_empty() {
                entity.insert(DeclarativeClassBindings {
                    base_class: class.clone(),
                    bindings: class_bindings.clone(),
                    resolved_class: String::new(),
                });
            }
            sync_conditional_subtree_component(entity, node.node_id(), children, &context, true);
            entity.with_related_entities::<ChildOf>(|parent| {
                spawn_declarative_child_nodes_in_world(
                    parent,
                    asset,
                    children,
                    context.clone(),
                    Some(slots),
                    false,
                    true,
                );
            });
        }
        _ => panic!("internal declarative shell root supports only container nodes"),
    }
}
