use super::model::{
    AddSelect, AddSelectOption, SELECT_CHEVRON_CLOSED, Select, SelectChevron, SelectChevronGlyph,
    SelectOptionButton, SelectOptionIndicator, SelectOptionState, SelectPanel, SelectTrigger,
};
use super::systems::{select_option_click, select_trigger_click};
use crate::build_pending::UiBuildPending;
use crate::button::AddButton;
use crate::scroll::MouseWheelScroll;
use crate::style::{
    apply_utility_patch, merge_classes, resolve_class_patch_or_empty,
    resolve_classes_with_fallback, root_visual_styles_from_patch, scrollbar_width,
    text_primary_color,
};
use crate::text::AddText;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::text::{Justify, LineHeight, TextLayout};

const DEFAULT_SELECT_CLASS: &str = "select-root";
const DEFAULT_SELECT_TRIGGER_CLASS: &str = "select-trigger";
const DEFAULT_SELECT_PANEL_CLASS: &str = "select-panel";
const DEFAULT_SELECT_CHEVRON_CLASS: &str = "select-chevron";
const DEFAULT_SELECT_CHEVRON_LABEL_CLASS: &str = "select-chevron-label";
const DEFAULT_SELECT_OPTION_CLASS: &str = "select-option";
const DEFAULT_SELECT_INDICATOR_CLASS: &str = "select-indicator";
const DEFAULT_SELECT_INDICATOR_LABEL_CLASS: &str = "select-indicator-label";

pub(crate) fn add_select(
    mut commands: Commands,
    query: Query<(Entity, &AddSelect)>,
) {
    for (entity, add_select) in query {
        let add_select = add_select.clone();
        let root_patch = resolve_classes_with_fallback(
            DEFAULT_SELECT_CLASS,
            add_select.class.as_deref(),
            "select root",
        );
        let mut root_node = super::default_select_node();
        apply_utility_patch(&mut root_node, &root_patch);
        commands
            .entity(entity)
            .queue_silenced(move |mut entity_commands: EntityWorldMut| {
                let select_entity = entity_commands.id();
                let trigger =
                    spawn_select_trigger(&mut entity_commands, select_entity, &add_select);
                let chevron = spawn_select_chevron(&mut entity_commands, &add_select);
                let chevron_glyph = spawn_select_chevron_glyph(&mut entity_commands, &add_select);
                let panel = spawn_select_panel(&mut entity_commands, &add_select);

                entity_commands.add_child(trigger);
                entity_commands.world_scope(|world| {
                    world.entity_mut(trigger).add_child(chevron);
                });
                entity_commands.world_scope(|world| {
                    world.entity_mut(chevron).add_child(chevron_glyph);
                });

                let options = add_select
                    .options
                    .iter()
                    .map(|option| {
                        let option_entity = spawn_select_option(
                            &mut entity_commands,
                            select_entity,
                            panel,
                            option,
                            &add_select,
                        );
                        SelectOptionState {
                            entity: option_entity,
                            value: option.value.clone(),
                            text: option.text.clone(),
                            localized_text: option.localized_text,
                            localized_text_format: option.localized_text_format.clone(),
                            disabled: option.disabled,
                        }
                    })
                    .collect::<Vec<_>>();

                entity_commands.insert((
                    root_node,
                    Select {
                        name: add_select.name.clone(),
                        value: add_select.value.clone(),
                        options,
                        panel,
                        trigger,
                        chevron_glyph,
                        open: false,
                        disabled: add_select.disabled,
                    },
                    Visibility::Visible,
                ));
                entity_commands
                    .remove::<AddSelect>()
                    .remove::<UiBuildPending>();
            });
    }
}

fn spawn_select_trigger(
    entity_commands: &mut EntityWorldMut,
    select_entity: Entity,
    add_select: &AddSelect,
) -> Entity {
    entity_commands.world_scope(|world| {
        world
            .spawn((
                SelectTrigger {
                    select: select_entity,
                },
                AddButton {
                    name: format!("{}_trigger", add_select.name),
                    text: selected_trigger_text(add_select),
                    disabled: add_select.disabled,
                    class: Some(merge_classes(
                        DEFAULT_SELECT_TRIGGER_CLASS,
                        add_select
                            .trigger_class
                            .as_deref()
                            .or(add_select.class.as_deref()),
                    )),
                    label_class: add_select.label_class.clone(),
                    ..default()
                },
            ))
            .observe(select_trigger_click)
            .id()
    })
}

fn spawn_select_chevron(entity_commands: &mut EntityWorldMut, add_select: &AddSelect) -> Entity {
    let patch = resolve_classes_with_fallback(
        DEFAULT_SELECT_CHEVRON_CLASS,
        add_select.chevron_class.as_deref(),
        "select chevron",
    );
    let styles = root_visual_styles_from_patch(&patch);
    entity_commands.world_scope(|world| {
        let mut node = Node::default();
        apply_utility_patch(&mut node, &patch);
        let mut entity = world.spawn((
            SelectChevron,
            Pickable::IGNORE,
            Visibility::Visible,
            node,
            BackgroundColor(Color::NONE),
        ));
        if let Some(styles) = styles {
            entity.insert(styles);
        }
        entity.id()
    })
}

