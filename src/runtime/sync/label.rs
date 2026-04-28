use crate::runtime::state::{DeclarativeLabelForTarget, DeclarativeLabelNode};
use beuvy_runtime::input::{InputField, InputValueChangedMessage};
use bevy::input_focus::InputFocus;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

pub(crate) fn handle_declarative_label_click(
    mut events: MessageReader<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    labels: Query<&DeclarativeLabelForTarget>,
    mut fields: ParamSet<(
        Query<(Entity, &Name, &InputField), With<InputField>>,
        Query<&mut InputField>,
    )>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    for event in events.read() {
        if event.button != PointerButton::Primary {
            continue;
        }
        let Ok(label) = labels.get(event.entity) else {
            continue;
        };
        let Some((target_entity, target_input_type, target_checked, target_name)) = fields
            .p0()
            .iter()
            .find(|(_, name, _)| name.as_str() == label.0.as_str())
            .map(|(entity, _, field)| (entity, field.input_type, field.checked, field.name.clone()))
        else {
            continue;
        };
        if let Ok(mut field) = fields.p1().get_mut(target_entity) {
            if field.input_type == beuvy_runtime::input::InputType::Checkbox {
                field.checked = !field.checked;
                value_changed.write(InputValueChangedMessage {
                    entity: target_entity,
                    name: field.name.clone(),
                    value: field.submitted_value(),
                    runtime_value: beuvy_runtime::input::InputRuntimeValue::Bool(field.checked),
                });
            } else if target_input_type == beuvy_runtime::input::InputType::Radio && !target_checked {
                let radio_group = target_name;
                let radio_targets = fields
                    .p0()
                    .iter()
                    .filter_map(|(entity, _, field)| {
                        (field.input_type == beuvy_runtime::input::InputType::Radio
                            && field.name == radio_group)
                            .then_some(entity)
                    })
                    .collect::<Vec<_>>();
                for radio_target in radio_targets {
                    if let Ok(mut radio_field) = fields.p1().get_mut(radio_target) {
                        let next_checked = radio_target == target_entity;
                        if radio_field.checked != next_checked {
                            radio_field.checked = next_checked;
                            value_changed.write(InputValueChangedMessage {
                                entity: radio_target,
                                name: radio_field.name.clone(),
                                value: radio_field.submitted_value(),
                                runtime_value: beuvy_runtime::input::InputRuntimeValue::Text(
                                    radio_field.value().to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }
        input_focus.set(target_entity);
    }
}

pub(crate) fn infer_wrapped_label_targets(
    mut commands: Commands,
    labels: Query<
        (Entity, &Children),
        (
            With<DeclarativeLabelNode>,
            Without<DeclarativeLabelForTarget>,
        ),
    >,
    inputs: Query<&Name, With<InputField>>,
) {
    for (entity, children) in &labels {
        let Some(target) = children
            .iter()
            .find_map(|child| inputs.get(child).ok().map(|name| name.as_str().to_string()))
        else {
            continue;
        };
        if let Ok(mut entity_commands) = commands.get_entity(entity) {
            entity_commands.try_insert(DeclarativeLabelForTarget(target));
        }
    }
}
