use crate::button::{
    ActiveButton, Button, ButtonInner, ButtonLabel, DisabledButton, sync_button_active_state,
};
use crate::interaction_style::UiDisabled;
use crate::select::model::{
    Select, SelectChevron, SelectOptionButton, SelectOptionIndicator, SelectPanel, SelectTrigger,
    selected_option, sync_select_label, trigger_label_entity,
};
use bevy::prelude::*;
use bevy::ui::Val::{Percent, Px};
use bevy_localization::Localization;
use std::collections::HashSet;

pub(crate) fn sync_select_semantics(
    mut commands: Commands,
    localization: Option<Res<Localization>>,
    mut select_queries: ParamSet<(
        Query<Entity, With<Select>>,
        Query<Entity, Changed<Select>>,
        Query<&mut Select>,
    )>,
    trigger_labels: Query<&ButtonLabel>,
    disabled_buttons: Query<Has<DisabledButton>, With<Button>>,
    mut panel_nodes: Query<&mut Node, With<SelectPanel>>,
) {
    let localization_changed = localization
        .as_ref()
        .is_some_and(|value| value.is_changed());
    let mut dirty = if localization_changed {
        select_queries.p0().iter().collect::<Vec<_>>()
    } else {
        select_queries.p1().iter().collect::<Vec<_>>()
    };
    dirty.sort_unstable();
    dirty.dedup();

    for entity in dirty {
        let mut select_states = select_queries.p2();
        let Ok(mut select) = select_states.get_mut(entity) else {
            continue;
        };

        sync_trigger_disabled(
            &mut commands,
            &disabled_buttons,
            &mut panel_nodes,
            &mut select,
        );

        if let Some(label_entity) = trigger_label_entity(&trigger_labels, &select) {
            sync_select_label(
                &mut commands,
                localization.as_deref(),
                label_entity,
                selected_option(&select),
                &select.value,
            );
        }

        for option in &select.options {
            sync_button_active_state(&mut commands, option.entity, option.value == select.value);
            sync_option_disabled(
                &mut commands,
                &disabled_buttons,
                option.entity,
                option.disabled,
            );
            if let Ok(label) = trigger_labels.get(option.entity) {
                sync_select_label(
                    &mut commands,
                    localization.as_deref(),
                    label.entity,
                    Some(option),
                    &option.value,
                );
            }
        }
    }
}

fn sync_trigger_disabled(
    commands: &mut Commands,
    disabled_buttons: &Query<Has<DisabledButton>, With<Button>>,
    panel_nodes: &mut Query<&mut Node, With<SelectPanel>>,
    select: &mut Select,
) {
    sync_option_disabled(commands, disabled_buttons, select.trigger, select.disabled);
    if select.disabled && select.open {
        select.open = false;
        if let Ok(mut panel) = panel_nodes.get_mut(select.panel) {
            panel.display = Display::None;
        }
    }
}

fn sync_option_disabled(
    commands: &mut Commands,
    disabled_buttons: &Query<Has<DisabledButton>, With<Button>>,
    entity: Entity,
    disabled: bool,
) {
    let Ok(has_disabled) = disabled_buttons.get(entity) else {
        return;
    };
    let Ok(mut entity_commands) = commands.get_entity(entity) else {
        return;
    };

    if disabled {
        if !has_disabled {
            entity_commands.try_insert((DisabledButton, UiDisabled));
        }
    } else if has_disabled {
        entity_commands
            .try_remove::<DisabledButton>()
            .try_remove::<UiDisabled>();
    }
}

pub(crate) fn sync_select_button_layouts(
    mut buttons: Query<
        (
            &mut Node,
            Option<&SelectTrigger>,
            Option<&SelectOptionButton>,
        ),
        Added<Button>,
    >,
) {
    for (mut node, trigger, option) in &mut buttons {
        if trigger.is_none() && option.is_none() {
            continue;
        }

        node.min_width = Px(0.0);
        node.max_width = Percent(100.0);
        node.flex_grow = 1.0;
        node.justify_content = JustifyContent::FlexStart;
        node.align_items = AlignItems::Center;
        node.position_type = PositionType::Relative;
        node.overflow = Overflow::clip_x();
        node.column_gap = Px(10.0);
        node.padding.right = Px(12.0);
    }
}

pub(crate) fn sync_select_option_indicators(
    added_option_buttons: Query<Entity, Added<SelectOptionButton>>,
    activated_option_buttons: Query<Entity, (With<SelectOptionButton>, Added<ActiveButton>)>,
    mut removed_active_buttons: RemovedComponents<ActiveButton>,
    option_buttons: Query<(&Children, Has<ActiveButton>), With<SelectOptionButton>>,
    mut indicators: Query<&mut Visibility, With<SelectOptionIndicator>>,
) {
    let mut dirty_buttons = HashSet::new();
    for entity in &added_option_buttons {
        dirty_buttons.insert(entity);
    }
    for entity in &activated_option_buttons {
        dirty_buttons.insert(entity);
    }
    for entity in removed_active_buttons.read() {
        if option_buttons.contains(entity) {
            dirty_buttons.insert(entity);
        }
    }
    if dirty_buttons.is_empty() {
        return;
    }

    for entity in dirty_buttons {
        let Ok((children, is_active)) = option_buttons.get(entity) else {
            continue;
        };

        for child in children {
            let Ok(mut visibility) = indicators.get_mut(*child) else {
                continue;
            };
            *visibility = if is_active {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub(crate) fn sync_select_accessory_layout(
    select_buttons: Query<
        (
            &ButtonLabel,
            Option<&SelectTrigger>,
            Option<&SelectOptionButton>,
        ),
        With<Button>,
    >,
    mut nodes: ParamSet<(
        Query<&mut Node, With<ButtonInner>>,
        Query<&mut Node, Or<(With<SelectChevron>, With<SelectOptionIndicator>)>>,
    )>,
) {
    for (label, trigger, option) in &select_buttons {
        if trigger.is_none() && option.is_none() {
            continue;
        }

        if let Ok(mut label_node) = nodes.p0().get_mut(label.entity) {
            label_node.flex_grow = 1.0;
            label_node.flex_shrink = 1.0;
            label_node.min_width = Px(0.0);
        }
    }

    for mut node in &mut nodes.p1() {
        node.position_type = PositionType::Relative;
        node.left = Val::Auto;
        node.right = Val::Auto;
        node.top = Val::Auto;
        node.bottom = Val::Auto;
        node.margin.left = Val::Auto;
    }
}
