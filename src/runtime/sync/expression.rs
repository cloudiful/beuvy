#[path = "expression/helpers.rs"]
mod helpers;

use self::helpers::{
    evaluate_anchor_popup, evaluate_binary_expr, evaluate_math_args, resolve_runtime_path_in_scope,
};
use crate::ast::{DeclarativeRuntimeExpr, DeclarativeRuntimeStmt};
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use crate::value::UiValue;
use bevy::prelude::*;
use std::collections::HashMap;

pub(crate) use self::helpers::truthy;

pub(crate) fn evaluate_runtime_expr(
    entity: Entity,
    expr: &DeclarativeRuntimeExpr,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
) -> Option<UiValue> {
    let mut locals = HashMap::new();
    evaluate_runtime_expr_with_locals(
        entity,
        expr,
        parents,
        states,
        computed,
        roots,
        values,
        ref_rects,
        stack,
        &mut locals,
    )
}

pub(super) fn evaluate_runtime_expr_with_locals(
    entity: Entity,
    expr: &DeclarativeRuntimeExpr,
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
    locals: &mut HashMap<String, UiValue>,
) -> Option<UiValue> {
    match expr {
        DeclarativeRuntimeExpr::BindingPath(path) => resolve_runtime_path_in_scope(
            entity, path, parents, states, computed, roots, values, ref_rects, stack, locals,
        ),
        DeclarativeRuntimeExpr::Literal(value) => Some(UiValue::from_literal(value)),
        DeclarativeRuntimeExpr::NumberLiteral(number) => Some(UiValue::Number(*number)),
        DeclarativeRuntimeExpr::ArrayLiteral(items) => {
            Some(UiValue::list(items.iter().filter_map(|item| {
                evaluate_runtime_expr_with_locals(
                    entity, item, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )
            })))
        }
        DeclarativeRuntimeExpr::GetBoundingClientRect { target_path } => {
            let ref_id = resolve_runtime_path_in_scope(
                entity,
                target_path,
                parents,
                states,
                computed,
                roots,
                values,
                ref_rects,
                stack,
                locals,
            )?
            .text()?
            .to_string();
            ref_rects.get_rect(&ref_id).cloned()
        }
        DeclarativeRuntimeExpr::FieldAccess { base, field } => evaluate_runtime_expr_with_locals(
            entity, base, parents, states, computed, roots, values, ref_rects, stack, locals,
        )?
        .field(field)
        .cloned(),
        DeclarativeRuntimeExpr::UnaryNot { expr } => {
            Some(UiValue::from(!truthy(&evaluate_runtime_expr_with_locals(
                entity, expr, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?)))
        }
        DeclarativeRuntimeExpr::Binary { left, op, right } => {
            let left = evaluate_runtime_expr_with_locals(
                entity, left, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?;
            let right = evaluate_runtime_expr_with_locals(
                entity, right, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?;
            evaluate_binary_expr(left, *op, right)
        }
        DeclarativeRuntimeExpr::ObjectLiteral(fields) => {
            Some(UiValue::object(fields.iter().map(|(name, expr)| {
                let value = evaluate_runtime_expr_with_locals(
                    entity, expr, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )
                .unwrap_or(UiValue::Null);
                (name.clone(), value)
            })))
        }
        DeclarativeRuntimeExpr::MathMin { args } => evaluate_math_args(
            entity, args, parents, states, computed, roots, values, ref_rects, stack, locals,
        )
        .map(|value: Vec<f64>| UiValue::from(value.into_iter().fold(f64::INFINITY, f64::min))),
        DeclarativeRuntimeExpr::MathMax { args } => evaluate_math_args(
            entity, args, parents, states, computed, roots, values, ref_rects, stack, locals,
        )
        .map(|value: Vec<f64>| UiValue::from(value.into_iter().fold(f64::NEG_INFINITY, f64::max))),
        DeclarativeRuntimeExpr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            if truthy(&evaluate_runtime_expr_with_locals(
                entity, condition, parents, states, computed, roots, values, ref_rects, stack,
                locals,
            )?) {
                evaluate_runtime_expr_with_locals(
                    entity, then_expr, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )
            } else {
                evaluate_runtime_expr_with_locals(
                    entity, else_expr, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )
            }
        }
        DeclarativeRuntimeExpr::Block(statements) => evaluate_runtime_block(
            entity, statements, parents, states, computed, roots, values, ref_rects, stack, locals,
        ),
        DeclarativeRuntimeExpr::AnchorPopup {
            anchor_rect,
            shell_rect,
            popup_width,
            popup_min_height,
            gap,
            margin,
        } => {
            let anchor_rect = evaluate_runtime_expr_with_locals(
                entity,
                anchor_rect,
                parents,
                states,
                computed,
                roots,
                values,
                ref_rects,
                stack,
                locals,
            )?;
            let shell_rect = evaluate_runtime_expr_with_locals(
                entity, shell_rect, parents, states, computed, roots, values, ref_rects, stack,
                locals,
            )?;
            let popup_width = evaluate_runtime_expr_with_locals(
                entity,
                popup_width,
                parents,
                states,
                computed,
                roots,
                values,
                ref_rects,
                stack,
                locals,
            )?
            .number()? as f32;
            let popup_min_height = evaluate_runtime_expr_with_locals(
                entity,
                popup_min_height,
                parents,
                states,
                computed,
                roots,
                values,
                ref_rects,
                stack,
                locals,
            )?
            .number()? as f32;
            let gap = evaluate_runtime_expr_with_locals(
                entity, gap, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?
            .number()? as f32;
            let margin = evaluate_runtime_expr_with_locals(
                entity, margin, parents, states, computed, roots, values, ref_rects, stack, locals,
            )?
            .number()? as f32;
            evaluate_anchor_popup(
                &anchor_rect,
                &shell_rect,
                popup_width,
                popup_min_height,
                gap,
                margin,
            )
        }
    }
}

fn evaluate_runtime_block(
    entity: Entity,
    statements: &[DeclarativeRuntimeStmt],
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
    locals: &mut HashMap<String, UiValue>,
) -> Option<UiValue> {
    let mut block_locals = locals.clone();
    evaluate_runtime_statements(
        entity,
        statements,
        parents,
        states,
        computed,
        roots,
        values,
        ref_rects,
        stack,
        &mut block_locals,
    )
}

fn evaluate_runtime_statements(
    entity: Entity,
    statements: &[DeclarativeRuntimeStmt],
    parents: &Query<&ChildOf>,
    states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    stack: &mut Vec<String>,
    locals: &mut HashMap<String, UiValue>,
) -> Option<UiValue> {
    for statement in statements {
        match statement {
            DeclarativeRuntimeStmt::Const { name, expr } => {
                let value = evaluate_runtime_expr_with_locals(
                    entity, expr, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )?;
                locals.insert(name.clone(), value);
            }
            DeclarativeRuntimeStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let branch = if truthy(&evaluate_runtime_expr_with_locals(
                    entity, condition, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                )?) {
                    then_branch
                } else {
                    else_branch
                };
                if let Some(value) = evaluate_runtime_statements(
                    entity, branch, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                ) {
                    return Some(value);
                }
            }
            DeclarativeRuntimeStmt::Return(expr) => {
                return evaluate_runtime_expr_with_locals(
                    entity, expr, parents, states, computed, roots, values, ref_rects, stack,
                    locals,
                );
            }
        }
    }
    None
}
