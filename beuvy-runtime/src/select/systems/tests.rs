use super::*;
use crate::select::model::{Select, SelectPanel, SelectTrigger};
use crate::select::systems::placement::SELECT_PANEL_GAP;
use bevy::prelude::*;
use bevy::ui::Val::{Auto, Percent, Px};

fn computed_node(width: f32, height: f32) -> ComputedNode {
    ComputedNode {
        size: Vec2::new(width, height),
        inverse_scale_factor: 1.0,
        ..default()
    }
}

fn spawn_open_select(app: &mut App, trigger_y: f32) -> Entity {
    let root = app
        .world_mut()
        .spawn((
            Node {
                overflow: Overflow::scroll_y(),
                ..default()
            },
            computed_node(300.0, 300.0),
            UiGlobalTransform::default(),
        ))
        .id();
    let select = app.world_mut().spawn_empty().id();
    let trigger = app
        .world_mut()
        .spawn((
            SelectTrigger { select },
            computed_node(160.0, 32.0),
            UiGlobalTransform::from_translation(Vec2::new(0.0, trigger_y)),
        ))
        .id();
    let panel = app
        .world_mut()
        .spawn((
            SelectPanel,
            Node {
                display: Display::Flex,
                max_height: Px(360.0),
                ..default()
            },
            computed_node(160.0, 120.0),
        ))
        .id();

    app.world_mut().entity_mut(root).add_child(select);
    app.world_mut()
        .entity_mut(select)
        .add_children(&[trigger, panel]);
    app.world_mut().entity_mut(select).insert(Select {
        name: "select".to_string(),
        value: "a".to_string(),
        options: Vec::new(),
        panel,
        trigger,
        chevron_glyph: Entity::PLACEHOLDER,
        open: true,
        disabled: false,
    });

    panel
}

#[test]
fn select_panel_flips_above_when_bottom_space_is_tight() {
    let mut app = App::new();
    app.add_systems(Update, sync_select_panel_placement);
    let panel = spawn_open_select(&mut app, 130.0);

    app.update();

    let node = app.world().entity(panel).get::<Node>().expect("panel node");
    assert_eq!(node.top, Auto);
    assert_eq!(node.bottom, Percent(100.0));
    assert_eq!(node.margin.bottom, Px(SELECT_PANEL_GAP));
    assert_eq!(node.max_height, Px(258.0));
}

#[test]
fn select_panel_opens_below_when_bottom_space_is_available() {
    let mut app = App::new();
    app.add_systems(Update, sync_select_panel_placement);
    let panel = spawn_open_select(&mut app, 0.0);

    app.update();

    let node = app.world().entity(panel).get::<Node>().expect("panel node");
    assert_eq!(node.top, Percent(100.0));
    assert_eq!(node.bottom, Auto);
    assert_eq!(node.margin.top, Px(SELECT_PANEL_GAP));
    assert_eq!(node.max_height, Px(128.0));
}
