use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::Val::Px;

pub struct BackdropPlugin;

impl Plugin for BackdropPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component, Debug, Clone)]
pub struct Backdrop {
    pub base_layer: Entity,
    pub top_vignette: Entity,
    pub bottom_vignette: Entity,
    pub left_vignette: Entity,
    pub right_vignette: Entity,
    pub base_alpha: u8,
    pub vignette_alpha: u8,
}

pub fn spawn_backdrop(commands: &mut Commands, z_index: i32) -> Entity {
    let root = commands
        .spawn((
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                left: Px(0.0),
                right: Px(0.0),
                top: Px(0.0),
                bottom: Px(0.0),
                ..default()
            },
            Visibility::Visible,
            BackgroundColor(Color::srgba_u8(0, 0, 0, 0)),
            GlobalZIndex(z_index),
        ))
        .id();

    let base_layer = commands
        .spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: Px(0.0),
                right: Px(0.0),
                top: Px(0.0),
                bottom: Px(0.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 12, 4, 0)),
        ))
        .id();
    let top_vignette = commands
        .spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: Px(0.0),
                right: Px(0.0),
                top: Px(0.0),
                height: Val::Percent(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 20, 6, 0)),
        ))
        .id();
    let bottom_vignette = commands
        .spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: Px(0.0),
                right: Px(0.0),
                bottom: Px(0.0),
                height: Val::Percent(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 20, 6, 0)),
        ))
        .id();
    let left_vignette = commands
        .spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                left: Px(0.0),
                top: Px(0.0),
                bottom: Px(0.0),
                width: Val::Percent(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 12, 4, 0)),
        ))
        .id();
    let right_vignette = commands
        .spawn((
            Pickable::IGNORE,
            Node {
                position_type: PositionType::Absolute,
                right: Px(0.0),
                top: Px(0.0),
                bottom: Px(0.0),
                width: Val::Percent(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(0, 12, 4, 0)),
        ))
        .id();

    commands.entity(root).add_children(&[
        base_layer,
        top_vignette,
        bottom_vignette,
        left_vignette,
        right_vignette,
    ]);
    commands.entity(root).insert(Backdrop {
        base_layer,
        top_vignette,
        bottom_vignette,
        left_vignette,
        right_vignette,
        base_alpha: 68,
        vignette_alpha: 20,
    });

    root
}
