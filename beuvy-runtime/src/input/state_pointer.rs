use super::metrics::node_logical_rect;
use super::{keep_caret_visible, set_checkable_state, shift_pressed};
use crate::input::{
    DisabledInput, InputClickState, InputField, InputScrollOffset, InputTextEngine,
    InputValueChangedMessage, clear_input_focus, set_input_focus,
};
use crate::text::FontResource;
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::input_focus::InputFocus;
use bevy::picking::pointer::PointerButton;
use bevy::prelude::*;
use bevy::text::ComputedTextBlock;

const DRAG_SELECT_THRESHOLD: f32 = 3.0;

pub(crate) fn input_click(
    mut event: On<Pointer<Click>>,
    mut input_focus: ResMut<InputFocus>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    text_engine: Res<InputTextEngine>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
    mut fields: Query<
        (&mut InputField, &mut InputClickState),
        (With<InputField>, Without<DisabledInput>),
    >,
    sibling_fields: Query<(Entity, &InputField), With<InputField>>,
    text_blocks: Query<(&ComputedTextBlock, &InputScrollOffset), With<crate::input::InputText>>,
    viewports: Query<(&ComputedNode, &UiGlobalTransform), With<crate::input::InputViewport>>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let (input_type, checked, group_name) = {
        let Ok((field, _)) = fields.get(event.entity) else {
            return;
        };
        (field.input_type, field.checked, field.name.clone())
    };

    set_input_focus(&mut input_focus, event.entity);
    if input_type == crate::input::InputType::Checkbox {
        if let Ok((mut field, _)) = fields.get_mut(event.entity) {
            let next_checked = !checked;
            set_checkable_state(
                event.entity,
                &mut field,
                next_checked,
                &mut value_changed,
            );
        }
        event.propagate(false);
        return;
    }
    if input_type == crate::input::InputType::Radio {
        let targets = sibling_fields
            .iter()
            .filter_map(|(entity, sibling)| {
                (sibling.input_type == crate::input::InputType::Radio && sibling.name == group_name)
                    .then_some(entity)
            })
            .collect::<Vec<_>>();
        for entity in targets {
            if let Ok((mut sibling_field, _)) = fields.get_mut(entity) {
                let next_checked = entity == event.entity;
                set_checkable_state(
                    entity,
                    &mut sibling_field,
                    next_checked,
                    &mut value_changed,
                );
            }
        }
        event.propagate(false);
        return;
    }
    let Ok((mut field, mut click_state)) = fields.get_mut(event.entity) else {
        return;
    };
    let (Some(viewport_entity), Some(text_entity)) = (field.viewport_entity, field.text_entity)
    else {
        return;
    };
    let Ok((block, scroll_offset)) = text_blocks.get(text_entity) else {
        return;
    };
    let Ok((computed, transform)) = viewports.get(viewport_entity) else {
        return;
    };
    let inverse_scale_factor = computed.inverse_scale_factor();

    let now = time.elapsed_secs_f64();
    let is_chained_click = now - click_state.last_click_time <= 0.5;
    click_state.click_count = if is_chained_click {
        click_state.click_count.saturating_add(1).min(3)
    } else {
        1
    };
    click_state.last_click_time = now;

    let logical_rect = node_logical_rect(computed, transform);
    let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let click_position = event.pointer_location.position;
    let is_same_click_spot = click_state
        .last_click_position
        .is_some_and(|last| last.distance(click_position) <= DRAG_SELECT_THRESHOLD);
    let byte = text_engine.hit_byte(
        block,
        inverse_scale_factor,
        local_x + scroll_offset.x,
        local_y + scroll_offset.y,
    );
    match click_state.click_count {
        1 => field.edit_state.set_caret(byte, shift_pressed(&keys)),
        2 => {
            if is_same_click_spot {
                field.edit_state.select_word_at(byte);
            } else {
                click_state.click_count = 1;
                field.edit_state.set_caret(byte, shift_pressed(&keys));
            }
        }
        _ => {
            if is_same_click_spot {
                field.edit_state.select_all();
            } else {
                click_state.click_count = 1;
                field.edit_state.set_caret(byte, shift_pressed(&keys));
            }
        }
    }
    click_state.last_click_position = Some(click_position);
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag_start(
    mut event: On<Pointer<DragStart>>,
    mut input_focus: ResMut<InputFocus>,
    time: Res<Time>,
    text_engine: Res<InputTextEngine>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_blocks: Query<(&ComputedTextBlock, &InputScrollOffset), With<crate::input::InputText>>,
    viewports: Query<(&ComputedNode, &UiGlobalTransform), With<crate::input::InputViewport>>,
) {
    if event.button != PointerButton::Primary {
        return;
    }
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    set_input_focus(&mut input_focus, event.entity);
    let (Some(viewport_entity), Some(text_entity)) = (field.viewport_entity, field.text_entity)
    else {
        return;
    };
    let Ok((block, scroll_offset)) = text_blocks.get(text_entity) else {
        return;
    };
    let Ok((computed, transform)) = viewports.get(viewport_entity) else {
        return;
    };
    let inverse_scale_factor = computed.inverse_scale_factor();

    let logical_rect = node_logical_rect(computed, transform);
    let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let byte = text_engine.hit_byte(
        block,
        inverse_scale_factor,
        local_x + scroll_offset.x,
        local_y + scroll_offset.y,
    );
    field.edit_state.set_caret(byte, false);
    field.preferred_caret_x = Some(local_x);
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag(
    mut event: On<Pointer<Drag>>,
    time: Res<Time>,
    text_engine: Res<InputTextEngine>,
    mut fields: Query<&mut InputField, (With<InputField>, Without<DisabledInput>)>,
    text_blocks: Query<(&ComputedTextBlock, &InputScrollOffset), With<crate::input::InputText>>,
    viewports: Query<(&ComputedNode, &UiGlobalTransform), With<crate::input::InputViewport>>,
) {
    if event.button != PointerButton::Primary || event.distance.length() < DRAG_SELECT_THRESHOLD {
        return;
    }
    let Ok(mut field) = fields.get_mut(event.entity) else {
        return;
    };
    let (Some(viewport_entity), Some(text_entity)) = (field.viewport_entity, field.text_entity)
    else {
        return;
    };
    let Ok((block, scroll_offset)) = text_blocks.get(text_entity) else {
        return;
    };
    let Ok((computed, transform)) = viewports.get(viewport_entity) else {
        return;
    };
    let inverse_scale_factor = computed.inverse_scale_factor();

    let logical_rect = node_logical_rect(computed, transform);
    let local_x = (event.pointer_location.position.x - logical_rect.min.x).max(0.0);
    let local_y = (event.pointer_location.position.y - logical_rect.min.y).max(0.0);
    let byte = text_engine.hit_byte(
        block,
        inverse_scale_factor,
        local_x + scroll_offset.x,
        local_y + scroll_offset.y,
    );
    field.edit_state.set_caret(byte, true);
    field.preferred_caret_x = Some(local_x);
    keep_caret_visible(&mut field, &time);
    event.propagate(false);
}

pub(crate) fn input_drag_end(mut event: On<Pointer<DragEnd>>) {
    if event.button != PointerButton::Primary {
        return;
    }
    event.propagate(false);
}

pub(crate) fn clear_input_focus_on_foreign_click(
    mut commands: Commands,
    mut pointer_clicks: MessageReader<Pointer<Click>>,
    mut fields: Query<(&mut InputField, Has<DisabledInput>)>,
    mut input_focus: ResMut<InputFocus>,
    font_resource: Res<FontResource>,
    mut value_changed: MessageWriter<InputValueChangedMessage>,
) {
    let Some(active) = input_focus.get().filter(|entity| fields.contains(*entity)) else {
        return;
    };

    for click in pointer_clicks.read() {
        if click.entity == active {
            return;
        }
        if let Ok((mut field, disabled)) = fields.get_mut(active) {
            if super::commit_numeric_field(active, &mut field, &mut value_changed) {
                crate::input::text::update_input_text(
                    &mut commands,
                    &font_resource,
                    &field,
                    disabled,
                );
            }
        }
        clear_input_focus(&mut input_focus);
        return;
    }
}
