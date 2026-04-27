use super::{ActiveButton, Button, ButtonClickMessage, DisabledButton};
use crate::focus::{UiHovered, UiPressed};
use bevy::prelude::*;

pub(super) fn button_hover_over(
    event: On<Pointer<Over>>,
    mut commands: Commands,
    buttons: Query<Has<DisabledButton>, With<Button>>,
) {
    let Ok(disabled) = buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_insert(UiHovered);
    }
}

pub(super) fn button_hover_out(
    event: On<Pointer<Out>>,
    mut commands: Commands,
    buttons: Query<Has<DisabledButton>, With<Button>>,
) {
    let Ok(disabled) = buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_remove::<UiHovered>();
    }
}

pub(super) fn button_click(
    event: On<Pointer<Click>>,
    mut button_click_message: MessageWriter<ButtonClickMessage>,
    buttons: Query<(&Button, Has<DisabledButton>)>,
) {
    let Ok((button, disabled)) = buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    button_click_message.write(ButtonClickMessage {
        button: button.clone(),
        entity: event.entity,
    });
}

pub(super) fn button_press(
    event: On<Pointer<Press>>,
    mut commands: Commands,
    buttons: Query<Has<DisabledButton>, With<Button>>,
) {
    let Ok(disabled) = buttons.get(event.entity) else {
        return;
    };
    if disabled {
        return;
    }
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_insert(UiPressed);
    }
}

pub fn sync_button_active_state(commands: &mut Commands, entity: Entity, active: bool) {
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    if active {
        entity_commands.try_insert(ActiveButton);
    } else {
        entity_commands.try_remove::<ActiveButton>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_pressed_does_not_create_active_button() {
        let mut world = World::new();
        let entity = world.spawn((Button::default(), UiPressed)).id();

        assert!(world.entity(entity).contains::<UiPressed>());
        assert!(!world.entity(entity).contains::<ActiveButton>());
    }
}
