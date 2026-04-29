use super::bindings::{
    apply_common_bindings_to_entity, conditional_chain_matches,
    resolve_declarative_button_event_bindings,
};
use super::context::DeclarativeUiBuildContext;
use super::controls::{
    build_declarative_button, build_declarative_input, build_declarative_select,
};
use super::state::{
    DeclarativeClassBindings, DeclarativeConditionalChainState, DeclarativeConditionalSubtree,
    DeclarativeLabelForTarget, DeclarativeLabelNode, DeclarativeLocalState, DeclarativeNodeId,
    DeclarativeOnClickAssignment, DeclarativeRootComputedLocals, DeclarativeRootViewModel,
    DeclarativeSelectTextBindings, DeclarativeUiSlot,
};
use super::style::{DeclarativeEntityInsert, apply_node_style, insert_runtime_visuals};
use super::text::{build_add_text, content_has_dynamic_bindings};
use crate::ast::*;
use crate::value::UiValue;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn spawn_declarative_ui_tree_collect_slots(
    parent: &mut ChildSpawnerCommands,
    asset: &DeclarativeUiAsset,
    context: DeclarativeUiBuildContext,
) -> (Entity, HashMap<String, Entity>) {
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
    let root = spawn_declarative_ui_tree_inner(
        parent,
        asset,
        &asset.root,
        context,
        Some(&mut slots),
        true,
        true,
    );
    (root, slots.into_iter().collect())
}

pub fn spawn_declarative_ui_tree_collect_slots_in_world(
    parent: &mut RelatedSpawner<ChildOf>,
    asset: &DeclarativeUiAsset,
    context: DeclarativeUiBuildContext,
) -> (Entity, HashMap<String, Entity>) {
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
    let root = spawn_declarative_ui_tree_inner_in_world(
        parent,
        asset,
        &asset.root,
        context,
        Some(&mut slots),
        true,
        true,
    );
    (root, slots.into_iter().collect())
}

fn spawn_declarative_ui_tree_inner(
    parent: &mut ChildSpawnerCommands,
    asset: &DeclarativeUiAsset,
    node: &DeclarativeUiNode,
    context: DeclarativeUiBuildContext,
    mut slots: Option<&mut Vec<(String, Entity)>>,
    is_root: bool,
    supports_runtime_conditional_subtree_rebuild: bool,
) -> Entity {
    if !node_matches_condition(node, &context) {
        return parent.spawn_empty().id();
    }

    let mut entity = parent.spawn_empty();
    build_spawned_node(
        &mut entity,
        asset,
        node,
        &context,
        is_root,
        supports_runtime_conditional_subtree_rebuild,
    );
    let id = entity.id();
    if let Some(outlet) = outlet_name(node)
        && let Some(slots) = slots.as_deref_mut()
    {
        slots.push((outlet.to_string(), id));
    }
    if let Some(children) = node_children(node) {
        parent.commands().entity(id).with_children(|child_parent| {
            spawn_declarative_child_nodes(
                child_parent,
                asset,
                children,
                context.clone(),
                slots.as_deref_mut(),
                false,
                supports_runtime_conditional_subtree_rebuild,
            );
        });
    }
    id
}

fn spawn_declarative_ui_tree_inner_in_world(
    parent: &mut RelatedSpawner<ChildOf>,
    asset: &DeclarativeUiAsset,
    node: &DeclarativeUiNode,
    context: DeclarativeUiBuildContext,
    mut slots: Option<&mut Vec<(String, Entity)>>,
    is_root: bool,
    supports_runtime_conditional_subtree_rebuild: bool,
) -> Entity {
    if !node_matches_condition(node, &context) {
        return parent.spawn_empty().id();
    }

    let mut entity = parent.spawn_empty();
    build_spawned_node(
        &mut entity,
        asset,
        node,
        &context,
        is_root,
        supports_runtime_conditional_subtree_rebuild,
    );
    let id = entity.id();
    if let Some(outlet) = outlet_name(node)
        && let Some(slots) = slots.as_deref_mut()
    {
        slots.push((outlet.to_string(), id));
    }
    if let Some(children) = node_children(node) {
        entity.with_related_entities::<ChildOf>(|child_parent| {
            spawn_declarative_child_nodes_in_world(
                child_parent,
                asset,
                children,
                context.clone(),
                slots.as_deref_mut(),
                false,
                supports_runtime_conditional_subtree_rebuild,
            );
        });
    }
    id
}

