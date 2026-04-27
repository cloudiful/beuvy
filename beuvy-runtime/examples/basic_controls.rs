use beuvy_runtime::{AddButton, AddInput, AddText, UiKitPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime basic controls".to_string(),
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
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Stretch,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn(AddText {
                text: "beuvy-runtime".to_string(),
                ..default()
            });
            parent.spawn(AddInput {
                name: "pilot_name".to_string(),
                placeholder: "Pilot name".to_string(),
                size_chars: Some(24),
                ..default()
            });
            parent.spawn(AddButton {
                name: "launch".to_string(),
                text: "Launch".to_string(),
                ..default()
            });
        });
}
