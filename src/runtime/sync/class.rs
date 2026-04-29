use super::resolve::resolve_runtime_condition;
use super::{evaluate_runtime_expr, truthy};
use crate::runtime::state::{
    DeclarativeClassBindings, DeclarativeLocalState, DeclarativeRefRects,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use crate::{DeclarativeClassBinding, value::UiValue};
use beuvy_runtime::button::ButtonLabel;
use beuvy_runtime::link::LinkLabel;
use beuvy_runtime::interaction_style::{
    pointer_cancel, pointer_drag_end, pointer_hover_out, pointer_hover_over, pointer_press,
    pointer_release,
};
use beuvy_runtime::style::{
    apply_utility_patch, resolve_class_patch_or_empty, root_visual_styles_from_patch,
    text_visual_styles_from_patch,
};
use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub(crate) struct DeclarativeClassBaseline {
    node: Option<Node>,
    background: Option<BackgroundColor>,
    border: Option<BorderColor>,
    text: Option<TextColor>,
    outline: Option<Outline>,
}

#[allow(clippy::type_complexity)]
pub(crate) fn sync_declarative_class_bindings(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    values: Res<DeclarativeUiRuntimeValues>,
    ref_rects: Res<DeclarativeRefRects>,
    mut query: Query<(
        Entity,
        &mut DeclarativeClassBindings,
        Option<&DeclarativeClassBaseline>,
        Option<&mut Node>,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut TextColor>,
        Option<&mut Outline>,
        Option<&ButtonLabel>,
        Option<&LinkLabel>,
    )>,
) {
    for (
        entity,
        mut binding,
        baseline,
        mut node,
        mut background,
        mut border,
        mut text,
        mut outline,
        label,
        link_label,
    ) in &mut query
    {
        if baseline.is_none()
            && node.is_none()
            && background.is_none()
            && border.is_none()
            && text.is_none()
            && outline.is_none()
            && label.is_none()
            && link_label.is_none()
        {
            continue;
        }

        let captured = DeclarativeClassBaseline {
            node: node.as_deref().cloned(),
            background: background.as_deref().copied(),
            border: border.as_deref().cloned(),
            text: text.as_deref().copied(),
            outline: outline.as_deref().cloned(),
        };
        let baseline = baseline.cloned().unwrap_or_else(|| {
            queue_entity_silenced(&mut commands, entity, {
                let captured = captured.clone();
                move |entity| {
                    entity.insert(captured);
                }
            });
            captured
        });

        let resolved = resolve_class_binding_string(
            entity, &binding, &parents, &states, &computed, &roots, &values, &ref_rects,
        );
        if binding.resolved_class == resolved {
            apply_label_dynamic_class(label.map(|v| v.entity), entity, &mut commands, &resolved);
            apply_label_dynamic_class(
                link_label.map(|v| v.entity),
                entity,
                &mut commands,
                &resolved,
            );
            continue;
        }
        binding.resolved_class = resolved.clone();

        if let Some(node) = node.as_deref_mut()
            && let Some(base_node) = &baseline.node
        {
            *node = base_node.clone();
            let patch = resolve_class_patch_or_empty(&resolved, "declarative dynamic class");
            apply_utility_patch(node, &patch);
        }

        reset_visual_baseline(
            entity,
            &mut commands,
            &baseline,
            background.as_deref_mut(),
            border.as_deref_mut(),
            text.as_deref_mut(),
            outline.as_deref_mut(),
        );

        let patch = resolve_class_patch_or_empty(&resolved, "declarative dynamic class");
        let root_styles = if text.is_some() && background.is_none() && border.is_none() {
            text_visual_styles_from_patch(&patch).unwrap_or_default()
        } else {
            root_visual_styles_from_patch(&patch).unwrap_or_default()
        };
        queue_entity_silenced(&mut commands, entity, move |entity| {
            entity
                .insert(root_styles)
                .observe(pointer_hover_over)
                .observe(pointer_hover_out)
                .observe(pointer_press)
                .observe(pointer_release)
                .observe(pointer_cancel)
                .observe(pointer_drag_end);
        });

        apply_label_dynamic_class(label.map(|v| v.entity), entity, &mut commands, &resolved);
        apply_label_dynamic_class(
            link_label.map(|v| v.entity),
            entity,
            &mut commands,
            &resolved,
        );
    }
}

fn apply_label_dynamic_class(
    label: Option<Entity>,
    source: Entity,
    commands: &mut Commands,
    resolved: &str,
) {
    let Some(label) = label else {
        return;
    };
    let patch = resolve_class_patch_or_empty(resolved, "declarative dynamic button label class");
    let label_styles = text_visual_styles_from_patch(&patch).unwrap_or_default();
    queue_entity_silenced(commands, label, move |entity| {
        entity.insert((
            label_styles,
            beuvy_runtime::interaction_style::UiStateStyleSource(source),
        ));
    });
}

fn resolve_class_binding_string(
    entity: Entity,
    binding: &DeclarativeClassBindings,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> String {
    let mut classes = binding.base_class.trim().to_string();
    for class_binding in &binding.bindings {
        match class_binding {
            DeclarativeClassBinding::Conditional {
                class_name,
                condition,
            } => {
                if !resolve_runtime_condition(
                    entity, condition, parents, states, computed, roots, values, ref_rects,
                ) {
                    continue;
                }
                push_classes(&mut classes, class_name);
            }
            DeclarativeClassBinding::RuntimeExpr { expr } => {
                let Some(value) = evaluate_runtime_expr(
                    entity,
                    expr,
                    parents,
                    states,
                    computed,
                    roots,
                    values,
                    ref_rects,
                    &mut Vec::new(),
                ) else {
                    continue;
                };
                push_classes_from_value(&mut classes, &value);
            }
        }
    }
    classes
}

fn push_classes_from_value(classes: &mut String, value: &UiValue) {
    match value {
        UiValue::Null | UiValue::Bool(false) => {}
        UiValue::Bool(true) => {}
        UiValue::Number(_) => {}
        UiValue::Text(value) => push_classes(classes, value),
        UiValue::List(items) => {
            for item in items.iter() {
                push_classes_from_value(classes, item);
            }
        }
        UiValue::Object(fields) => {
            for (class_name, enabled) in fields.iter() {
                if truthy(enabled) {
                    push_classes(classes, class_name);
                }
            }
        }
    }
}

fn push_classes(classes: &mut String, raw: &str) {
    for class_name in raw.split_whitespace() {
        if class_name.is_empty() {
            continue;
        }
        if !classes.is_empty() {
            classes.push(' ');
        }
        classes.push_str(class_name);
    }
}

fn reset_visual_baseline(
    entity: Entity,
    commands: &mut Commands,
    baseline: &DeclarativeClassBaseline,
    background: Option<&mut BackgroundColor>,
    border: Option<&mut BorderColor>,
    text: Option<&mut TextColor>,
    outline: Option<&mut Outline>,
) {
    match (background, baseline.background) {
        (Some(current), Some(base)) => *current = base,
        (Some(_), None) => {
            queue_entity_silenced(commands, entity, |entity| {
                entity.remove::<BackgroundColor>();
            });
        }
        (None, Some(base)) => {
            queue_entity_silenced(commands, entity, move |entity| {
                entity.insert(base);
            });
        }
        (None, None) => {}
    }
    match (border, baseline.border.clone()) {
        (Some(current), Some(base)) => *current = base,
        (Some(_), None) => {
            queue_entity_silenced(commands, entity, |entity| {
                entity.remove::<BorderColor>();
            });
        }
        (None, Some(base)) => {
            queue_entity_silenced(commands, entity, move |entity| {
                entity.insert(base);
            });
        }
        (None, None) => {}
    }
    match (text, baseline.text) {
        (Some(current), Some(base)) => *current = base,
        (Some(_), None) => {
            queue_entity_silenced(commands, entity, |entity| {
                entity.remove::<TextColor>();
            });
        }
        (None, Some(base)) => {
            queue_entity_silenced(commands, entity, move |entity| {
                entity.insert(base);
            });
        }
        (None, None) => {}
    }
    match (outline, baseline.outline.clone()) {
        (Some(current), Some(base)) => *current = base,
        (Some(_), None) => {
            queue_entity_silenced(commands, entity, |entity| {
                entity.remove::<Outline>();
            });
        }
        (None, Some(base)) => {
            queue_entity_silenced(commands, entity, move |entity| {
                entity.insert(base);
            });
        }
        (None, None) => {}
    }
}

fn queue_entity_silenced(
    commands: &mut Commands,
    entity: Entity,
    f: impl FnOnce(&mut EntityWorldMut) + Send + Sync + 'static,
) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };
    entity_commands.queue_silenced(move |mut entity: EntityWorldMut| {
        f(&mut entity);
    });
}
