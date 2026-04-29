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
                padding: UiRect::axes(Val::Px(40.0), Val::Px(32.0)),
                row_gap: Val::Px(20.0),
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
            spawn_labeled_toggle_row(
                parent,
                "Checkbox input",
                AddInput {
                    name: "enable_audio".to_string(),
                    input_type: InputType::Checkbox,
                    checked: true,
                    ..default()
                },
            );
            spawn_labeled_toggle_row(
                parent,
                "Radio input (easy)",
                AddInput {
                    name: "difficulty".to_string(),
                    input_type: InputType::Radio,
                    value: "easy".to_string(),
                    input_value: Some("easy".to_string()),
                    checked: true,
                    ..default()
                },
            );
            spawn_labeled_toggle_row(
                parent,
                "Radio input (hard)",
                AddInput {
                    name: "difficulty".to_string(),
                    input_type: InputType::Radio,
                    value: "hard".to_string(),
                    input_value: Some("hard".to_string()),
                    ..default()
                },
            );
            parent
                .spawn((
                    Node {
                        width: Val::Px(440.0),
                        row_gap: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                ))
                .with_children(|field| {
                    spawn_field_label(field, "Password input");
                    field.spawn(AddInput {
                        name: "secret".to_string(),
                        input_type: InputType::Password,
                        value: "hunter2".to_string(),
                        placeholder: "Password".to_string(),
                        size_chars: Some(20),
                        ..default()
                    });
                });
        });
}

fn spawn_labeled_toggle_row(parent: &mut ChildSpawnerCommands, label: &str, input: AddInput) {
    parent
        .spawn((
            Node {
                column_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|row| {
            row.spawn(input);
            spawn_inline_label(row, label);
        });
}

fn spawn_field_label(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Node::default(),
        TextLayout::default(),
        AddText {
            text: text.to_string(),
            size: 13.0,
            color: Color::srgb_u8(75, 85, 99),
            ..default()
        },
    ));
}

fn spawn_inline_label(parent: &mut ChildSpawnerCommands, text: &str) {
    parent.spawn((
        Node::default(),
        TextLayout::default(),
        AddText {
            text: text.to_string(),
            size: 15.0,
            color: Color::srgb_u8(31, 41, 55),
            ..default()
        },
    ));
}
