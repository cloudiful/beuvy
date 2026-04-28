use crate::runtime::bindings::condition_expr_matches;
use crate::runtime::context::DeclarativeUiBuildContext;
use crate::runtime::state::{
    DeclarativeFieldsetState, DeclarativeLocalState, DeclarativeRootViewModel,
};
use crate::value::UiValue;
use bevy::prelude::*;

pub(crate) fn sync_fieldset_disabled_states(
    roots: Query<&DeclarativeRootViewModel>,
    locals: Query<&DeclarativeLocalState>,
    mut fieldsets: Query<(Entity, &mut DeclarativeFieldsetState)>,
) {
    for (entity, mut fieldset) in &mut fieldsets {
        fieldset.disabled = if let Some(expr) = &fieldset.disabled_expr {
            let root = roots.get(entity).map(|value| value.0.clone()).unwrap_or_default();
            let local_state = locals
                .get(entity)
                .map(|state| state.0.clone())
                .unwrap_or_default();
            let context = DeclarativeUiBuildContext::default()
                .with_root(root)
                .with_local_state(local_state.into_iter().collect::<Vec<(String, UiValue)>>());
            condition_expr_matches(expr, &context) || fieldset.disabled
        } else {
            fieldset.disabled
        };
    }
}
