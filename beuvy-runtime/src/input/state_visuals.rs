use super::metrics::{node_global_rect, text_x_for_byte};
use crate::focus::UiFocused;
use crate::input::{
    DisabledInput, InputCaret, InputCursorPosition, InputField, InputScrollOffset, InputSelection,
    InputText, InputType,
};
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::text::TextLayoutInfo;
use bevy::window::PrimaryWindow;

pub(crate) fn sync_input_focus_visuals(
    mut commands: Commands,
    input_focus: Res<bevy::input_focus::InputFocus>,
    fields: Query<(Entity, Option<&UiGlobalTransform>, Option<&ComputedNode>), With<InputField>>,
    added_fields: Query<(), Added<InputField>>,
    focused: Query<Entity, With<UiFocused>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if !input_focus.is_changed() && added_fields.is_empty() {
        return;
    }

    let focus_target = input_focus.get();

    for (entity, transform, computed) in &fields {
        let should_focus = focus_target == Some(entity);
        let is_focused = focused.contains(entity);

        if should_focus == is_focused {
            continue;
        }

        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };

        if should_focus {
            entity_commands.try_insert(UiFocused);
            if let (Some(transform), Some(computed)) = (transform, computed) {
                let (_, _, center) = transform.to_scale_angle_translation();
                let rect = Rect::from_center_size(center, computed.size());
                let inverse = computed.inverse_scale_factor();
                let content_inset = computed.content_inset();
                let content_left = rect.min.x + content_inset.min_inset.x;
                entity_commands.try_insert(InputCursorPosition {
                    x: content_left * inverse,
                    y: rect.center().y * inverse,
                });
            }
        } else {
            entity_commands.try_remove::<UiFocused>();
        }
    }

    let primary_window = primary_window.iter().next();
    if focus_target.is_some() && focus_target == primary_window {
        for entity in &focused {
            let Ok(mut entity_commands) = commands.get_entity(entity) else {
                continue;
            };
            entity_commands.try_remove::<UiFocused>();
        }
    }
}

pub(crate) fn sync_input_edit_visuals(
    mut commands: Commands,
    time: Res<Time>,
    fields: Query<
        (
            Entity,
            &InputField,
            Has<DisabledInput>,
            Has<UiFocused>,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<InputField>,
    >,
    text_nodes: Query<(&TextLayoutInfo, &ComputedNode, &UiGlobalTransform), With<InputText>>,
    mut visuals: ParamSet<(
        Query<(&mut InputScrollOffset, &mut Node)>,
        Query<(&mut Node, &mut Visibility), With<InputSelection>>,
        Query<(&mut Node, &mut Visibility), With<InputCaret>>,
    )>,
) {
    for (entity, field, disabled, focused, input_computed, input_transform) in &fields {
        if matches!(field.input_type, InputType::Range) || field.text_entity == Entity::PLACEHOLDER
        {
            continue;
        }
        let display_text = field.edit_state.display_text_string(&field.placeholder);
        let Ok((layout, text_computed, text_transform)) = text_nodes.get(field.text_entity) else {
            continue;
        };

        let input_rect = node_global_rect(input_computed, input_transform);
        let text_rect = node_global_rect(text_computed, text_transform);
        let input_scale = input_computed.inverse_scale_factor();
        let text_scale = text_computed.inverse_scale_factor();
        let text_origin = text_rect.min * text_scale - input_rect.min * input_scale;
        let text_height = layout.size.y.max(text_computed.size().y * text_scale);
        let caret_width = 2.0;
        let input_inset = input_computed.content_inset();
        let input_content_width = (input_computed.size().x * input_scale
            - input_inset.min_inset.x
            - input_inset.max_inset.x)
            .max(0.0);
        let raw_caret_x =
            text_x_for_byte(layout, &display_text.text, field.edit_state.display_caret_byte());
        if focused && !disabled {
            if let Ok((mut scroll_offset, mut text_node)) = visuals.p0().get_mut(field.text_entity)
            {
                let max_offset = (layout.size.x - input_content_width).max(0.0);
                if raw_caret_x < scroll_offset.x {
                    scroll_offset.x = raw_caret_x.max(0.0);
                } else if raw_caret_x > scroll_offset.x + input_content_width {
                    scroll_offset.x = (raw_caret_x - input_content_width).max(0.0);
                }
                scroll_offset.x = scroll_offset.x.min(max_offset);
                text_node.left = Val::Px(-scroll_offset.x);
            }
        }
        let caret_x = text_origin.x + raw_caret_x;
        let caret_top = text_origin.y;
        let caret_center_y = caret_top + text_height * 0.5;

        if field.selection_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p1().get_mut(field.selection_entity) {
                if let Some(range) = field.edit_state.selection_range() {
                    let start_x =
                        text_origin.x + text_x_for_byte(layout, &display_text.text, range.start);
                    let end_x =
                        text_origin.x + text_x_for_byte(layout, &display_text.text, range.end);
                    node.left = Val::Px(start_x);
                    node.top = Val::Px(caret_top);
                    node.width = Val::Px((end_x - start_x).max(0.0));
                    node.height = Val::Px(text_height.max(0.0));
                    *visibility = if end_x > start_x {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }

        if field.caret_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p2().get_mut(field.caret_entity) {
                node.left = Val::Px((caret_x - caret_width * 0.5).max(0.0));
                node.top = Val::Px(caret_top);
                node.height = Val::Px(text_height.max(0.0));
                let blink_visible = time.elapsed_secs_f64() < field.caret_blink_resume_at
                    || (time.elapsed_secs_f64() * 2.0).floor() as i64 % 2 == 0;
                *visibility = if focused && !disabled && blink_visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        if focused && !disabled {
            if let Ok(mut entity_commands) = commands.get_entity(entity) {
                entity_commands.try_insert(InputCursorPosition {
                    x: (input_rect.min.x * input_scale + caret_x).max(0.0),
                    y: caret_center_y.max(0.0),
                });
            }
        }
    }
}
