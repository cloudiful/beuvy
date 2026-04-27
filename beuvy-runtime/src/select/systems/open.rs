use crate::button::DisabledButton;
use crate::select::model::{
    Select, SelectOptionButton, SelectPanel, SelectTrigger, SelectValueChangedMessage,
};
use bevy::prelude::*;

pub(crate) fn close_selects_on_foreign_click(
    mut pointer_clicks: MessageReader<Pointer<Click>>,
    trigger_buttons: Query<(), With<SelectTrigger>>,
    option_buttons: Query<(), With<SelectOptionButton>>,
    select_entities: Query<Entity, With<Select>>,
    mut select_states: Query<&mut Select>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
) {
    let Some(click) = pointer_clicks.read().next() else {
        return;
    };
    if trigger_buttons.contains(click.entity) || option_buttons.contains(click.entity) {
        return;
    }

    for entity in &select_entities {
        set_select_open(&mut select_states, &mut panel_nodes, entity, false);
    }
}

pub(crate) fn select_trigger_click(
    event: On<Pointer<Click>>,
    trigger_buttons: Query<(&SelectTrigger, Has<DisabledButton>)>,
    select_entities: Query<Entity, With<Select>>,
    mut select_states: Query<&mut Select>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
) {
    let Ok((trigger, disabled)) = trigger_buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }

    let should_open = select_states
        .get(trigger.select)
        .map(|select| !select.open && !select.disabled)
        .unwrap_or(false);

    for entity in &select_entities {
        set_select_open(
            &mut select_states,
            &mut panel_nodes,
            entity,
            entity == trigger.select && should_open,
        );
    }
}

pub(crate) fn select_option_click(
    event: On<Pointer<Click>>,
    option_buttons: Query<(&SelectOptionButton, Has<DisabledButton>)>,
    mut select_states: Query<&mut Select>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
    mut value_changes: MessageWriter<SelectValueChangedMessage>,
) {
    let Ok((option, disabled)) = option_buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }

    {
        let Ok(mut select) = select_states.get_mut(option.select) else {
            return;
        };
        if select.value != option.value {
            select.value = option.value.clone();
            value_changes.write(SelectValueChangedMessage {
                entity: option.select,
                name: select.name.clone(),
                value: select.value.clone(),
            });
        }
    }

    set_select_open(&mut select_states, &mut panel_nodes, option.select, false);
}

pub(super) fn set_select_open(
    selects: &mut Query<&mut Select>,
    panel_nodes: &mut Query<&mut Node, With<SelectPanel>>,
    entity: Entity,
    open: bool,
) {
    let Ok(mut select) = selects.get_mut(entity) else {
        return;
    };

    if select.open == open {
        return;
    }

    select.open = open;

    if let Ok(mut panel_node) = panel_nodes.get_mut(select.panel) {
        panel_node.display = if open { Display::Flex } else { Display::None };
    }
}
