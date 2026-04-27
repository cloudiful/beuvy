use super::metrics::{
    caret_geometry_for_byte, node_global_rect, selection_rects_for_range, text_x_for_byte,
};
use crate::focus::UiFocused;
use crate::input::{
    DisabledInput, InputCaret, InputCursorPosition, InputField, InputScrollOffset, InputSelection,
    InputSelectionSegment, InputText, InputType,
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
    text_nodes: Query<&TextLayoutInfo, With<InputText>>,
    mut visuals: ParamSet<(
        Query<(&mut InputScrollOffset, &mut Node)>,
        Query<(&mut Node, &mut Visibility), With<InputSelection>>,
        Query<(&mut Node, &mut Visibility), With<InputCaret>>,
        Query<(&ChildOf, &mut Node, &mut Visibility), With<InputSelectionSegment>>,
    )>,
) {
    for (entity, field, disabled, focused, input_computed, input_transform) in &fields {
        if matches!(field.input_type, InputType::Range) || field.text_entity == Entity::PLACEHOLDER
        {
            continue;
        }
        let display_text = field.edit_state.display_text_string(&field.placeholder);
        let Ok(layout) = text_nodes.get(field.text_entity) else {
            continue;
        };

        let input_rect = node_global_rect(input_computed, input_transform);
        let input_scale = input_computed.inverse_scale_factor();
        let caret_width = 2.0;
        let input_inset = input_computed.content_inset();
        let input_content_width = (input_computed.size().x * input_scale
            - input_inset.min_inset.x
            - input_inset.max_inset.x)
            .max(0.0);
        let input_content_height = (input_computed.size().y * input_scale
            - input_inset.min_inset.y
            - input_inset.max_inset.y)
            .max(0.0);
        let caret_byte = field.edit_state.display_caret_byte();
        let raw_caret_x = text_x_for_byte(layout, &display_text.text, caret_byte);
        let mut scroll_x = 0.0;
        let mut scroll_y = 0.0;
        let (caret_line_x, caret_line_top, caret_line_height) = if field.is_multiline() {
            caret_geometry_for_byte(layout, &display_text.text, caret_byte)
                .unwrap_or((raw_caret_x, 0.0, layout.size.y.max(0.0)))
        } else {
            (raw_caret_x, 0.0, layout.size.y.max(0.0))
        };
        if let Ok((mut scroll_offset, mut text_node)) = visuals.p0().get_mut(field.text_entity) {
            if focused && !disabled {
                let max_offset_x = (layout.size.x - input_content_width).max(0.0);
                if raw_caret_x < scroll_offset.x {
                    scroll_offset.x = raw_caret_x.max(0.0);
                } else if raw_caret_x > scroll_offset.x + input_content_width {
                    scroll_offset.x = (raw_caret_x - input_content_width).max(0.0);
                }
                scroll_offset.x = scroll_offset.x.min(max_offset_x);
                if field.is_multiline() {
                    let max_offset_y = (layout.size.y - input_content_height).max(0.0);
                    if caret_line_top < scroll_offset.y {
                        scroll_offset.y = caret_line_top.max(0.0);
                    } else if caret_line_top + caret_line_height
                        > scroll_offset.y + input_content_height
                    {
                        scroll_offset.y =
                            (caret_line_top + caret_line_height - input_content_height).max(0.0);
                    }
                    scroll_offset.y = scroll_offset.y.min(max_offset_y);
                    text_node.top = Val::Px(-scroll_offset.y);
                }
                text_node.left = Val::Px(-scroll_offset.x);
            }
            scroll_x = scroll_offset.x;
            scroll_y = scroll_offset.y;
        }
        let text_origin = Vec2::new(
            input_inset.min_inset.x - scroll_x,
            input_inset.min_inset.y - scroll_y,
        );
        let caret_x = text_origin.x + caret_line_x;
        let caret_top = text_origin.y + caret_line_top;
        let caret_center_y = caret_top + caret_line_height * 0.5;

        if field.selection_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p1().get_mut(field.selection_entity) {
                if let Some(range) = field.edit_state.selection_range() {
                    let rects = if field.is_multiline() {
                        selection_rects_for_range(layout, &display_text.text, range.start, range.end)
                    } else {
                        let start_x =
                            text_x_for_byte(layout, &display_text.text, range.start);
                        let end_x = text_x_for_byte(layout, &display_text.text, range.end);
                        vec![(start_x, 0.0, (end_x - start_x).max(0.0), caret_line_height)]
                    };
                    node.left = Val::Px(text_origin.x);
                    node.top = Val::Px(text_origin.y);
                    node.width = Val::Px(0.0);
                    node.height = Val::Px(0.0);
                    *visibility = if rects.is_empty() {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    };
                    let mut assigned = 0usize;
                    for (parent, mut segment_node, mut segment_visibility) in &mut visuals.p3() {
                        if parent.parent() != field.selection_entity {
                            continue;
                        }
                        if let Some((left, top, width, height)) = rects.get(assigned).copied() {
                            segment_node.left = Val::Px(left);
                            segment_node.top = Val::Px(top);
                            segment_node.width = Val::Px(width);
                            segment_node.height = Val::Px(height.max(0.0));
                            *segment_visibility = if width > 0.0 {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                            assigned += 1;
                        } else {
                            *segment_visibility = Visibility::Hidden;
                        }
                    }
                } else {
                    *visibility = Visibility::Hidden;
                    for (parent, _, mut segment_visibility) in &mut visuals.p3() {
                        if parent.parent() == field.selection_entity {
                            *segment_visibility = Visibility::Hidden;
                        }
                    }
                }
            }
        }

        if field.caret_entity != Entity::PLACEHOLDER {
            if let Ok((mut node, mut visibility)) = visuals.p2().get_mut(field.caret_entity) {
                node.left = Val::Px((caret_x - caret_width * 0.5).max(0.0));
                node.top = Val::Px(caret_top);
                node.height = Val::Px(caret_line_height.max(0.0));
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
