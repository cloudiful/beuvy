use beuvy_runtime::{AddButton, AddText, UiKitPlugin};
use bevy::prelude::*;
use bevy::text::TextLayout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime button states".to_string(),
                resolution: (900, 560).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiKitPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(248, 250, 252)),
        ))
        .with_children(|parent| {
            spawn_text(parent, "Button state showcase", 22.0, Color::srgb_u8(15, 23, 42));
            spawn_text(
                parent,
                "Hover, press, tab-focus, and compare disabled or custom-sized variants.",
                14.0,
                Color::srgb_u8(71, 85, 105),
            );

            spawn_group(parent, "Default", |parent| {
                parent.spawn(AddButton {
                    name: "default_primary".to_string(),
                    text: "Primary Action".to_string(),
                    class: Some("button-root w-[180px]".to_string()),
                    ..default()
                });
                parent.spawn(AddButton {
                    name: "default_secondary".to_string(),
                    text: "Secondary Action".to_string(),
                    class: Some("button-root w-[180px]".to_string()),
                    ..default()
                });
            });

            spawn_group(parent, "Sizing", |parent| {
                parent.spawn(AddButton {
                    name: "compact".to_string(),
                    text: "Compact".to_string(),
                    class: Some("button-root min-h-[30px] px-[8px] py-[4px]".to_string()),
                    ..default()
                });
                parent.spawn(AddButton {
                    name: "wide".to_string(),
                    text: "Wide Button".to_string(),
                    class: Some("button-root min-h-[48px] w-[220px]".to_string()),
                    ..default()
                });
            });

            spawn_group(parent, "Disabled", |parent| {
                parent.spawn(AddButton {
                    name: "disabled_default".to_string(),
                    text: "Disabled".to_string(),
                    disabled: true,
                    ..default()
                });
                parent.spawn(AddButton {
                    name: "disabled_wide".to_string(),
                    text: "Disabled Wide".to_string(),
                    disabled: true,
                    class: Some("button-root min-h-[48px] w-[220px]".to_string()),
                    ..default()
                });
            });
        });
}

fn spawn_group(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    children: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent.spawn(AddText {
        text: title.to_string(),
        size: 16.0,
        color: Color::srgb_u8(30, 41, 59),
        ..default()
    });
    let mut group = parent.spawn(Node {
        padding: UiRect::all(Val::Px(16.0)),
        column_gap: Val::Px(12.0),
        row_gap: Val::Px(12.0),
        flex_wrap: FlexWrap::Wrap,
        overflow: Overflow::visible(),
        border_radius: BorderRadius::all(Val::Px(12.0)),
        ..default()
    });
    group.insert(BorderColor::all(Color::srgb_u8(203, 213, 225)));
    group.insert(BackgroundColor(Color::WHITE));
    group.with_children(children);
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    text: &str,
    size: f32,
    color: Color,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        TextLayout::default(),
        AddText {
            text: text.to_string(),
            size,
            color,
            ..default()
        },
    ));
}
