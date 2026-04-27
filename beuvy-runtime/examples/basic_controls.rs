use beuvy_runtime::input::InputType;
use beuvy_runtime::input::InputValueChangedMessage;
use beuvy_runtime::text::set_plain_text;
use beuvy_runtime::{AddButton, AddInput, AddSelect, AddSelectOption, AddText, UiKitPlugin};
use bevy::prelude::*;
use bevy::text::TextLayout;

#[derive(Component)]
struct SliderValueText;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime basic controls".to_string(),
                resolution: (1280, 860).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiKitPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, sync_slider_value_label)
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
                column_gap: Val::Px(20.0),
                overflow: Overflow::visible(),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(245, 247, 250)),
        ))
        .with_children(|parent| {
            spawn_column(parent, |parent| {
                spawn_panel(parent, "Text", |parent| {
                    spawn_text(parent, "Display Title", 22.0, Color::srgb_u8(15, 23, 42));
                    spawn_text(
                        parent,
                        "Body copy rendered through the runtime text builder.",
                        15.0,
                        Color::srgb_u8(30, 41, 59),
                    );
                    spawn_text(
                        parent,
                        "Secondary hint text for dense tool UIs.",
                        13.0,
                        Color::srgb_u8(100, 116, 139),
                    );
                });

                spawn_panel(parent, "Text Inputs", |parent| {
                    parent.spawn(AddInput {
                        name: "pilot_name".to_string(),
                        placeholder: "Pilot name".to_string(),
                        size_chars: Some(24),
                        ..default()
                    });
                    parent.spawn(AddInput {
                        name: "callsign".to_string(),
                        value: "ALPHA-7".to_string(),
                        size_chars: Some(16),
                        ..default()
                    });
                    parent.spawn(AddInput {
                        name: "disabled_text".to_string(),
                        value: "Locked field".to_string(),
                        size_chars: Some(18),
                        disabled: true,
                        ..default()
                    });
                });

                spawn_panel(parent, "Numeric Inputs", |parent| {
                    spawn_row(parent, |parent| {
                        parent.spawn(AddInput {
                            name: "count".to_string(),
                            input_type: InputType::Number,
                            value: "12".to_string(),
                            min: Some(0.0),
                            max: Some(64.0),
                            step: Some(1.0),
                            size_chars: Some(8),
                            ..default()
                        });
                        parent.spawn(AddInput {
                            name: "threshold".to_string(),
                            input_type: InputType::Number,
                            value: "0.75".to_string(),
                            min: Some(0.0),
                            max: Some(1.0),
                            step: Some(0.05),
                            size_chars: Some(8),
                            ..default()
                        });
                    });
                    parent.spawn((
                        SliderValueText,
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                        TextLayout::default(),
                        AddText {
                            text: "Volume: 45".to_string(),
                            size: 14.0,
                            color: Color::srgb_u8(71, 85, 105),
                            ..default()
                        },
                    ));
                    parent.spawn(AddInput {
                        name: "volume".to_string(),
                        input_type: InputType::Range,
                        value: "45".to_string(),
                        min: Some(0.0),
                        max: Some(100.0),
                        step: Some(5.0),
                        ..default()
                    });
                });
            });

            spawn_column(parent, |parent| {
                spawn_panel(parent, "Selects", |parent| {
                    parent.spawn(AddSelect {
                        name: "difficulty".to_string(),
                        value: "normal".to_string(),
                        options: vec![
                            option("difficulty_easy", "easy", "Easy"),
                            option("difficulty_normal", "normal", "Normal"),
                            option("difficulty_hard", "hard", "Hard"),
                        ],
                        ..default()
                    });
                    parent.spawn(AddSelect {
                        name: "region".to_string(),
                        value: "us-east".to_string(),
                        options: vec![
                            option("region_use1", "us-east", "US East"),
                            option("region_euw1", "eu-west", "EU West"),
                            AddSelectOption {
                                name: "region_apac".to_string(),
                                value: "apac".to_string(),
                                text: "APAC (disabled)".to_string(),
                                localized_text: None,
                                localized_text_format: None,
                                disabled: true,
                            },
                        ],
                        ..default()
                    });
                });

                spawn_panel(parent, "Buttons", |parent| {
                    spawn_row(parent, |parent| {
                        parent.spawn(AddButton {
                            name: "primary".to_string(),
                            text: "Primary Action".to_string(),
                            class: Some("button-root w-[180px]".to_string()),
                            ..default()
                        });
                        parent.spawn(AddButton {
                            name: "secondary".to_string(),
                            text: "Secondary".to_string(),
                            class: Some("button-root min-h-[36px] w-[140px] px-[10px]".to_string()),
                            ..default()
                        });
                    });
                    spawn_row(parent, |parent| {
                        parent.spawn(AddButton {
                            name: "disabled".to_string(),
                            text: "Disabled".to_string(),
                            class: Some("button-root w-[180px]".to_string()),
                            disabled: true,
                            ..default()
                        });
                        parent.spawn(AddButton {
                            name: "compact".to_string(),
                            text: "Compact".to_string(),
                            class: Some(
                                "button-root min-h-[30px] w-[120px] px-[8px] py-[4px]".to_string(),
                            ),
                            ..default()
                        });
                    });
                });
            });
        });
}

fn sync_slider_value_label(
    mut commands: Commands,
    mut events: MessageReader<InputValueChangedMessage>,
    labels: Query<Entity, With<SliderValueText>>,
) {
    let Some(label) = labels.iter().next() else {
        return;
    };
    for event in events.read() {
        if event.name == "volume" {
            set_plain_text(&mut commands, label, format!("Volume: {}", event.value));
        }
    }
}

fn spawn_column(
    parent: &mut ChildSpawnerCommands,
    children: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            width: Val::Px(600.0),
            row_gap: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::visible(),
            ..default()
        })
        .with_children(children);
}

fn spawn_panel(
    parent: &mut ChildSpawnerCommands,
    title: &str,
    children: impl FnOnce(&mut ChildSpawnerCommands),
) {
    let mut panel = parent.spawn(Node {
        width: Val::Percent(100.0),
        min_height: Val::Px(200.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(12.0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::visible(),
        border_radius: BorderRadius::all(Val::Px(12.0)),
        ..default()
    });
    panel.insert(BorderColor::all(Color::srgb_u8(209, 213, 219)));
    panel.insert(BackgroundColor(Color::WHITE));
    panel.with_children(|parent| {
        spawn_text(parent, title, 18.0, Color::srgb_u8(15, 23, 42));
        children(parent);
    });
}

fn spawn_row(parent: &mut ChildSpawnerCommands, children: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn(Node {
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(10.0),
            flex_wrap: FlexWrap::Wrap,
            overflow: Overflow::visible(),
            ..default()
        })
        .with_children(children);
}

fn spawn_text(parent: &mut ChildSpawnerCommands, text: &str, size: f32, color: Color) {
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

fn option(name: &str, value: &str, text: &str) -> AddSelectOption {
    AddSelectOption {
        name: name.to_string(),
        value: value.to_string(),
        text: text.to_string(),
        localized_text: None,
        localized_text_format: None,
        disabled: false,
    }
}
