# beuvy

`beuvy` is the facade crate for the Beuvy UI stack. By default it re-exports
`beuvy-runtime` and also enables the declarative authoring layer that parses a
compact UI asset format into typed runtime data.

Use this crate when an application needs either:

- the low-level runtime controls and utility styling from `beuvy-runtime`
- the higher-level declarative shell, bindings, localized text, refs, and style
  patches layered on top

## Crates

- `beuvy`: facade crate with feature-gated declarative authoring support.
- `beuvy-runtime`: reusable Bevy UI controls, utility-class styling, and
  state-driven visual styles.

## Install

```toml
[dependencies]
beuvy = "0.1.0"
```

Thin facade usage, runtime only:

```toml
[dependencies]
beuvy = { version = "0.1.0", default-features = false, features = ["runtime"] }
```

Declarative or Vue-flavored authoring layer:

```toml
[dependencies]
beuvy = { version = "0.1.0", default-features = false, features = ["vue"] }
```

The runtime crate can still be used directly:

```toml
[dependencies]
beuvy-runtime = "0.1.0"
```

## Feature Layout

- `runtime`: re-exports `beuvy-runtime` as the stable low-level surface.
- `declarative`: parser, asset loader, shell materialization, bindings.
- `vue`: current alias to `declarative`, reserved for higher-level authoring
  APIs as they split out over time.

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

For direct control construction, `beuvy` now works as a thin facade too:

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

If you only need the low-level layer, depending on `beuvy-runtime` directly is
still fine.

## License

Licensed under the Apache License, Version 2.0.
