# beuvy-runtime

`beuvy-runtime` is a compact UI kit for Bevy.

It provides:

- A core plugin for low-level UI controls
- Reusable text, button, input, and form-item builders
- Tailwind-like utility class parsing

The crate is intended as a reusable Bevy UI foundation. It does not include the
higher-level declarative `.vue` DSL used by GPMO.

## Version Compatibility

| beuvy-runtime | bevy | MSRV |
| --- | --- | --- |
| 0.1 | 0.18 | 1.85 |

## Install

```toml
[dependencies]
bevy = "0.18.1"
beuvy-runtime = "0.1.0"
bevy-localization = { package = "cloudiful-bevy-localization", version = "0.1.2" }
```

If you only need the low-level runtime controls through the top-level crate:

```toml
[dependencies]
beuvy = { version = "0.1.0", default-features = false, features = ["runtime"] }
```

## Quick Start

```rust
use bevy::prelude::*;
use beuvy_runtime::{AddButton, AddInput, AddText, UiKitPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
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
                padding: UiRect::all(Val::Px(16.0)),
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
                name: "player_name".to_string(),
                placeholder: "Pilot name".to_string(),
                ..default()
            });
            parent.spawn(AddButton {
                name: "confirm".to_string(),
                text: "Launch".to_string(),
                ..default()
            });
        });
}
```

`beuvy` re-exports the main runtime surface, so direct control construction can
also stay on the `beuvy` import path:

```rust
use bevy::prelude::*;
use beuvy::{AddButton, AddText, UiKitPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiKitPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(AddText {
                text: "beuvy".to_string(),
                ..default()
            });
            parent.spawn(AddButton {
                name: "confirm".to_string(),
                text: "Confirm".to_string(),
                ..default()
            });
        });
}
```

Use `beuvy-runtime` directly when you do not want the declarative layer.

## Utility Classes

Use `parse_utility_classes` to convert a class string into a reusable style
patch:

```rust
use beuvy_runtime::parse_utility_classes;

let patch = parse_utility_classes("flex flex-col gap-[12px] px-[10px]").unwrap();
println!("{patch:?}");
```

`beuvy-runtime` does not expose a runtime theme system anymore. Higher-level
token semantics and DSL-driven styling belong in `beuvy` or the application
layer.

## Fonts

`beuvy-runtime` does not require a crate-managed font asset. By default it
leaves the `TextFont.font` handle unset and relies on Bevy's built-in default
font.

If your app wants a custom font, insert a `FontResource` yourself:

```rust
use bevy::prelude::*;
use beuvy_runtime::text::FontResource;

fn setup_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(FontResource::from_handle(
        asset_server.load("fonts/YourFont.ttf"),
    ));
}
```

## Examples

- `cargo run -p beuvy-runtime --example basic_controls`
- `cargo run -p beuvy-runtime --example button_states`
- `cargo run -p beuvy-runtime --example control_events`
- `cargo run -p beuvy-runtime --example utility_classes`
