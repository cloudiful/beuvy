use crate::runtime::state::{DeclarativeLabelForTarget, DeclarativeTextBinding};
use beuvy_runtime::input::{InputField, InputValueChangedMessage};
use bevy::input_focus::InputFocus;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

pub(crate) fn handle_declarative_label_click(
    mut events: MessageReader<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    labels: Query<&DeclarativeLabelForTarget>,
    targets: Query<(Entity, &Name), With<InputField>>,
    mut fields: Query<&mut InputField>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    for event in events.read() {
        if event.button != PointerButton::Primary {
            continue;
        }
        let Ok(label) = labels.get(event.entity) else {
            continue;
        };
        let Some((target_entity, _)) = targets
            .iter()
            .find(|(_, name)| name.as_str() == label.0.as_str())
        else {
            continue;
        };
        if let Ok(mut field) = fields.get_mut(target_entity) {
            if field.input_type == beuvy_runtime::input::InputType::Checkbox {
                field.checked = !field.checked;
                value_changed.write(InputValueChangedMessage {
                    entity: target_entity,
                    name: field.name.clone(),
                    value: field.submitted_value(),
                });
            } else if field.input_type == beuvy_runtime::input::InputType::Radio {
                field.checked = true;
                value_changed.write(InputValueChangedMessage {
                    entity: target_entity,
                    name: field.name.clone(),
                    value: field.submitted_value(),
                });
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
            With<DeclarativeTextBinding>,
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
