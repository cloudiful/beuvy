use super::resolve::resolve_runtime_condition;
use crate::ast::DeclarativeContainerKind;
use crate::runtime::state::{
    DeclarativeContainerSemantic, DeclarativeDisabledExpr, DeclarativeExplicitDisabled,
    DeclarativeFieldsetState, DeclarativeLocalState, DeclarativeRefRects,
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
    semantics: Query<&DeclarativeContainerSemantic>,
    fieldsets: Query<&DeclarativeFieldsetState>,
    mut inputs: Query<
        (
            Entity,
            &DeclarativeExplicitDisabled,
            Option<&DeclarativeDisabledExpr>,
            &InputField,
            Has<DisabledInput>,
        ),
        With<InputField>,
    >,
    buttons: Query<
        (
            Entity,
            &DeclarativeExplicitDisabled,
            Option<&DeclarativeDisabledExpr>,
            Has<DisabledButton>,
        ),
        (With<Button>, Without<InputField>),
    >,
    mut selects: Query<(
        Entity,
        &DeclarativeExplicitDisabled,
        Option<&DeclarativeDisabledExpr>,
        &mut Select,
    )>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
) {
    for (entity, explicit_disabled, binding, field, disabled) in &mut inputs {
        let next = effective_disabled(
            entity,
            explicit_disabled.0,
            binding.map(|value| &value.0),
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
            &semantics,
            &fieldsets,
        );
        if disabled != next {
            set_input_disabled(&mut commands, &font_resource, entity, field, next);
        }
    }

    for (entity, explicit_disabled, binding, disabled) in &buttons {
        let next = effective_disabled(
            entity,
            explicit_disabled.0,
            binding.map(|value| &value.0),
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
            &semantics,
            &fieldsets,
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

    for (entity, explicit_disabled, binding, mut select) in &mut selects {
        let next = effective_disabled(
            entity,
            explicit_disabled.0,
            binding.map(|value| &value.0),
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
            &semantics,
            &fieldsets,
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

fn effective_disabled(
    entity: Entity,
    explicit_disabled: bool,
    disabled_expr: Option<&crate::DeclarativeConditionExpr>,
    parents: &Query<&ChildOf>,
    local_states: &Query<&DeclarativeLocalState>,
    computed: &Query<&DeclarativeRootComputedLocals>,
    roots: &Query<&DeclarativeRootViewModel>,
    values: &DeclarativeUiRuntimeValues,
    ref_rects: &DeclarativeRefRects,
    semantics: &Query<&DeclarativeContainerSemantic>,
    fieldsets: &Query<&DeclarativeFieldsetState>,
) -> bool {
    explicit_disabled
        || disabled_expr.is_some_and(|expr| {
            resolve_runtime_condition(
                entity,
                expr,
                parents,
                local_states,
                computed,
                roots,
                values,
                ref_rects,
            )
        })
        || ancestor_fieldset_disabled(entity, parents, semantics, fieldsets)
}

fn ancestor_fieldset_disabled(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    semantics: &Query<&DeclarativeContainerSemantic>,
    fieldsets: &Query<&DeclarativeFieldsetState>,
) -> bool {
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        if semantics
            .get(entity)
            .is_ok_and(|semantic| semantic.kind == DeclarativeContainerKind::Fieldset)
            && fieldsets.get(entity).is_ok_and(|fieldset| fieldset.disabled)
        {
            return true;
        }
    }
    false
}
