# beuvy

`beuvy` is a Vue-flavored declarative UI layer for Bevy. It lets you describe
UI with a compact SFC-like template format, parse it into typed assets, and
materialize it with the reusable controls from `beuvy-runtime`.

Use `beuvy` when you want:

- `<template>`-driven UI authoring
- Vue-style directives like `v-if`, `v-show`, `v-for`, `v-model`, and `@click`
- runtime bindings, refs, localized text, and style patches on top of Bevy UI

## Crates

- `beuvy`: the high-level declarative authoring crate.
- `beuvy-runtime`: reusable Bevy UI controls, utility-class styling, and
  state-driven visual styles used underneath `beuvy`.

## Install

```toml
[dependencies]
beuvy = { version = "0.1.0", default-features = false, features = ["vue"] }
```

If you want the current default bundle:

```toml
[dependencies]
beuvy = "0.1.0"
```

If you only need the low-level runtime controls:

```toml
[dependencies]
beuvy = { version = "0.1.0", default-features = false, features = ["runtime"] }
```

Or depend on the runtime crate directly:

```toml
[dependencies]
beuvy-runtime = "0.1.0"
```

## Vue-Style Authoring

`beuvy` accepts a compact SFC-like format with top-level `<template>`, optional
`<script>`, and optional `<style>` blocks. The current surface focuses on
pragmatic UI authoring rather than full Vue compatibility.

Supported patterns include:

- `v-if`, `v-else-if`, `v-else`, `v-show`
- `v-for`
- `v-model`
- `:class`, `:style`, `:value`, `:disabled`, `:ref`
- `@click`, `@input`, `@change`, `@scroll`, `@wheel`
- `{{ mustache }}` text interpolation and `$t(...)` localized text

Example:

```xml
<template>
  <section class="flex flex-col gap-3 p-4">
    <h1>{{ title }}</h1>

    <input v-model="draft.name" placeholder="Pilot name" />

    <button @click="selectedTab = 'inventory'">Inventory</button>

    <p v-if="draft.name">Hello {{ draft.name }}</p>

    <select v-model="selectedItem">
      <option v-for="entry in items" :value="entry.value">
        {{ entry.text }}
      </option>
    </select>
  </section>
</template>
```

## Bevy Integration

Register the declarative runtime plugin:

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

Programmatic parsing is also available:

```rust
use beuvy::parse_declarative_ui_asset;

let asset = parse_declarative_ui_asset(
    r#"<template><button @click="tab = 'inventory'">Open</button></template>"#,
)?;
```

## Feature Layout

- `runtime`: re-exports `beuvy-runtime` for direct low-level use.
- `declarative`: parser, asset loader, shell materialization, bindings.
- `vue`: preferred high-level feature alias for declarative authoring.

## Runtime Layer

`beuvy-runtime` still exists as the lower-level layer. `beuvy` re-exports the
main runtime surface, so direct control construction can stay on the `beuvy`
import path:

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

Use `beuvy-runtime` directly only when you do not want the declarative layer.

## License

Licensed under the Apache License, Version 2.0.
