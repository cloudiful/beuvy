use super::walk_descendants;
use crate::ast::DeclarativeContainerKind;
use crate::runtime::state::{DeclarativeContainerSemantic, DeclarativeNodeId};
use beuvy_runtime::input::{InputField, InputRuntimeValue, InputValueChangedMessage};
use beuvy_runtime::select::{Select, SelectPanel};
use bevy::input_focus::InputFocus;
use bevy::prelude::*;

pub(crate) fn handle_label_clicks(
    mut clicks: MessageReader<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    semantics: Query<&DeclarativeContainerSemantic>,
    children: Query<&Children>,
    node_ids: Query<(Entity, &DeclarativeNodeId)>,
    mut inputs: Query<(Entity, &mut InputField)>,
    mut selects: Query<(Entity, &mut Select)>,
    mut panels: Query<&mut Node, With<SelectPanel>>,
    mut input_changes: MessageWriter<InputValueChangedMessage>,
) {
    for click in clicks.read() {
        let Ok(semantic) = semantics.get(click.entity) else {
            continue;
        };
        if semantic.kind != DeclarativeContainerKind::Label {
            continue;
        }

        if let Some(target) = semantic
            .label_for
            .as_deref()
            .and_then(|name| resolve_named_target(name, &inputs, &selects, &node_ids))
            .or_else(|| resolve_nested_target(click.entity, &children, &inputs, &selects))
        {
            activate_label_target(
                target,
                &mut input_focus,
                &mut inputs,
                &mut selects,
                &mut panels,
                &mut input_changes,
            );
        }
    }
}

fn resolve_named_target(
    name: &str,
    inputs: &Query<(Entity, &mut InputField)>,
    selects: &Query<(Entity, &mut Select)>,
    node_ids: &Query<(Entity, &DeclarativeNodeId)>,
) -> Option<Entity> {
    for (entity, input) in inputs.iter() {
        if input.name == name {
            return Some(entity);
        }
    }
    for (entity, select) in selects.iter() {
        if select.name == name {
            return Some(entity);
        }
    }
    for (entity, node_id) in node_ids.iter() {
        if node_id.0 == name {
            return Some(entity);
        }
    }
    None
}

fn resolve_nested_target(
    label: Entity,
    children: &Query<&Children>,
    inputs: &Query<(Entity, &mut InputField)>,
    selects: &Query<(Entity, &mut Select)>,
) -> Option<Entity> {
    let mut target = None;
    walk_descendants(label, children, &mut |entity| {
        if target.is_some() {
            return;
        }
        if inputs.contains(entity) || selects.contains(entity) {
            target = Some(entity);
        }
    });
    target
}

fn activate_label_target(
    target: Entity,
    input_focus: &mut ResMut<InputFocus>,
    inputs: &mut Query<(Entity, &mut InputField)>,
    selects: &mut Query<(Entity, &mut Select)>,
    panels: &mut Query<&mut Node, With<SelectPanel>>,
    input_changes: &mut MessageWriter<InputValueChangedMessage>,
) {
    if let Ok((entity, mut input)) = inputs.get_mut(target) {
        input_focus.set(entity);
        if input.is_toggle() {
            match input.input_type {
                beuvy_runtime::input::InputType::Checkbox => input.checked = !input.checked,
                beuvy_runtime::input::InputType::Radio => input.checked = true,
                _ => {}
            }
            let runtime_value = match input.input_type {
                beuvy_runtime::input::InputType::Checkbox => InputRuntimeValue::Bool(input.checked),
                beuvy_runtime::input::InputType::Radio => {
                    InputRuntimeValue::Text(input.value().to_string())
                }
                beuvy_runtime::input::InputType::Number
                | beuvy_runtime::input::InputType::Range => input
                    .numeric_value()
                    .map(|value| InputRuntimeValue::Number(value as f64))
                    .unwrap_or_else(|| InputRuntimeValue::Text(input.value().to_string())),
                beuvy_runtime::input::InputType::Text
                | beuvy_runtime::input::InputType::Textarea
                | beuvy_runtime::input::InputType::Password => {
                    InputRuntimeValue::Text(input.value().to_string())
                }
            };
            input_changes.write(InputValueChangedMessage {
                entity,
                name: input.name.clone(),
                value: input.value().to_string(),
                runtime_value,
            });
        }
        return;
    }

    if let Ok((_entity, mut select)) = selects.get_mut(target) {
        select.open = true;
        if let Ok(mut panel) = panels.get_mut(select.panel) {
            panel.display = Display::Flex;
        }
    }
}
