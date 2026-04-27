# beuvy

`beuvy` is a declarative UI layer for Bevy. It parses a compact UI asset format
into typed runtime data and materializes that data with reusable controls from
`beuvy-runtime`.

Use this crate when an application needs data-driven UI shells, bindings,
localized text, conditional nodes, repeated nodes, refs, and style patches.
Use `beuvy-runtime` directly when only low-level controls and utility classes
are needed.

## Crates

- `beuvy`: declarative UI parser, asset loader, and Bevy runtime integration.
- `beuvy-runtime`: reusable Bevy UI controls, utility-class styling, and
  state-driven visual styles.

## Install

```toml
[dependencies]
beuvy = "0.1.0"
```

The runtime crate can also be used directly:

```toml
[dependencies]
beuvy-runtime = "0.1.0"
```

## Quick Start

```rust
use bevy::prelude::*;
use beuvy::DeclarativeUiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DeclarativeUiPlugin)
        .run();
}
```

## Runtime Controls

For direct control construction, use `beuvy-runtime`:

```rust
use bevy::prelude::*;
use beuvy_runtime::{AddButton, AddText, UiKitPlugin};

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

## License

Licensed under the Apache License, Version 2.0.
