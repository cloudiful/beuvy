use super::resolve::resolve_runtime_path;
use crate::ast::DeclarativeRefSource;
use crate::runtime::refs::resolve_ref;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeRefBinding, DeclarativeRefRects, DeclarativeResolvedRef,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use bevy::prelude::*;

pub(crate) fn materialize_declarative_refs(
    mut commands: Commands,
    query: Query<(Entity, &DeclarativeRefBinding), Added<DeclarativeRefBinding>>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    values: Res<DeclarativeUiRuntimeValues>,
    ref_rects: Res<DeclarativeRefRects>,
) {
    for (entity, ref_binding) in &query {
        let ref_id = match &ref_binding.0 {
            DeclarativeRefSource::Static(ref_id) => Some(ref_id.clone()),
            DeclarativeRefSource::Binding(path) => resolve_runtime_path(
                entity,
                path,
                &parents,
                &local_states,
                &computed,
                &roots,
                &values,
                &ref_rects,
            )
            .and_then(|value| value.text().map(str::to_string)),
        };
        let Some(ref_id) = ref_id else {
            continue;
        };
        commands
            .entity(entity)
            .insert(DeclarativeResolvedRef(ref_id.clone()));
        resolve_ref(&mut commands, entity, &ref_id);
    }
}
