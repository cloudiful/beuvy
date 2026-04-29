use beuvy_runtime::{AddImage, AddLink, AddText, UiKitPlugin};
use bevy::prelude::*;
use bevy::text::TextLayout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime content core".to_string(),
                resolution: (980, 640).into(),
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
                padding: UiRect::axes(Val::Px(40.0), Val::Px(28.0)),
                row_gap: Val::Px(14.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(248, 250, 252)),
        ))
        .with_children(|parent| {
            text(parent, "Content page essentials", 30.0, Color::srgb_u8(15, 23, 42));
            text(
                parent,
                "Heading, paragraph, helper text, image, link, rule, and list primitives.",
                15.0,
                Color::srgb_u8(71, 85, 105),
            );
            parent.spawn((
                Node {
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    ..default()
                },
                AddImage {
                    src: "branding/icon.png".to_string(),
                    alt: "Project icon".to_string(),
                    ..default()
                },
            ));
            parent.spawn((
                Node {
                    width: Val::Px(220.0),
                    ..default()
                },
                AddLink {
                    name: "docs".to_string(),
                    href: "/docs/getting-started".to_string(),
                    text: "Open getting started docs".to_string(),
                    ..default()
                },
            ));
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgb_u8(203, 213, 225)),
            ));
            parent
                .spawn((
                    Node {
                        row_gap: Val::Px(8.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|list| {
                    text(list, "• Semantic content tags", 15.0, Color::srgb_u8(30, 41, 59));
                    text(
                        list,
                        "• Fieldset and legend defaults",
                        15.0,
                        Color::srgb_u8(30, 41, 59),
                    );
                    text(
                        list,
                        "• Small / strong / emphasis styling hooks",
                        15.0,
                        Color::srgb_u8(30, 41, 59),
                    );
                });
        });
}

fn text(parent: &mut ChildSpawnerCommands, value: &str, size: f32, color: Color) {
    parent.spawn((
        Node::default(),
        TextLayout::default(),
        AddText {
            text: value.to_string(),
            size,
            color,
            ..default()
        },
    ));
}
