use super::expression::evaluate_runtime_expr;
use crate::ast::DeclarativeConditionExpr;
use crate::runtime::context::resolve_path;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use crate::value::UiValue;
use bevy::prelude::*;

pub(super) fn resolve_runtime_condition(
    entity: Entity,
    expr: &DeclarativeConditionExpr,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> bool {
    match expr {
        DeclarativeConditionExpr::Binding(path) => resolve_runtime_path(
            entity, path, parents, states, computed, roots, values, ref_rects,
        )
        .and_then(|value| value.bool())
        .unwrap_or(false),
        DeclarativeConditionExpr::Equals { name, value } => resolve_runtime_path(
            entity, name, parents, states, computed, roots, values, ref_rects,
        )
        .is_some_and(|candidate| candidate == UiValue::from_literal(value)),
    }
}

pub fn resolve_runtime_path(
    entity: Entity,
    path: &str,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
) -> Option<UiValue> {
    let mut stack = Vec::new();
    resolve_runtime_path_with_extra_locals(
        entity, path, None, parents, states, computed, roots, values, ref_rects, &mut stack,
    )
}

pub(crate) fn resolve_runtime_path_with_extra_locals(
    entity: Entity,
    path: &str,
    extra_locals: Option<&std::collections::HashMap<String, UiValue>>,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
) -> Option<UiValue> {
    if path == "props" {
        let mut current = Some(entity);
        while let Some(candidate) = current {
            if let Ok(root) = roots.get(candidate) {
                return Some(root.0.clone());
            }
            current = parents.get(candidate).ok().map(ChildOf::parent);
        }
        return None;
    }
    let path = path.strip_prefix("props.").unwrap_or(path);
    if let Some(extra_locals) = extra_locals {
        if let Some(value) = extra_locals.get(path) {
            return Some(value.clone());
        }
        if let Some((head, tail)) = path.split_once('.')
            && let Some(value) = extra_locals.get(head)
            && let Some(value) = resolve_path(value, tail)
        {
            return Some(value.clone());
        }
    }
    let mut current = Some(entity);
    while let Some(candidate) = current {
        if let Ok(local_state) = states.get(candidate)
            && let Some(value) = local_state.0.get(path)
        {
            return Some(value.clone());
        }
        if let Some((head, tail)) = path.split_once('.')
            && let Ok(local_state) = states.get(candidate)
            && let Some(value) = local_state.0.get(head)
            && let Some(value) = resolve_path(value, tail)
        {
            return Some(value.clone());
        }
        if let Ok(root_computed) = computed.get(candidate)
            && let Some(value) = resolve_computed_path(
                candidate,
                path,
                parents,
                states,
                computed,
                roots,
                values,
                ref_rects,
                root_computed,
                stack,
            )
        {
            return Some(value);
        }
        if let Ok(root) = roots.get(candidate)
            && let Some(value) = resolve_path(&root.0, path)
        {
            return Some(value.clone());
        }
        current = parents.get(candidate).ok().map(ChildOf::parent);
    }
    values.get(path).cloned()
}

fn resolve_computed_path(
    entity: Entity,
    path: &str,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    root_computed: &DeclarativeRootComputedLocals,
    stack: &mut Vec<String>,
) -> Option<UiValue> {
    let (head, tail) = path
        .split_once('.')
        .map_or((path, None), |(head, tail)| (head, Some(tail)));
    let expr = root_computed.0.get(head)?;
    if stack.iter().any(|name| name == head) {
        return None;
    }
    stack.push(head.to_string());
    let value = evaluate_runtime_expr(
        entity, expr, parents, states, computed, roots, values, ref_rects, stack,
    );
    stack.pop();
    let value = value?;
    match tail {
        Some(tail) => resolve_path(&value, tail).cloned(),
        None => Some(value),
    }
}
