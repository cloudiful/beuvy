use crate::focus::{UiHovered, UiPressed};
use bevy::prelude::*;

pub fn pointer_hover_over(event: On<Pointer<Over>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_insert(UiHovered);
    }
}

pub fn pointer_hover_out(event: On<Pointer<Out>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_remove::<UiHovered>();
        entity_commands.try_remove::<UiPressed>();
    }
}

pub fn pointer_press(event: On<Pointer<Press>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_insert(UiPressed);
    }
}

pub fn pointer_release(event: On<Pointer<Release>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_remove::<UiPressed>();
    }
}

pub fn pointer_cancel(event: On<Pointer<Cancel>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_remove::<UiPressed>();
    }
}

pub fn pointer_drag_end(event: On<Pointer<DragEnd>>, mut commands: Commands) {
    if let Ok(mut entity_commands) = commands.get_entity(event.entity) {
        entity_commands.try_remove::<UiPressed>();
    }
}
