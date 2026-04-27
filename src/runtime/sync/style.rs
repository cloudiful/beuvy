use super::evaluate_runtime_expr;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeNodeStyleBindingComponent, DeclarativeRefRects,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use bevy::prelude::*;
use bevy::ui::Val;

pub(crate) fn sync_declarative_node_style_bindings(
    mut nodes: Query<(Entity, &DeclarativeNodeStyleBindingComponent, &mut Node)>,
    parents: Query<&ChildOf>,
    states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    values: Res<DeclarativeUiRuntimeValues>,
    ref_rects: Res<DeclarativeRefRects>,
) {
    for (entity, binding, mut node) in &mut nodes {
        let left_value = binding.0.left.as_ref().and_then(|expr| {
            evaluate_runtime_expr(
                entity,
                expr,
                &parents,
                &states,
                &computed,
                &roots,
                &values,
                &ref_rects,
                &mut Vec::new(),
            )
        });
        let top_value = binding.0.top.as_ref().and_then(|expr| {
            evaluate_runtime_expr(
                entity,
                expr,
                &parents,
                &states,
                &computed,
                &roots,
                &values,
                &ref_rects,
                &mut Vec::new(),
            )
        });

        if let Some(left) = left_value.as_ref().and_then(|value| value.number()) {
            node.left = Val::Px(left as f32);
        }
        if let Some(top) = top_value.as_ref().and_then(|value| value.number()) {
            node.top = Val::Px(top as f32);
        }
    }
}
