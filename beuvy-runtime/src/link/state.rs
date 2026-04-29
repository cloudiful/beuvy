use super::{DisabledLink, Link, LinkActivatedMessage};
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input_focus::InputFocus;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;

pub(super) fn link_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    mut activated: MessageWriter<LinkActivatedMessage>,
    links: Query<(&Link, Has<DisabledLink>)>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok((link, disabled)) = links.get(event.entity) else {
        return;
    };
    input_focus.set(event.entity);
    if disabled {
        event.propagate(false);
        return;
    }
    activated.write(LinkActivatedMessage {
        entity: event.entity,
        name: link.name.clone(),
        href: link.href.clone(),
    });
    event.propagate(false);
}

pub(super) fn link_keyboard_activate(
    mut keyboard_inputs: MessageReader<KeyboardInput>,
    input_focus: Res<InputFocus>,
    mut activated: MessageWriter<LinkActivatedMessage>,
    links: Query<(&Link, Has<DisabledLink>)>,
) {
    let Some(active) = input_focus.get() else {
        return;
    };
    let Ok((link, disabled)) = links.get(active) else {
        return;
    };
    if disabled {
        return;
    }
    for input in keyboard_inputs.read() {
        if input.state != ButtonState::Pressed {
            continue;
        }
        let activate = match &input.logical_key {
            Key::Enter => true,
            Key::Character(value) => value == " ",
            _ => false,
        };
        if activate {
            activated.write(LinkActivatedMessage {
                entity: active,
                name: link.name.clone(),
                href: link.href.clone(),
            });
        }
    }
}
