use super::evaluate_runtime_expr_with_locals;
use crate::ast::{DeclarativeBinaryOp, DeclarativeRuntimeExpr};
use crate::runtime::context::resolve_path;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use crate::runtime::sync::resolve::resolve_runtime_path_with_extra_locals;
use crate::value::UiValue;
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) fn resolve_runtime_path_in_scope(
    entity: Entity,
    path: &str,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
    locals: &HashMap<String, UiValue>,
) -> Option<UiValue> {
    if let Some(value) = locals.get(path) {
        return Some(value.clone());
    }
    if let Some((head, tail)) = path.split_once('.')
        && let Some(value) = locals.get(head)
    {
        return resolve_path(value, tail).cloned();
    }
    resolve_runtime_path_with_extra_locals(
        entity, path, None, parents, states, computed, roots, values, ref_rects, stack,
    )
}

pub(super) fn evaluate_binary_expr(
    left: UiValue,
    op: DeclarativeBinaryOp,
    right: UiValue,
) -> Option<UiValue> {
    match op {
        DeclarativeBinaryOp::Add => Some(UiValue::from(left.number()? + right.number()?)),
        DeclarativeBinaryOp::Subtract => Some(UiValue::from(left.number()? - right.number()?)),
        DeclarativeBinaryOp::Multiply => Some(UiValue::from(left.number()? * right.number()?)),
        DeclarativeBinaryOp::Divide => Some(UiValue::from(left.number()? / right.number()?)),
        DeclarativeBinaryOp::LessThan => Some(UiValue::from(left.number()? < right.number()?)),
        DeclarativeBinaryOp::LessThanOrEqual => {
            Some(UiValue::from(left.number()? <= right.number()?))
        }
        DeclarativeBinaryOp::GreaterThan => Some(UiValue::from(left.number()? > right.number()?)),
        DeclarativeBinaryOp::GreaterThanOrEqual => {
            Some(UiValue::from(left.number()? >= right.number()?))
        }
        DeclarativeBinaryOp::Equal => Some(UiValue::from(left == right)),
        DeclarativeBinaryOp::NotEqual => Some(UiValue::from(left != right)),
    }
}

pub(super) fn evaluate_math_args(
    entity: Entity,
    args: &[DeclarativeRuntimeExpr],
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
    locals: &mut HashMap<String, UiValue>,
) -> Option<Vec<f64>> {
    let mut numbers = Vec::with_capacity(args.len());
    for arg in args {
        numbers.push(
            evaluate_runtime_expr_with_locals(
                entity, arg, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?
            .number()?,
        );
    }
    if numbers.is_empty() {
        return None;
    }
    Some(numbers)
}

pub(crate) fn truthy(value: &UiValue) -> bool {
    match value {
        UiValue::Null => false,
        UiValue::Bool(value) => *value,
        UiValue::Number(number) => match number {
            crate::DeclarativeNumber::I32(value) => *value != 0,
            crate::DeclarativeNumber::I64(value) => *value != 0,
            crate::DeclarativeNumber::F32(value) => *value != 0.0,
            crate::DeclarativeNumber::F64(value) => *value != 0.0,
        },
        UiValue::Text(value) => !value.is_empty(),
        UiValue::Object(fields) => !fields.is_empty(),
        UiValue::List(items) => !items.is_empty(),
    }
}

pub(super) fn evaluate_anchor_popup(
    anchor_rect: &UiValue,
    shell_rect: &UiValue,
    popup_width: f32,
    popup_min_height: f32,
    gap: f32,
    margin: f32,
) -> Option<UiValue> {
    let anchor_left = field_number(anchor_rect, "left")? as f32;
    let anchor_top = field_number(anchor_rect, "top")? as f32;
    let anchor_width = field_number(anchor_rect, "width")? as f32;
    let shell_left = field_number(shell_rect, "left")? as f32;
    let shell_top = field_number(shell_rect, "top")? as f32;
    let shell_width = field_number(shell_rect, "width")? as f32;
    let shell_height = field_number(shell_rect, "height")? as f32;

    let anchor_left_local = anchor_left - shell_left;
    let anchor_top_local = anchor_top - shell_top;
    let preferred_left = anchor_left_local + anchor_width + gap;
    let left = if preferred_left + popup_width + margin <= shell_width {
        preferred_left
    } else {
        (anchor_left_local - popup_width - gap).max(margin)
    };
    let top = anchor_top_local.clamp(
        margin,
        (shell_height - popup_min_height - margin).max(margin),
    );

    Some(UiValue::object([
        ("left", UiValue::from(left)),
        ("top", UiValue::from(top)),
    ]))
}

fn field_number(value: &UiValue, name: &str) -> Option<f64> {
    value.field(name)?.number()
}
