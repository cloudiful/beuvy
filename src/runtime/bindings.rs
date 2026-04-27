use super::context::DeclarativeUiBuildContext;
use super::state::{
    DeclarativeDisabledExpr, DeclarativeEventBindings, DeclarativeModelBinding,
    DeclarativeNodeStyleBindingComponent, DeclarativeRefBinding, DeclarativeShowExpr,
    DeclarativeValueBinding, ResolvedDeclarativeEventBinding,
};
use super::style::DeclarativeEntityInsert;
use crate::ast::*;
use crate::value::UiValue;
use bevy::prelude::*;
use std::collections::BTreeMap;

pub(crate) fn resolve_declarative_button_event_bindings(
    node: &DeclarativeUiNode,
    context: &DeclarativeUiBuildContext,
) -> Option<Vec<ResolvedDeclarativeEventBinding>> {
    let DeclarativeUiNode::Button { event_bindings, .. } = node else {
        unreachable!();
    };

    event_bindings
        .iter()
        .map(|binding| resolve_event_binding(binding, context))
        .collect()
}

pub(crate) fn resolve_event_binding(
    binding: &DeclarativeEventBinding,
    context: &DeclarativeUiBuildContext,
) -> Option<ResolvedDeclarativeEventBinding> {
    let mut params = BTreeMap::new();
    for (name, value) in &binding.params {
        let value = match value {
            DeclarativeValueExpr::Literal(value) => literal_value_string(value),
            DeclarativeValueExpr::Binding(path) => context.string(path)?,
        };
        params.insert(name.clone(), value);
    }
    Some(ResolvedDeclarativeEventBinding {
        kind: binding.kind,
        action_id: binding.action_id.clone(),
        params,
    })
}

fn literal_value_string(value: &DeclarativeLiteral) -> String {
    match value {
        DeclarativeLiteral::String(value) => value.clone(),
        DeclarativeLiteral::Bool(value) => value.to_string(),
        DeclarativeLiteral::Number(value) => match value {
            DeclarativeNumber::I32(value) => value.to_string(),
            DeclarativeNumber::I64(value) => value.to_string(),
            DeclarativeNumber::F32(value) => value.to_string(),
            DeclarativeNumber::F64(value) => value.to_string(),
        },
    }
}

pub(crate) fn conditional_matches(
    conditional: &DeclarativeConditional,
    context: &DeclarativeUiBuildContext,
) -> bool {
    match conditional {
        DeclarativeConditional::Always | DeclarativeConditional::Else => true,
        DeclarativeConditional::If(expr) | DeclarativeConditional::ElseIf(expr) => {
            condition_expr_matches(expr, context)
        }
    }
}

pub(crate) fn conditional_chain_matches(
    conditional: &DeclarativeConditional,
    context: &DeclarativeUiBuildContext,
    previous_branch_matched: &mut Option<bool>,
) -> bool {
    match conditional {
        DeclarativeConditional::Always => {
            *previous_branch_matched = None;
            true
        }
        DeclarativeConditional::If(expr) => {
            let matched = condition_expr_matches(expr, context);
            *previous_branch_matched = Some(matched);
            matched
        }
        DeclarativeConditional::ElseIf(expr) => {
            let previous_matched = previous_branch_matched.unwrap_or(false);
            let matched = !previous_matched && condition_expr_matches(expr, context);
            *previous_branch_matched = Some(previous_matched || matched);
            matched
        }
        DeclarativeConditional::Else => {
            let previous_matched = previous_branch_matched.unwrap_or(false);
            let matched = !previous_matched;
            *previous_branch_matched = Some(true);
            matched
        }
    }
}

pub(crate) fn condition_expr_matches(
    expr: &DeclarativeConditionExpr,
    context: &DeclarativeUiBuildContext,
) -> bool {
    match expr {
        DeclarativeConditionExpr::Binding(path) => context.bool(path).unwrap_or(false),
        DeclarativeConditionExpr::Equals { name, value } => context
            .resolve(name)
            .is_some_and(|candidate| candidate == &UiValue::from_literal(value)),
    }
}

pub(crate) fn visibility_for_show_binding(
    show_expr: Option<&DeclarativeConditionExpr>,
    context: &DeclarativeUiBuildContext,
) -> Visibility {
    if show_expr
        .map(|expr| condition_expr_matches(expr, context))
        .unwrap_or(true)
    {
        Visibility::Visible
    } else {
        Visibility::Hidden
    }
}

pub(crate) fn apply_common_bindings_to_entity(
    entity: &mut impl DeclarativeEntityInsert,
    show_expr: Option<&DeclarativeConditionExpr>,
    disabled_expr: Option<&DeclarativeConditionExpr>,
    value_binding: Option<&str>,
    model_binding: Option<&str>,
    ref_binding: Option<&DeclarativeRefSource>,
    style_binding: Option<&DeclarativeNodeStyleBinding>,
    event_bindings: &[DeclarativeEventBinding],
    context: &DeclarativeUiBuildContext,
) {
    if let Some(expr) = show_expr {
        entity.insert_component(DeclarativeShowExpr(expr.clone()));
        entity.insert_component(visibility_for_show_binding(show_expr, context));
    }
    if let Some(expr) = disabled_expr {
        entity.insert_component(DeclarativeDisabledExpr(expr.clone()));
    }
    if let Some(path) = model_binding.or(value_binding) {
        entity.insert_component(DeclarativeValueBinding(path.to_string()));
    }
    if model_binding.is_some() {
        entity.insert_component(DeclarativeModelBinding);
    }
    if let Some(ref_binding) = ref_binding {
        entity.insert_component(DeclarativeRefBinding(ref_binding.clone()));
    }
    if let Some(binding) = style_binding {
        entity.insert_component(DeclarativeNodeStyleBindingComponent(binding.clone()));
    }
    if !event_bindings.is_empty() {
        let resolved = event_bindings
            .iter()
            .filter_map(|binding| resolve_event_binding(binding, context))
            .collect::<Vec<_>>();
        if !resolved.is_empty() {
            entity.insert_component(DeclarativeEventBindings(resolved));
        }
    }
}
