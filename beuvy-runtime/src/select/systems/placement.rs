use crate::select::model::{Select, SelectPanel, SelectTrigger};
use bevy::prelude::*;
use bevy::ui::Val::{Auto, Px};
use bevy::window::PrimaryWindow;

pub(super) const SELECT_PANEL_GAP: f32 = 6.0;
const SELECT_PANEL_DEFAULT_MAX_HEIGHT: f32 = 360.0;
const SELECT_PANEL_MIN_HEIGHT: f32 = 48.0;

pub(crate) fn sync_select_panel_placement(
    selects: Query<&Select>,
    trigger_nodes: Query<(&ComputedNode, &UiGlobalTransform), With<SelectTrigger>>,
    mut panel_nodes: Query<(&mut Node, &ComputedNode), With<SelectPanel>>,
    parents: Query<&ChildOf>,
    ancestors: Query<(&Node, &ComputedNode, &UiGlobalTransform), Without<SelectPanel>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    let window_rect = primary_window.iter().next().map(|window| Rect {
        min: Vec2::ZERO,
        max: window.size(),
    });

    for select in &selects {
        if !select.open {
            continue;
        }

        let Ok((trigger_computed, trigger_transform)) = trigger_nodes.get(select.trigger) else {
            continue;
        };
        let Ok((mut panel_node, panel_computed)) = panel_nodes.get_mut(select.panel) else {
            continue;
        };

        let trigger_rect = node_global_rect(trigger_computed, trigger_transform);
        let clip_rect = nearest_vertical_clip_rect(select.panel, &parents, &ancestors)
            .or(window_rect)
            .unwrap_or(trigger_rect);
        let below_space = (clip_rect.max.y - trigger_rect.max.y - SELECT_PANEL_GAP).max(0.0);
        let above_space = (trigger_rect.min.y - clip_rect.min.y - SELECT_PANEL_GAP).max(0.0);
        let open_up = below_space < SELECT_PANEL_MIN_HEIGHT && above_space > below_space;
        let available_space = if open_up { above_space } else { below_space };
        let max_height = available_space
            .max(SELECT_PANEL_MIN_HEIGHT)
            .min(SELECT_PANEL_DEFAULT_MAX_HEIGHT);
        let panel_height = panel_computed.size().y * panel_computed.inverse_scale_factor();
        let panel_width = panel_computed.size().x * panel_computed.inverse_scale_factor();
        let trigger_width = trigger_computed.size().x * trigger_computed.inverse_scale_factor();

        let left = trigger_rect.min.x - clip_rect.min.x;
        let top = if open_up {
            trigger_rect.min.y - clip_rect.min.y - panel_height - SELECT_PANEL_GAP
        } else {
            trigger_rect.max.y - clip_rect.min.y + SELECT_PANEL_GAP
        };
        let max_left = (clip_rect.width() - panel_width).max(0.0);
        let max_top = (clip_rect.height() - panel_height).max(0.0);
        let clamped_left = left.clamp(0.0, max_left);
        let clamped_top = top.clamp(0.0, max_top);

        panel_node.left = Px(clamped_left);
        panel_node.right = Auto;
        panel_node.top = Px(clamped_top);
        panel_node.bottom = Auto;
        panel_node.min_width = Px(trigger_width);
        panel_node.margin.top = Px(0.0);
        panel_node.margin.bottom = Px(0.0);
        panel_node.max_height = Px(max_height);
        panel_node.overflow = Overflow::scroll_y();
    }
}

fn nearest_vertical_clip_rect(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    ancestors: &Query<(&Node, &ComputedNode, &UiGlobalTransform), Without<SelectPanel>>,
) -> Option<Rect> {
    while let Ok(parent) = parents.get(entity) {
        entity = parent.parent();
        let Ok((node, computed, transform)) = ancestors.get(entity) else {
            continue;
        };
        if !node.overflow.y.is_visible() {
            return Some(node_global_rect(computed, transform));
        }
    }
    None
}

fn node_global_rect(computed: &ComputedNode, transform: &UiGlobalTransform) -> Rect {
    let (_, _, center) = transform.to_scale_angle_translation();
    let physical = Rect::from_center_size(center, computed.size());
    let inverse = computed.inverse_scale_factor();
    Rect {
        min: physical.min * inverse,
        max: physical.max * inverse,
    }
}