fn spawn_declarative_child_nodes(
    parent: &mut ChildSpawnerCommands,
    asset: &DeclarativeUiAsset,
    children: &[DeclarativeUiNode],
    context: DeclarativeUiBuildContext,
    mut slots: Option<&mut Vec<(String, Entity)>>,
    is_root: bool,
    supports_runtime_conditional_subtree_rebuild: bool,
) {
    let mut previous_branch_matched = None;
    for child in children {
        match child {
            DeclarativeUiNode::Template {
                for_each, children, ..
            } => {
                previous_branch_matched = None;
                for (index, item) in context
                    .template_items(&for_each.source)
                    .iter()
                    .cloned()
                    .enumerate()
                {
                    spawn_declarative_child_nodes(
                        parent,
                        asset,
                        children,
                        context.with_template_iteration(
                            item,
                            &for_each.item_alias,
                            for_each.index_alias.as_deref(),
                            index,
                        ),
                        slots.as_deref_mut(),
                        false,
                        false,
                    );
                }
            }
            _ => {
                if !child_matches_conditional_chain(child, &context, &mut previous_branch_matched) {
                    continue;
                }
                spawn_declarative_ui_tree_inner(
                    parent,
                    asset,
                    child,
                    context.clone(),
                    slots.as_deref_mut(),
                    is_root,
                    supports_runtime_conditional_subtree_rebuild,
                );
            }
        }
    }
}

pub(crate) fn spawn_declarative_child_nodes_in_world(
    parent: &mut RelatedSpawner<ChildOf>,
    asset: &DeclarativeUiAsset,
    children: &[DeclarativeUiNode],
    context: DeclarativeUiBuildContext,
    mut slots: Option<&mut Vec<(String, Entity)>>,
    is_root: bool,
    supports_runtime_conditional_subtree_rebuild: bool,
) {
    let mut previous_branch_matched = None;
    for child in children {
        match child {
            DeclarativeUiNode::Template {
                for_each, children, ..
            } => {
                previous_branch_matched = None;
                for (index, item) in context
                    .template_items(&for_each.source)
                    .iter()
                    .cloned()
                    .enumerate()
                {
                    spawn_declarative_child_nodes_in_world(
                        parent,
                        asset,
                        children,
                        context.with_template_iteration(
                            item,
                            &for_each.item_alias,
                            for_each.index_alias.as_deref(),
                            index,
                        ),
                        slots.as_deref_mut(),
                        false,
                        false,
                    );
                }
            }
            _ => {
                if !child_matches_conditional_chain(child, &context, &mut previous_branch_matched) {
                    continue;
                }
                spawn_declarative_ui_tree_inner_in_world(
                    parent,
                    asset,
                    child,
                    context.clone(),
                    slots.as_deref_mut(),
                    is_root,
                    supports_runtime_conditional_subtree_rebuild,
                );
            }
        }
    }
}

