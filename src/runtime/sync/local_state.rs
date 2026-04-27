use crate::runtime::state::{DeclarativeLocalState, DeclarativeOnClickAssignment};
use crate::value::UiValue;
use beuvy_runtime::button::ButtonClickMessage;
use bevy::prelude::*;

pub(crate) fn apply_declarative_local_state_assignments(
    mut button_clicks: MessageReader<ButtonClickMessage>,
    assignments: Query<&DeclarativeOnClickAssignment>,
    parents: Query<&ChildOf>,
    mut local_states: Query<&mut DeclarativeLocalState>,
) {
    for click in button_clicks.read() {
        let Ok(assignment) = assignments.get(click.entity) else {
            continue;
        };
        let mut current = Some(click.entity);
        while let Some(entity) = current {
            if let Ok(mut local_state) = local_states.get_mut(entity) {
                if !local_state.0.contains_key(&assignment.name) {
                    current = parents.get(entity).ok().map(ChildOf::parent);
                    continue;
                }
                local_state.0.insert(
                    assignment.name.clone(),
                    UiValue::from_literal(&assignment.value),
                );
                break;
            }
            current = parents.get(entity).ok().map(ChildOf::parent);
        }
    }
}
