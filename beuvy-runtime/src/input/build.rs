use super::edit::TextEditState;
use super::range::{spawn_range_fill, spawn_range_thumb, spawn_range_track};
use super::text::{
    apply_check_input_shape, default_check_indicator_node, default_check_input_node, default_input_node,
    default_textarea_node, input_text_bundle, input_text_marker, input_text_node,
};
use super::{
    AddInput, DisabledInput, InputCaret, InputCheckRoot, InputClickState, InputField,
    InputIndicator, InputScrollOffset, InputSelection, InputType, InputViewport, RangeState,
    UndoStack,
};
use crate::build_pending::UiBuildPending;
use crate::focus::{UiFocusable, hidden_outline};
use crate::interaction_style::UiDisabled;
use crate::style::{
    apply_utility_patch, checkbox_border_color, checkbox_indicator_color,
    input_selection_color, resolve_classes_with_fallback, root_visual_styles_from_patch,
};
use crate::text::AddText;
use bevy::picking::Pickable;
use bevy::prelude::*;

const DEFAULT_INPUT_CLASS: &str = "input-root";
const DEFAULT_TEXTAREA_CLASS: &str = "textarea-root";
const DEFAULT_RANGE_CLASS: &str = "input-range-root";
const DEFAULT_CHECKBOX_CLASS: &str = "checkbox-root";
const DEFAULT_RADIO_CLASS: &str = "radio-root";

