use beuvy_runtime::input::InputType;
use beuvy_runtime::{AddInput, AddText, UiKitPlugin};
use bevy::prelude::*;
use bevy::text::TextLayout;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime form core".to_string(),
                resolution: (960, 540).into(),
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
                ..default()
            },
            BackgroundColor(Color::srgb_u8(248, 250, 252)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Node::default(),
                TextLayout::default(),
                AddText {
                    text: "Form core controls".to_string(),
                    size: 20.0,
                    color: Color::srgb_u8(15, 23, 42),
                    ..default()
                },
            ));
            parent.spawn(AddInput {
                name: "enable_audio".to_string(),
                input_type: InputType::Checkbox,
                checked: true,
                ..default()
            });
            parent.spawn(AddInput {
                name: "difficulty".to_string(),
                input_type: InputType::Radio,
                value: "easy".to_string(),
                input_value: Some("easy".to_string()),
                checked: true,
                ..default()
            });
            parent.spawn(AddInput {
                name: "difficulty".to_string(),
                input_type: InputType::Radio,
                value: "hard".to_string(),
                input_value: Some("hard".to_string()),
                ..default()
            });
            parent.spawn(AddInput {
                name: "secret".to_string(),
                input_type: InputType::Password,
                value: "hunter2".to_string(),
                placeholder: "Password".to_string(),
                size_chars: Some(20),
                ..default()
            });
        });
}