fn spawn_select_chevron_glyph(
    entity_commands: &mut EntityWorldMut,
    _add_select: &AddSelect,
) -> Entity {
    let patch =
        resolve_class_patch_or_empty(DEFAULT_SELECT_CHEVRON_LABEL_CLASS, "select chevron label");
    entity_commands.world_scope(|world| {
        world
            .spawn((
                SelectChevronGlyph,
                Pickable::IGNORE,
                Node::default(),
                TextLayout::new_with_justify(Justify::Center),
                AddText {
                    text: SELECT_CHEVRON_CLOSED.to_string(),
                    size: patch
                        .text_size
                        .unwrap_or_else(crate::style::font_size_control),
                    line_height: LineHeight::RelativeToFont(1.0),
                    color: patch
                        .visual
                        .text_color
                        .as_deref()
                        .and_then(crate::style::resolve_color_value)
                        .unwrap_or_else(text_primary_color),
                    ..default()
                },
            ))
            .id()
    })
}

fn spawn_select_panel(entity_commands: &mut EntityWorldMut, add_select: &AddSelect) -> Entity {
    let patch = resolve_classes_with_fallback(
        DEFAULT_SELECT_PANEL_CLASS,
        add_select.panel_class.as_deref(),
        "select panel",
    );
    let styles = root_visual_styles_from_patch(&patch);
    entity_commands.world_scope(|world| {
        let mut node = Node::default();
        apply_utility_patch(&mut node, &patch);
        node.display = Display::None;
        node.position_type = PositionType::Absolute;
        node.scrollbar_width = scrollbar_width();
        let mut entity = world.spawn((
            SelectPanel,
            node,
            ScrollPosition::default(),
            MouseWheelScroll,
            BackgroundColor(Color::NONE),
            GlobalZIndex(20),
            Visibility::Visible,
        ));
        if let Some(styles) = styles {
            entity.insert(styles);
        }
        entity.id()
    })
}

fn spawn_select_option(
    root: &mut EntityWorldMut,
    select: Entity,
    panel: Entity,
    add_select_option: &AddSelectOption,
    add_select: &AddSelect,
) -> Entity {
    let option_entity = root.world_scope(|world| {
        world
            .spawn((
                SelectOptionButton {
                    select,
                    value: add_select_option.value.clone(),
                },
                AddButton {
                    name: add_select_option.name.clone(),
                    text: add_select_option.text.clone(),
                    localized_text: add_select_option.localized_text,
                    localized_text_format: add_select_option.localized_text_format.clone(),
                    disabled: add_select_option.disabled,
                    class: Some(merge_classes(
                        DEFAULT_SELECT_OPTION_CLASS,
                        add_select
                            .option_class
                            .as_deref()
                            .or(add_select.class.as_deref()),
                    )),
                    label_class: add_select.label_class.clone(),
                    ..default()
                },
            ))
            .observe(select_option_click)
            .id()
    });

    let indicator = root.world_scope(|world| {
        let patch = resolve_classes_with_fallback(
            DEFAULT_SELECT_INDICATOR_CLASS,
            add_select.indicator_class.as_deref(),
            "select indicator",
        );
        let indicator_label_patch = resolve_class_patch_or_empty(
            DEFAULT_SELECT_INDICATOR_LABEL_CLASS,
            "select indicator label",
        );
        let indicator_dot_color = indicator_label_patch
            .visual
            .text_color
            .as_deref()
            .and_then(crate::style::resolve_color_value)
            .unwrap_or_else(text_primary_color);
        let styles = root_visual_styles_from_patch(&patch);
        let mut node = Node::default();
        apply_utility_patch(&mut node, &patch);
        let mut entity = world.spawn((
            SelectOptionIndicator,
            Pickable::IGNORE,
            Visibility::Hidden,
            node,
            BackgroundColor(Color::NONE),
        ));
        if let Some(styles) = styles {
            entity.insert(styles);
        }
        let indicator = entity.id();
        let dot = world
            .spawn((
                Pickable::IGNORE,
                Node {
                    width: Val::Px(8.0),
                    height: Val::Px(8.0),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BackgroundColor(indicator_dot_color),
            ))
            .id();
        world.entity_mut(indicator).add_child(dot);
        indicator
    });

    root.world_scope(|world| {
        world.entity_mut(option_entity).add_child(indicator);
    });
    root.world_scope(|world| {
        world.entity_mut(panel).add_child(option_entity);
    });

    option_entity
}

fn selected_trigger_text(add_select: &AddSelect) -> String {
    add_select
        .options
        .iter()
        .find(|option| option.value == add_select.value)
        .map(|option| option.text.clone())
        .or_else(|| add_select.options.first().map(|option| option.text.clone()))
        .unwrap_or_else(|| add_select.value.clone())
}
