use super::resolve::resolve_runtime_path;
use crate::runtime::state::{
    DeclarativeLocalState, DeclarativeModelBinding, DeclarativeRefRects,
    DeclarativeRootComputedLocals, DeclarativeRootViewModel, DeclarativeUiRuntimeValues,
    DeclarativeValueBinding,
};
use beuvy_runtime::button::ButtonLabel;
use beuvy_runtime::input::{
    DisabledInput, InputField, InputType, InputValueChangedMessage, set_input_value,
};
use beuvy_runtime::text::FontResource;
use beuvy_runtime::{
    Select, SelectValueChangedMessage, selected_option, sync_select_label, trigger_label_entity,
};
use bevy::prelude::*;

pub(crate) fn sync_declarative_field_values(
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
    mut inputs: Query<(
        Entity,
        &DeclarativeValueBinding,
        &mut InputField,
        Has<DisabledInput>,
    )>,
    mut selects: Query<(Entity, &DeclarativeValueBinding, &mut Select)>,
) {
    for (entity, binding, mut field, disabled) in &mut inputs {
        if field.focused && field.dirty_since_focus {
            continue;
        }
        let Some(value) = resolve_runtime_path(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        )
        .and_then(|value| match field.input_type {
            InputType::Text => value.text().map(str::to_string),
            InputType::Number | InputType::Range => value
                .number()
                .map(|value| value.to_string())
                .or_else(|| value.text().map(str::to_string)),
        }) else {
            continue;
        };
        set_input_value(&mut commands, &font_resource, &mut field, disabled, value);
    }

    for (entity, binding, mut select) in &mut selects {
        let Some(value) = resolve_runtime_path(
            entity,
            &binding.0,
            &parents,
            &local_states,
            &computed,
            &roots,
            &values,
            &ref_rects,
        )
        .and_then(|value| value.text().map(str::to_string)) else {
            continue;
        };
        if select.value == value {
            continue;
        }
        select.value = value;
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

pub(crate) fn write_input_values_to_runtime_store(
    mut input_changes: MessageReader<InputValueChangedMessage>,
    mut values: ResMut<DeclarativeUiRuntimeValues>,
    bindings: Query<&DeclarativeValueBinding, (With<InputField>, With<DeclarativeModelBinding>)>,
) {
    for change in input_changes.read() {
        let Ok(binding) = bindings.get(change.entity) else {
            continue;
        };
        values.set(binding.0.clone(), change.value.clone());
    }
}

pub(crate) fn write_select_values_to_runtime_store(
    mut select_changes: MessageReader<SelectValueChangedMessage>,
    mut values: ResMut<DeclarativeUiRuntimeValues>,
    bindings: Query<&DeclarativeValueBinding, (With<Select>, With<DeclarativeModelBinding>)>,
) {
    for change in select_changes.read() {
        let Ok(binding) = bindings.get(change.entity) else {
            continue;
        };
        values.set(binding.0.clone(), change.value.clone());
    }
}