fn build_spawned_node(
    entity: &mut impl DeclarativeEntityInsert,
    asset: &DeclarativeUiAsset,
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
    is_root: bool,
    supports_runtime_conditional_subtree_rebuild: bool,
) {
    insert_context_state(entity, asset, context, is_root);
    entity.insert_component(DeclarativeNodeId(node.node_id().to_string()));
    match node {
        DeclarativeUiNode::Container {
            node_id,
            class,
            class_bindings,
            node,
            style_binding,
            outlet,
            show_expr,
            visual_style,
            state_visual_styles,
            ref_binding,
            event_bindings,
            children,
            ..
        } => {
            entity.insert_component((apply_node_style(Node::default(), node), Visibility::Visible));
            if outlet.is_some() {
                entity.insert_component(DeclarativeUiSlot);
            }
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                None,
                None,
                None,
                None,
                ref_binding.as_ref(),
                style_binding.as_ref(),
                event_bindings,
                context,
            );
            insert_runtime_visuals(entity, visual_style, state_visual_styles);
            insert_class_bindings(entity, class, class_bindings);
            insert_conditional_subtree_component(
                entity,
                node_id,
                children,
                context,
                supports_runtime_conditional_subtree_rebuild,
            );
        }
        DeclarativeUiNode::Text {
            class,
            class_bindings,
            content,
            show_expr,
            ref_binding,
            style,
            ..
        } => {
            let (add_text, binding) = build_add_text(content, style, context);
            entity.insert_component(add_text);
            insert_runtime_visuals(entity, &style.visual_style, &style.state_visual_styles);
            if let Some(binding) = binding {
                entity.insert_component(binding);
            }
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                None,
                None,
                None,
                None,
                ref_binding.as_ref(),
                None,
                &[],
                context,
            );
            insert_class_bindings(entity, class, class_bindings);
        }
        DeclarativeUiNode::Label {
            class,
            class_bindings,
            content,
            show_expr,
            ref_binding,
            style,
            for_target,
            children,
            ..
        } => {
            let (add_text, binding) = build_add_text(content, style, context);
            entity.insert_component((
                add_text,
                apply_node_style(Node::default(), &DeclarativeNodeStyle::default()),
            ));
            entity.insert_component(DeclarativeLabelNode);
            insert_runtime_visuals(entity, &style.visual_style, &style.state_visual_styles);
            if let Some(binding) = binding {
                entity.insert_component(binding);
            }
            if let Some(for_target) = for_target {
                entity.insert_component(DeclarativeLabelForTarget(for_target.clone()));
            }
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                None,
                None,
                None,
                None,
                ref_binding.as_ref(),
                None,
                &[],
                context,
            );
            insert_class_bindings(entity, class, class_bindings);
            insert_conditional_subtree_component(
                entity,
                node.node_id(),
                children,
                context,
                supports_runtime_conditional_subtree_rebuild,
            );
        }
        DeclarativeUiNode::Button {
            content,
            onclick,
            ref_binding,
            class,
            class_bindings,
            style_binding,
            ..
        } => {
            entity.insert_component(build_declarative_button(node, context));
            if content_has_dynamic_bindings(content) {
                entity.insert_component(super::state::DeclarativeTextBinding(content.clone()));
            }
            if let Some(bindings) = resolve_declarative_button_event_bindings(node, context) {
                entity.insert_component(super::state::DeclarativeEventBindings(bindings));
            }
            if let Some(DeclarativeOnClick::Assign { name, value }) = onclick {
                entity.insert_component(DeclarativeOnClickAssignment {
                    name: name.clone(),
                    value: value.clone(),
                });
            }
            apply_common_bindings_to_entity(
                entity,
                None,
                None,
                None,
                None,
                None,
                ref_binding.as_ref(),
                style_binding.as_ref(),
                &[],
                context,
            );
            insert_class_bindings(entity, class, class_bindings);
        }
        DeclarativeUiNode::Input {
            show_expr,
            disabled_expr,
            value_binding,
            model_binding,
            checked_binding,
            ref_binding,
            event_bindings,
            class,
            class_bindings,
            style_binding,
            ..
        } => {
            entity.insert_component(build_declarative_input(node, context));
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                disabled_expr.as_ref(),
                value_binding.as_deref(),
                model_binding.as_deref(),
                checked_binding.as_deref(),
                ref_binding.as_ref(),
                style_binding.as_ref(),
                event_bindings,
                context,
            );
            insert_class_bindings(entity, class, class_bindings);
        }
        DeclarativeUiNode::Select {
            options,
            show_expr,
            disabled_expr,
            value_binding,
            model_binding,
            ref_binding,
            event_bindings,
            class,
            class_bindings,
            style_binding,
            ..
        } => {
            entity.insert_component(build_declarative_select(node, context));
            if options
                .iter()
                .any(|option| content_has_dynamic_bindings(&option.content))
            {
                entity.insert_component(DeclarativeSelectTextBindings(options.clone()));
            }
            apply_common_bindings_to_entity(
                entity,
                show_expr.as_ref(),
                disabled_expr.as_ref(),
                value_binding.as_deref(),
                model_binding.as_deref(),
                None,
                ref_binding.as_ref(),
                style_binding.as_ref(),
                event_bindings,
                context,
            );
            insert_class_bindings(entity, class, class_bindings);
        }
        DeclarativeUiNode::Template { .. } => {
            panic!("template nodes are expanded by the parent declarative builder")
        }
    }
}

