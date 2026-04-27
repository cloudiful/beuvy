use super::resolve::resolve_runtime_condition;
use crate::runtime::state::{
    DeclarativeDisabledExpr, DeclarativeLocalState, DeclarativeRefRects,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
};
use beuvy_runtime::button::{Button, ButtonLabel, DisabledButton};
use beuvy_runtime::input::{DisabledInput, InputField, set_input_disabled};
use beuvy_runtime::text::FontResource;
use beuvy_runtime::{
    Select, SelectPanel, selected_option, sync_select_label, trigger_label_entity,
};
use bevy::prelude::*;

pub(crate) fn sync_declarative_disabled(
    mut commands: Commands,
    values: Res<DeclarativeUiRuntimeValues>,
    font_resource: Res<FontResource>,
    localization: Option<Res<bevy_localization::Localization>>,
    parents: Query<&ChildOf>,
    local_states: Query<&DeclarativeLocalState>,
    computed: Query<&DeclarativeRootComputedLocals>,
    roots: Query<&DeclarativeRootViewModel>,
    ref_rects: Res<DeclarativeRefRects>,
    button_labels: Query<&ButtonLabel>,
    mut inputs: Query<
        (
            Entity,
            &DeclarativeDisabledExpr,
            &InputField,
            Has<DisabledInput>,
        ),
        With<InputField>,
    >,
    buttons: Query<
        (Entity, &DeclarativeDisabledExpr, Has<DisabledButton>),
        (With<Button>, Without<InputField>),
    >,
    mut selects: Query<(Entity, &DeclarativeDisabledExpr, &mut Select)>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
) {
    for (entity, binding, field, disabled) in &mut inputs {
        let next = resolve_runtime_condition(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        if disabled != next {
            set_input_disabled(&mut commands, &font_resource, entity, field, next);
        }
    }

    for (entity, binding, disabled) in &buttons {
        let next = resolve_runtime_condition(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        if disabled == next {
            continue;
        }
        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };
        if next {
            entity_commands
                .try_insert((DisabledButton, beuvy_runtime::interaction_style::UiDisabled));
        } else {
            entity_commands
                .try_remove::<DisabledButton>()
                .try_remove::<beuvy_runtime::interaction_style::UiDisabled>();
        }
    }

    for (entity, binding, mut select) in &mut selects {
        let next = resolve_runtime_condition(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        );
        if select.disabled == next {
            continue;
        }
        select.disabled = next;
        if next {
            select.open = false;
            if let Ok(mut panel) = panel_nodes.get_mut(select.panel) {
                panel.display = Display::None;
            }
        }
        if let (Some(localization), Some(label_entity)) = (
            localization.as_deref(),
            trigger_label_entity(&button_labels, &select),
        ) {
            sync_select_label(
                &mut commands,
                Some(localization),
                label_entity,
                selected_option(&select),
                &select.value,
            );
        }
    }
}
