use super::resolve::resolve_runtime_condition;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefRects, DeclarativeRootComputedLocals,
    DeclarativeRootViewModel, DeclarativeShowExpr, DeclarativeUiRuntimeValues,
};
use bevy::prelude::*;

pub(crate) fn sync_declarative_visibility(
    values: Res<DeclarativeUiRuntimeValues>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    ref_rects: Res<DeclarativeRefRects>,
    mut query: Query<(Entity, &DeclarativeShowExpr, &mut Visibility)>,
) {
    for (entity, binding, mut visibility) in &mut query {
        let shown = resolve_runtime_condition(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        let next = if shown {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
}
