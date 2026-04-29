use super::resolve::resolve_runtime_path;
use crate::runtime::state::{
    DeclarativeCheckedBinding, DeclarativeLocalState, DeclarativeModelBinding,
    DeclarativeRefRects, DeclarativeRootComputedLocals, DeclarativeRootViewModel,
    DeclarativeUiRuntimeValues, DeclarativeValueBinding,
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
        Option<&DeclarativeCheckedBinding>,
        &mut InputField,
        Has<DisabledInput>,
    )>,
    mut selects: Query<(Entity, &DeclarativeValueBinding, &mut Select)>,
) {
    for (entity, binding, checked_binding, mut field, disabled) in &mut inputs {
        if matches!(field.input_type, InputType::Checkbox) {
            let binding_path = checked_binding.map(|binding| binding.0.as_str()).unwrap_or(&binding.0);
            if let Some(next_checked) = resolve_runtime_path(
                entity,
                binding_path,
                &parents,
                &local_states,
                &computed,
                &roots,
                &values,
                &ref_rects,
            )
            .and_then(|value| value.bool())
            {
                field.checked = next_checked;
            }
            continue;
        }
        if matches!(field.input_type, InputType::Radio) {
            let binding_path = checked_binding.map(|binding| binding.0.as_str()).unwrap_or(&binding.0);
            if let Some(next_value) = resolve_runtime_path(
                entity,
                binding_path,
                &parents,
                &local_states,
                &computed,
                &roots,
                &values,
                &ref_rects,
            )
            .and_then(|value| value.text().map(str::to_string))
            {
                field.checked = field.input_value.as_deref() == Some(next_value.as_str());
            }
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
            InputType::Text | InputType::Textarea | InputType::Password => {
                value.text().map(str::to_string)
            }
            InputType::Number | InputType::Range => value
                .number()
                .map(|value| value.to_string())
                .or_else(|| value.text().map(str::to_string)),
            InputType::Checkbox | InputType::Radio => None,
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
    bindings: Query<
        (&DeclarativeValueBinding, &InputField),
        (With<InputField>, With<DeclarativeModelBinding>),
    >,
) {
    for change in input_changes.read() {
        let Ok((binding, field)) = bindings.get(change.entity) else {
            continue;
        };
        match field.input_type {
            InputType::Checkbox => values.set(binding.0.clone(), field.checked),
            InputType::Radio => values.set(binding.0.clone(), change.value.clone()),
            _ => values.set(binding.0.clone(), change.value.clone()),
        }
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