pub fn rematerialize_declarative_container_children_in_world(
    entity: &mut EntityWorldMut,
    asset: &DeclarativeUiAsset,
    node: &DeclarativeUiNode,
    context: DeclarativeUiBuildContext,
) {
    let DeclarativeUiNode::Container {
        node_id, children, ..
    } = node
    else {
        panic!("runtime subtree rematerialization requires a container node");
    };

    entity.despawn_related::<Children>();
    entity.with_related_entities::<ChildOf>(|parent| {
        spawn_declarative_child_nodes_in_world(
            parent,
            asset,
            children,
            context.clone(),
            None,
            false,
            true,
        );
    });
    sync_conditional_subtree_component(entity, node_id, children, &context, true);
}

fn insert_class_bindings(
    entity: &mut impl DeclarativeEntityInsert,
    base_class: &str,
    bindings: &[DeclarativeClassBinding],
) {
    if bindings.is_empty() && base_class.trim().is_empty() {
        return;
    }
    entity.insert_component(DeclarativeClassBindings {
        base_class: base_class.to_string(),
        bindings: bindings.to_vec(),
        resolved_class: String::new(),
    });
}

fn insert_context_state(
    entity: &mut impl DeclarativeEntityInsert,
    asset: &DeclarativeUiAsset,
    context: &DeclarativeUiBuildContext,
    is_root: bool,
) {
    if is_root && !asset.root_state.is_empty() {
        entity.insert_component(DeclarativeLocalState(
            asset
                .root_state
                .iter()
                .map(|assignment| {
                    (
                        assignment.name.clone(),
                        UiValue::from_literal(&assignment.value),
                    )
                })
                .collect(),
        ));
    }
    if !asset.root_computed.is_empty() {
        entity.insert_component(DeclarativeRootComputedLocals::from(
            asset.root_computed.as_slice(),
        ));
    }
    entity.insert_component(DeclarativeRootViewModel(context.root().clone()));
    if is_root {
    } else {
        let root_state_names = asset
            .root_state
            .iter()
            .map(|assignment| assignment.name.as_str())
            .collect::<std::collections::HashSet<_>>();
        let local_state = context
            .local_state()
            .iter()
            .filter(|(name, _)| !root_state_names.contains(name.as_str()))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<std::collections::HashMap<_, _>>();
        if !local_state.is_empty() {
            entity.insert_component(DeclarativeLocalState(local_state));
        }
    }
}

fn insert_conditional_subtree_component(
    entity: &mut impl DeclarativeEntityInsert,
    container_node_id: &str,
    children: &[DeclarativeUiNode],
    context: &DeclarativeUiBuildContext,
    supports_runtime_conditional_subtree_rebuild: bool,
) {
    let Some(component) = conditional_subtree_component(
        container_node_id,
        children,
        context,
        supports_runtime_conditional_subtree_rebuild,
    ) else {
        return;
    };
    entity.insert_component(component);
}

pub(crate) fn sync_conditional_subtree_component(
    entity: &mut EntityWorldMut,
    container_node_id: &str,
    children: &[DeclarativeUiNode],
    context: &DeclarativeUiBuildContext,
    supports_runtime_conditional_subtree_rebuild: bool,
) {
    let component = conditional_subtree_component(
        container_node_id,
        children,
        context,
        supports_runtime_conditional_subtree_rebuild,
    );
    if let Some(component) = component {
        entity.insert(component);
    } else {
        entity.remove::<DeclarativeConditionalSubtree>();
    }
}

fn conditional_subtree_component(
    container_node_id: &str,
    children: &[DeclarativeUiNode],
    context: &DeclarativeUiBuildContext,
    supports_runtime_conditional_subtree_rebuild: bool,
) -> Option<DeclarativeConditionalSubtree> {
    if !supports_runtime_conditional_subtree_rebuild {
        return None;
    }

    let chains = direct_conditional_chain_states(children, context);
    if chains.is_empty() {
        return None;
    }

    Some(DeclarativeConditionalSubtree {
        container_node_id: container_node_id.to_string(),
        chains,
    })
}

