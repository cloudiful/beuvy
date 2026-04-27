# beuvy-runtime

`beuvy-runtime` is a compact UI kit for Bevy.

It provides:

- A core plugin for low-level UI controls
- Reusable text, button, input, and form-item builders
- Tailwind-like utility class parsing

The crate is intended as a reusable Bevy UI foundation. It does not include the
higher-level declarative `.vue` DSL used by GPMO.

## Install

```toml
[dependencies]
bevy = "0.18.1"
beuvy-runtime = "0.1.0"
bevy-localization = { package = "cloudiful-bevy-localization", version = "0.1.2" }
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

## Examples

- `cargo run -p beuvy-runtime --example basic_controls`
- `cargo run -p beuvy-runtime --example utility_classes`
