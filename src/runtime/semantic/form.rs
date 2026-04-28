use super::{nearest_ancestor, walk_descendants};
use crate::ast::DeclarativeContainerKind;
use crate::runtime::state::{
    DeclarativeContainerSemantic, DeclarativeFormResetMessage, DeclarativeFormSubmitMessage,
};
use crate::value::UiValue;
use beuvy_runtime::button::{Button, ButtonClickMessage, ButtonType};
use beuvy_runtime::input::{InputField, InputSubmitMessage};
use beuvy_runtime::select::Select;
use bevy::prelude::*;

pub(crate) fn handle_form_button_clicks(
    mut button_clicks: MessageReader<ButtonClickMessage>,
    parents: Query<&ChildOf>,
    semantics: Query<&DeclarativeContainerSemantic>,
    children: Query<&Children>,
    buttons: Query<&Button>,
    inputs: Query<&InputField>,
    mut inputs_mut: Query<&mut InputField>,
    selects: Query<&Select>,
    mut selects_mut: Query<&mut Select>,
    mut submit_messages: MessageWriter<DeclarativeFormSubmitMessage>,
    mut reset_messages: MessageWriter<DeclarativeFormResetMessage>,
) {
    for click in button_clicks.read() {
        let Ok(button) = buttons.get(click.entity) else {
            continue;
        };
        let Some(form) = nearest_form(click.entity, &parents, &semantics) else {
            continue;
        };
        match button.button_type {
            ButtonType::Submit => {
                submit_messages.write(DeclarativeFormSubmitMessage {
                    entity: form,
                    values: collect_form_values(form, &children, &inputs, &selects),
                });
            }
            ButtonType::Reset => {
                reset_form_values(form, &children, &mut inputs_mut, &mut selects_mut);
                reset_messages.write(DeclarativeFormResetMessage { entity: form });
            }
            ButtonType::Button => {}
        }
    }
}

pub(crate) fn handle_form_input_submits(
    mut input_submits: MessageReader<InputSubmitMessage>,
    parents: Query<&ChildOf>,
    semantics: Query<&DeclarativeContainerSemantic>,
    children: Query<&Children>,
    inputs: Query<&InputField>,
    selects: Query<&Select>,
    mut submit_messages: MessageWriter<DeclarativeFormSubmitMessage>,
) {
    for submit in input_submits.read() {
        let Some(form) = nearest_form(submit.entity, &parents, &semantics) else {
            continue;
        };
        submit_messages.write(DeclarativeFormSubmitMessage {
            entity: form,
            values: collect_form_values(form, &children, &inputs, &selects),
        });
    }
}

fn nearest_form(
    entity: Entity,
    parents: &Query<&ChildOf>,
    semantics: &Query<&DeclarativeContainerSemantic>,
) -> Option<Entity> {
    nearest_ancestor(entity, parents, |candidate| {
        semantics
            .get(candidate)
            .is_ok_and(|semantic| semantic.kind == DeclarativeContainerKind::Form)
    })
}

fn collect_form_values(
    form: Entity,
    children: &Query<&Children>,
    inputs: &Query<&InputField>,
    selects: &Query<&Select>,
) -> std::collections::HashMap<String, UiValue> {
    let mut values = std::collections::HashMap::new();
    walk_descendants(form, children, &mut |entity| {
        if let Ok(input) = inputs.get(entity) {
            let value = match input.input_type {
                beuvy_runtime::input::InputType::Checkbox => UiValue::from(input.checked),
                beuvy_runtime::input::InputType::Radio => {
                    if input.checked {
                        UiValue::from(input.value().to_string())
                    } else {
                        return;
                    }
                }
                beuvy_runtime::input::InputType::Number
                | beuvy_runtime::input::InputType::Range => input
                    .numeric_value()
                    .map(UiValue::from)
                    .unwrap_or_else(|| UiValue::from(input.value().to_string())),
                beuvy_runtime::input::InputType::Text
                | beuvy_runtime::input::InputType::Textarea
                | beuvy_runtime::input::InputType::Password => {
                    UiValue::from(input.value().to_string())
                }
            };
            values.insert(input.name.clone(), value);
            return;
        }
        if let Ok(select) = selects.get(entity) {
            values.insert(select.name.clone(), UiValue::from(select.value.clone()));
        }
    });
    values
}

fn reset_form_values(
    form: Entity,
    children: &Query<&Children>,
    inputs: &mut Query<&mut InputField>,
    selects: &mut Query<&mut Select>,
) {
    walk_descendants(form, children, &mut |entity| {
        if let Ok(mut input) = inputs.get_mut(entity) {
            input.reset();
            return;
        }
        if let Ok(mut select) = selects.get_mut(entity) {
            select.value = select.initial_value.clone();
        }
    });
}