pub fn direct_conditional_chain_states(
    children: &[DeclarativeUiNode],
    context: &DeclarativeUiBuildContext,
) -> Vec<DeclarativeConditionalChainState> {
    let mut chains = Vec::new();
    let mut index = 0;
    while index < children.len() {
        match node_conditional(&children[index]) {
            Some(DeclarativeConditional::Always) | None => {
                index += 1;
            }
            Some(DeclarativeConditional::If(_)) => {
                let start_index = index;
                let mut end_index = index;
                while end_index + 1 < children.len() {
                    match node_conditional(&children[end_index + 1]) {
                        Some(DeclarativeConditional::ElseIf(_))
                        | Some(DeclarativeConditional::Else) => {
                            end_index += 1;
                        }
                        Some(DeclarativeConditional::Always)
                        | Some(DeclarativeConditional::If(_))
                        | None => break,
                    }
                }

                let mut previous_branch_matched = None;
                let mut active_branch_index = None;
                for (branch_offset, child) in children[start_index..=end_index].iter().enumerate() {
                    if child_matches_conditional_chain(child, context, &mut previous_branch_matched)
                    {
                        active_branch_index = Some(branch_offset);
                        break;
                    }
                }

                chains.push(DeclarativeConditionalChainState {
                    start_index,
                    end_index,
                    active_branch_index,
                });
                index = end_index + 1;
            }
            Some(DeclarativeConditional::ElseIf(_)) | Some(DeclarativeConditional::Else) => {
                index += 1;
            }
        }
    }

    chains
}

fn node_matches_condition(node: &DeclarativeUiNode, context: &DeclarativeUiBuildContext) -> bool {
    match node {
        DeclarativeUiNode::Container { conditional, .. }
        | DeclarativeUiNode::Text { conditional, .. }
        | DeclarativeUiNode::Label { conditional, .. }
        | DeclarativeUiNode::Button { conditional, .. }
        | DeclarativeUiNode::Input { conditional, .. }
        | DeclarativeUiNode::Select { conditional, .. } => {
            super::bindings::conditional_matches(conditional, context)
        }
        DeclarativeUiNode::Template { .. } => true,
    }
}

fn child_matches_conditional_chain(
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
    previous_branch_matched: &mut Option<bool>,
) -> bool {
    match node {
        DeclarativeUiNode::Container { conditional, .. }
        | DeclarativeUiNode::Text { conditional, .. }
        | DeclarativeUiNode::Label { conditional, .. }
        | DeclarativeUiNode::Button { conditional, .. }
        | DeclarativeUiNode::Input { conditional, .. }
        | DeclarativeUiNode::Select { conditional, .. } => {
            conditional_chain_matches(conditional, context, previous_branch_matched)
        }
        DeclarativeUiNode::Template { .. } => {
            *previous_branch_matched = None;
            true
        }
    }
}

pub(crate) fn node_conditional(node: &DeclarativeUiNode) -> Option<&DeclarativeConditional> {
    match node {
        DeclarativeUiNode::Container { conditional, .. }
        | DeclarativeUiNode::Text { conditional, .. }
        | DeclarativeUiNode::Label { conditional, .. }
        | DeclarativeUiNode::Button { conditional, .. }
        | DeclarativeUiNode::Input { conditional, .. }
        | DeclarativeUiNode::Select { conditional, .. } => Some(conditional),
        DeclarativeUiNode::Template { .. } => None,
    }
}

fn outlet_name(node: &DeclarativeUiNode) -> Option<&str> {
    match node {
        DeclarativeUiNode::Container { outlet, .. } => outlet.as_deref(),
        _ => None,
    }
}

