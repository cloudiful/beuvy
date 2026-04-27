use super::range::{spawn_range_fill, spawn_range_thumb, spawn_range_track};
use super::text::{default_input_node, input_text_bundle, input_text_marker, input_text_node};
use super::value::normalize_numeric_value;
use super::{AddInput, DisabledInput, InputField, InputType};
use crate::build_pending::UiBuildPending;
use crate::focus::{UiFocusable, hidden_outline};
use crate::interaction_style::UiDisabled;
use crate::style::{apply_utility_patch, resolve_classes_with_fallback};
use crate::text::AddText;
use bevy::picking::Pickable;
use bevy::prelude::*;

const DEFAULT_INPUT_CLASS: &str = "input-root";

pub(super) fn add_input(mut commands: Commands, query: Query<(Entity, &AddInput)>) {
    for (entity, add_input) in query {
        let add_input = add_input.clone();
        let root_patch = resolve_classes_with_fallback(
            DEFAULT_INPUT_CLASS,
            add_input.class.as_deref(),
            "input root",
        );
        let mut root_node = if add_input.input_type == InputType::Range {
            Node {
                min_width: Val::Px(120.0),
                flex_grow: 1.0,
                padding: UiRect::ZERO,
                border: UiRect::ZERO,
                ..default()
            }
        } else {
            default_input_node(add_input.size_chars)
        };
        apply_utility_patch(&mut root_node, &root_patch);

        commands
            .entity(entity)
            .queue_silenced(move |mut entity_commands: EntityWorldMut| {
                let input_entity = entity_commands.id();
                let normalized_value =
                    if matches!(add_input.input_type, InputType::Number | InputType::Range) {
                        normalize_numeric_value(
                            &add_input.value,
                            add_input.min,
                            add_input.max,
                            add_input.step,
                        )
                    } else {
                        add_input.value.clone()
                    };

                let mut text_entity = Entity::PLACEHOLDER;

                entity_commands.insert((
                    Name::new(add_input.name.clone()),
                    root_node,
                    Visibility::Visible,
                    BackgroundColor(Color::NONE),
                    hidden_outline(),
                    InputField {
                        name: add_input.name.clone(),
                        input_type: add_input.input_type,
                        value: normalized_value,
                        placeholder: add_input.placeholder.clone(),
                        text_entity,
                        preedit: None,
                        min: add_input.min,
                        max: add_input.max,
                        step: add_input.step,
                        range_track: None,
                        range_fill: None,
                        range_thumb: None,
                        drag_start_value: 0.0,
                    },
                ));

                if add_input.input_type == InputType::Range {
                    entity_commands.world_scope(|world| {
                        let track = spawn_range_track(world, input_entity);
                        let fill = spawn_range_fill(world);
                        let thumb = spawn_range_thumb(world);
                        world.entity_mut(track).add_children(&[fill, thumb]);
                        world.entity_mut(input_entity).add_child(track);
                        let mut input = world
                            .get_mut::<InputField>(input_entity)
                            .expect("input just inserted");
                        input.range_track = Some(track);
                        input.range_fill = Some(fill);
                        input.range_thumb = Some(thumb);
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
                        value: entity_commands
                            .get::<InputField>()
                            .map(|field| field.value.clone())
                            .unwrap_or_default(),
                        ..add_input.clone()
                    };
                    entity_commands.world_scope(|world| {
                        text_entity = world
                            .spawn((
                                input_text_marker(),
                                Pickable::IGNORE,
                                AddText {
                                    size: text_patch
                                        .text_size
                                        .unwrap_or_else(crate::style::font_size_control),
                                    ..input_text_bundle(&text_add_input)
                                },
                                text_node,
                            ))
                            .id();
                        world.entity_mut(input_entity).add_child(text_entity);
                        world
                            .get_mut::<InputField>(input_entity)
                            .expect("input just inserted")
                            .text_entity = text_entity;
                    });
                }

                if add_input.disabled {
                    entity_commands.insert((DisabledInput, UiDisabled));
                } else {
                    entity_commands.insert(UiFocusable);
                }

                entity_commands.observe(super::state::input_click);

                entity_commands
                    .remove::<AddInput>()
                    .remove::<UiBuildPending>();
            });
    }
}