pub(super) fn add_input(mut commands: Commands, query: Query<(Entity, &AddInput)>) {
    for (entity, add_input) in query {
        let add_input = add_input.clone();
        let default_root_class = if add_input.input_type == InputType::Range {
            DEFAULT_RANGE_CLASS
        } else if add_input.input_type == InputType::Checkbox {
            DEFAULT_CHECKBOX_CLASS
        } else if add_input.input_type == InputType::Radio {
            DEFAULT_RADIO_CLASS
        } else if add_input.input_type == InputType::Textarea {
            DEFAULT_TEXTAREA_CLASS
        } else {
            DEFAULT_INPUT_CLASS
        };
        let root_patch = resolve_classes_with_fallback(
            default_root_class,
            add_input.class.as_deref(),
            "input root",
        );
        let root_styles = root_visual_styles_from_patch(&root_patch);
        let mut root_node = if add_input.input_type == InputType::Range {
            Node {
                min_width: Val::Px(120.0),
                flex_grow: 1.0,
                padding: UiRect::ZERO,
                border: UiRect::ZERO,
                ..default()
            }
        } else if matches!(add_input.input_type, InputType::Checkbox | InputType::Radio) {
            default_check_input_node()
        } else if add_input.input_type == InputType::Textarea {
            default_textarea_node(add_input.size_chars, add_input.rows)
        } else {
            default_input_node(add_input.size_chars)
        };
        if matches!(add_input.input_type, InputType::Checkbox | InputType::Radio) {
            apply_check_input_shape(&mut root_node, add_input.input_type);
        }
        apply_utility_patch(&mut root_node, &root_patch);

        commands
            .entity(entity)
            .queue_silenced(move |mut entity_commands: EntityWorldMut| {
                let input_entity = entity_commands.id();
                let normalized_value = if add_input.input_type == InputType::Range {
                    super::value::normalize_numeric_value(
                        &add_input.value,
                        add_input.min,
                        add_input.max,
                        add_input.step,
                    )
                } else {
                    add_input.value.clone()
                };

                entity_commands.insert((
                    Name::new(add_input.name.clone()),
                    root_node,
                    Visibility::Visible,
                    BackgroundColor(if matches!(add_input.input_type, InputType::Checkbox | InputType::Radio) {
                        Color::WHITE
                    } else {
                        Color::NONE
                    }),
                    BorderColor::all(if matches!(add_input.input_type, InputType::Checkbox | InputType::Radio) {
                        checkbox_border_color()
                    } else {
                        Color::NONE
                    }),
                    InputField {
                        name: add_input.name.clone(),
                        input_type: add_input.input_type,
                        checked: add_input.checked,
                        input_value: add_input.input_value.clone(),
                        placeholder: add_input.placeholder.clone(),
                        viewport_entity: None,
                        text_entity: None,
                        selection_entity: None,
                        caret_entity: None,
                        edit_state: TextEditState::with_text(normalized_value),
                        min: add_input.min,
                        max: add_input.max,
                        step: add_input.step,
                        caret_blink_resume_at: 0.0,
                        preferred_caret_x: None,
                        undo_stack: UndoStack::default(),
                    },
                    InputClickState::default(),
                ));
                if add_input.input_type != InputType::Range {
                    entity_commands.insert(hidden_outline());
                }
                if let Some(styles) = root_styles.clone() {
                    entity_commands.insert(styles);
                }

                if add_input.input_type == InputType::Range {
                    entity_commands.world_scope(|world| {
                        let track = spawn_range_track(world, input_entity);
                        let fill = spawn_range_fill(world);
                        let thumb = spawn_range_thumb(world);
                        world.entity_mut(track).add_children(&[fill, thumb]);
                        world.entity_mut(input_entity).add_child(track);
                        world.entity_mut(input_entity).insert(RangeState {
                            track,
                            fill,
                            thumb,
                            drag_start_value: 0.0,
                        });
                    });
                } else if matches!(add_input.input_type, InputType::Checkbox | InputType::Radio) {
                    entity_commands.world_scope(|world| {
                        let indicator = world
                            .spawn((
                                InputIndicator,
                                InputCheckRoot,
                                Pickable::IGNORE,
                                Visibility::Hidden,
                                default_check_indicator_node(add_input.input_type),
                                BackgroundColor(checkbox_indicator_color()),
                                BorderColor::all(checkbox_indicator_color()),
                            ))
                            .id();
                        world.entity_mut(input_entity).add_child(indicator);
                        let mut input = world
                            .get_mut::<InputField>(input_entity)
                            .expect("input just inserted");
                        input.viewport_entity = Some(indicator);
                    });
                } else {
                    let text_patch = resolve_classes_with_fallback(
                        "input-text",
                        add_input.text_class.as_deref(),
                        "input text",
                    );
                    let mut text_node = input_text_node();
                    apply_utility_patch(&mut text_node, &text_patch);
                    let text_add_input = AddInput {
                        value: add_input.value.clone(),
                        ..add_input.clone()
                    };
                    entity_commands.world_scope(|world| {
                        let selection_color = input_selection_color();
                        let viewport_entity = world
                            .spawn((
                                InputViewport,
                                Pickable::IGNORE,
                                Node {
                                    position_type: PositionType::Relative,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    min_width: Val::Px(0.0),
                                    overflow: Overflow::clip(),
                                    ..default()
                                },
                            ))
                            .id();
                        let selection_entity = world
                            .spawn((
                                InputSelection,
                                Pickable::IGNORE,
                                Visibility::Hidden,
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(0.0),
                                    height: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(selection_color),
                            ))
                            .id();
                        world.entity_mut(selection_entity).with_children(|_parent| {
                            // Segments are spawned dynamically in sync_input_edit_visuals
                        });
                        let text_entity = world
                            .spawn((
                                input_text_marker(),
                                Pickable::IGNORE,
                                AddText {
                                    size: text_patch
                                        .text_size
                                        .unwrap_or_else(crate::style::font_size_control),
                                    ..input_text_bundle(&text_add_input)
                                },
                                InputScrollOffset::default(),
                                text_node,
                            ))
                            .id();
                        let caret_entity = world
                            .spawn((
                                InputCaret,
                                Pickable::IGNORE,
                                Visibility::Hidden,
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Px(crate::style::input_caret_width()),
                                    height: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(crate::style::input_caret_color()),
                            ))
                            .id();
                        world.entity_mut(viewport_entity).add_children(&[
                            selection_entity,
                            text_entity,
                            caret_entity,
                        ]);
                        world.entity_mut(input_entity).add_child(viewport_entity);
                        let mut input = world
                            .get_mut::<InputField>(input_entity)
                            .expect("input just inserted");
                        input.viewport_entity = Some(viewport_entity);
                        input.text_entity = Some(text_entity);
                        input.selection_entity = Some(selection_entity);
                        input.caret_entity = Some(caret_entity);
                    });
                }

                if add_input.disabled {
                    entity_commands.insert((DisabledInput, UiDisabled));
                } else {
                    if add_input.input_type != InputType::Range {
                        entity_commands.insert(UiFocusable);
                    }
                }

                entity_commands.observe(super::state::input_click);
                entity_commands.observe(super::state::input_drag_start);
                entity_commands.observe(super::state::input_drag);
                entity_commands.observe(super::state::input_drag_end);

                entity_commands
                    .remove::<AddInput>()
                    .remove::<UiBuildPending>();
            });
    }
}
