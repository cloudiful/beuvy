use crate::ast::DeclarativeEventKind;
use crate::runtime::state::{
    DeclarativeEventBindings, DeclarativeUiEventMessage, ResolvedDeclarativeEventBinding,
};
use beuvy_runtime::button::ButtonClickMessage;
use beuvy_runtime::input::{InputValueChangedMessage, InputValueCommittedMessage};
use beuvy_runtime::select::SelectValueChangedMessage;
use bevy::prelude::*;

pub(crate) fn dispatch_declarative_control_events(
    mut button_clicks: MessageReader<ButtonClickMessage>,
    mut input_changes: MessageReader<InputValueChangedMessage>,
    mut input_commits: MessageReader<InputValueCommittedMessage>,
    mut select_changes: MessageReader<SelectValueChangedMessage>,
    bindings: Query<&DeclarativeEventBindings>,
    mut output: MessageWriter<DeclarativeUiEventMessage>,
) {
    for click in button_clicks.read() {
        dispatch_matching(
            click.entity,
            DeclarativeEventKind::Activate,
            &bindings,
            &mut output,
        );
    }
    for change in input_changes.read() {
        dispatch_matching(change.entity, DeclarativeEventKind::Input, &bindings, &mut output);
    }
    for change in input_commits.read() {
        dispatch_matching(change.entity, DeclarativeEventKind::Change, &bindings, &mut output);
    }
    for change in select_changes.read() {
        dispatch_matching(change.entity, DeclarativeEventKind::Change, &bindings, &mut output);
    }
}

fn dispatch_matching(
    entity: Entity,
    kind: DeclarativeEventKind,
    bindings: &Query<&DeclarativeEventBindings>,
    output: &mut MessageWriter<DeclarativeUiEventMessage>,
) {
    let Ok(bindings) = bindings.get(entity) else {
        return;
    };
    for binding in bindings.0.iter().filter(|binding| binding.kind == kind) {
        write_event(entity, binding, output);
    }
}

fn write_event(
    entity: Entity,
    binding: &ResolvedDeclarativeEventBinding,
    output: &mut MessageWriter<DeclarativeUiEventMessage>,
) {
    output.write(DeclarativeUiEventMessage {
        entity,
        kind: binding.kind,
        action_id: binding.action_id.clone(),
        params: binding.params.clone(),
    });
}
