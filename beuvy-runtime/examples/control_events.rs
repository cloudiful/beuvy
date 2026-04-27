use beuvy_runtime::button::ButtonClickMessage;
use beuvy_runtime::input::{InputType, InputValueChangedMessage};
use beuvy_runtime::text::set_plain_text;
use beuvy_runtime::{
    AddButton, AddInput, AddSelect, AddSelectOption, AddText, SelectValueChangedMessage,
    UiKitPlugin,
};
use bevy::prelude::*;
use bevy::text::TextLayout;

#[derive(Component)]
struct EventLogText;

#[derive(Resource, Default)]
struct EventLog {
    entries: Vec<String>,
}

fn main() {
    App::new()
        .init_resource::<EventLog>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "beuvy-runtime control events".to_string(),
                resolution: (1180, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(UiKitPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                record_button_events,
                record_input_events,
                record_select_events,
            ),
        )
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
            BackgroundColor(Color::srgb_u8(248, 250, 252)),
        ))
        .with_children(|parent| {
            let mut left = parent.spawn(Node {
                width: Val::Px(420.0),
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::visible(),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            });
            left.insert(BorderColor::all(Color::srgb_u8(203, 213, 225)));
            left.insert(BackgroundColor(Color::WHITE));
            left.with_children(|parent| {
                spawn_text(
                    parent,
                    "Interact with these controls",
                    18.0,
                    Color::srgb_u8(15, 23, 42),
                );
                parent.spawn(AddInput {
                    name: "display_name".to_string(),
                    placeholder: "Type a display name".to_string(),
                    size_chars: Some(24),
                    ..default()
                });
                parent.spawn(AddInput {
                    name: "zoom".to_string(),
                    input_type: InputType::Range,
                    value: "35".to_string(),
                    min: Some(0.0),
                    max: Some(100.0),
                    step: Some(5.0),
                    ..default()
                });
                parent.spawn(AddSelect {
                    name: "theme".to_string(),
                    value: "light".to_string(),
                    options: vec![
                        option("theme_light", "light", "Light"),
                        option("theme_dark", "dark", "Dark"),
                        option("theme_hc", "high-contrast", "High Contrast"),
                    ],
                    ..default()
                });
                parent
                    .spawn(Node {
                        column_gap: Val::Px(10.0),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(AddButton {
                            name: "save".to_string(),
                            text: "Save".to_string(),
                            class: Some("button-root w-[120px]".to_string()),
                            ..default()
                        });
                        parent.spawn(AddButton {
                            name: "reset".to_string(),
                            text: "Reset".to_string(),
                            class: Some("button-root w-[120px]".to_string()),
                            ..default()
                        });
                    });
            });

            let mut right = parent.spawn(Node {
                flex_grow: 1.0,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::visible(),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            });
            right.insert(BorderColor::all(Color::srgb_u8(203, 213, 225)));
            right.insert(BackgroundColor(Color::WHITE));
            right.with_children(|parent| {
                spawn_text(parent, "Event log", 18.0, Color::srgb_u8(15, 23, 42));
                parent.spawn((
                    EventLogText,
                    Node {
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    TextLayout::default(),
                    AddText {
                        text: "No events yet.\nClick, type, drag, or select.".to_string(),
                        size: 14.0,
                        color: Color::srgb_u8(71, 85, 105),
                        ..default()
                    },
                ));
            });
        });
}

fn record_button_events(
    mut commands: Commands,
    mut events: MessageReader<ButtonClickMessage>,
    mut log: ResMut<EventLog>,
    labels: Query<Entity, With<EventLogText>>,
) {
    let mut changed = false;
    for event in events.read() {
        log.entries.push(format!(
            "button:{} on {:?}",
            event.button.name, event.entity
        ));
        changed = true;
    }
    if changed {
        sync_event_log_text(&mut commands, &log, &labels);
    }
}

fn record_input_events(
    mut commands: Commands,
    mut events: MessageReader<InputValueChangedMessage>,
    mut log: ResMut<EventLog>,
    labels: Query<Entity, With<EventLogText>>,
) {
    let mut changed = false;
    for event in events.read() {
        log.entries
            .push(format!("input:{} = {}", event.name, event.value));
        changed = true;
    }
    if changed {
        sync_event_log_text(&mut commands, &log, &labels);
    }
}

fn record_select_events(
    mut commands: Commands,
    mut events: MessageReader<SelectValueChangedMessage>,
    mut log: ResMut<EventLog>,
    labels: Query<Entity, With<EventLogText>>,
) {
    let mut changed = false;
    for event in events.read() {
        log.entries
            .push(format!("select:{} = {}", event.name, event.value));
        changed = true;
    }
    if changed {
        sync_event_log_text(&mut commands, &log, &labels);
    }
}

fn sync_event_log_text(
    commands: &mut Commands,
    log: &EventLog,
    labels: &Query<Entity, With<EventLogText>>,
) {
    let Some(entity) = labels.iter().next() else {
        return;
    };
    let text = log
        .entries
        .iter()
        .rev()
        .take(12)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    set_plain_text(commands, entity, text);
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