fn node_children(node: &DeclarativeUiNode) -> Option<&[DeclarativeUiNode]> {
    match node {
        DeclarativeUiNode::Container { children, .. }
        | DeclarativeUiNode::Label { children, .. } => Some(children),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::super::shell::materialize_declarative_ui_shell_on_entity_in_world;
    use super::*;
    use crate::runtime::state::DeclarativeConditionalSubtree;
    use crate::{UiValue, parse_declarative_ui_asset};

    #[test]
    fn sibling_conditional_chain_materializes_only_first_matching_branch() {
        let asset = parse_declarative_ui_asset(
            r#"
            <template>
              <div>
                <slot name="graphic" v-if="tab === 'graphic'" />
                <slot name="audio" v-else-if="tab === 'audio'" />
                <slot name="general" v-else />
              </div>
            </template>
            "#,
        )
        .expect("asset should parse");
        let mut app = App::new();
        let parent = app.world_mut().spawn_empty().id();

        let collected_slots = materialize_declarative_ui_shell_on_entity_in_world(
            &mut app.world_mut().entity_mut(parent),
            &asset,
            DeclarativeUiBuildContext::default()
                .with_local_state([("tab".to_string(), UiValue::from("graphic"))]),
        );

        assert!(collected_slots.contains_key("graphic"));
        assert!(!collected_slots.contains_key("audio"));
        assert!(!collected_slots.contains_key("general"));
    }

    #[test]
    fn sibling_conditional_chain_uses_else_when_no_prior_branch_matches() {
        let asset = parse_declarative_ui_asset(
            r#"
            <template>
              <div>
                <slot name="graphic" v-if="tab === 'graphic'" />
                <slot name="audio" v-else-if="tab === 'audio'" />
                <slot name="general" v-else />
              </div>
            </template>
            "#,
        )
        .expect("asset should parse");
        let mut app = App::new();
        let parent = app.world_mut().spawn_empty().id();

        let collected_slots = materialize_declarative_ui_shell_on_entity_in_world(
            &mut app.world_mut().entity_mut(parent),
            &asset,
            DeclarativeUiBuildContext::default()
                .with_local_state([("tab".to_string(), UiValue::from("general"))]),
        );

        assert!(!collected_slots.contains_key("graphic"));
        assert!(!collected_slots.contains_key("audio"));
        assert!(collected_slots.contains_key("general"));
    }

    #[test]
    fn root_container_records_conditional_subtree_metadata() {
        let asset = parse_declarative_ui_asset(
            r#"
            <template>
              <div>
                <slot name="graphic" v-if="tab === 'graphic'" />
                <slot name="audio" v-else-if="tab === 'audio'" />
                <slot name="general" v-else />
              </div>
            </template>
            "#,
        )
        .expect("asset should parse");
        let mut app = App::new();
        let parent = app.world_mut().spawn_empty().id();

        materialize_declarative_ui_shell_on_entity_in_world(
            &mut app.world_mut().entity_mut(parent),
            &asset,
            DeclarativeUiBuildContext::default()
                .with_local_state([("tab".to_string(), UiValue::from("audio"))]),
        );

        let subtree = app
            .world()
            .get::<DeclarativeConditionalSubtree>(parent)
            .expect("root container should record conditional subtree metadata");
        assert_eq!(subtree.container_node_id, "0");
        assert_eq!(subtree.chains.len(), 1);
        assert_eq!(subtree.chains[0].start_index, 0);
        assert_eq!(subtree.chains[0].end_index, 2);
        assert_eq!(subtree.chains[0].active_branch_index, Some(1));
    }

    #[test]
    fn template_descendants_do_not_record_conditional_subtree_metadata() {
        let asset = parse_declarative_ui_asset(
            r#"
            <template>
              <div>
                <template v-for="entry in items">
                  <section>
                    <slot name="detail" v-if="entry.visible" />
                    <slot name="fallback" v-else />
                  </section>
                </template>
              </div>
            </template>
            "#,
        )
        .expect("asset should parse");
        let mut app = App::new();
        let parent = app.world_mut().spawn_empty().id();

        materialize_declarative_ui_shell_on_entity_in_world(
            &mut app.world_mut().entity_mut(parent),
            &asset,
            DeclarativeUiBuildContext::default().with_local_state([(
                "items".to_string(),
                UiValue::list([UiValue::object([("visible", UiValue::from(true))])]),
            )]),
        );

        let mut query = app.world_mut().query::<&DeclarativeConditionalSubtree>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }
}
