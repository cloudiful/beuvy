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

## Version Compatibility

| beuvy | bevy | MSRV |
| --- | --- | --- |
| 0.1 | 0.18 | 1.85 |

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

Low-level runtime usage lives in
[`beuvy-runtime/README.md`](./beuvy-runtime/README.md).

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

### Current HTML-Like Surface

The current goal is "native-feeling basics first", not full DOM parity. The
surface below is the practical authoring set that works today.

| Area | Supported |
| --- | --- |
| Containers | `div`, `section`, `header`, `footer`, `main`, `nav`, `aside`, `article`, `form`, `fieldset` |
| Text | `span`, `p`, `legend`, `small`, `strong`, `em`, `h1`-`h6` |
| Controls | `button`, `input`, `textarea`, `select`, `option`, `label` |
| Input types | `text`, `password`, `number`, `range`, `checkbox`, `radio` |
| Content tags | `img`, `a`, `hr`, `ul`, `ol`, `li` |
| Vue-style bindings | `v-if`, `v-else-if`, `v-else`, `v-show`, `v-for`, `v-model`, `:class`, `:style`, `:value`, `:checked`, `:disabled`, `:ref`, `:src`, `:alt`, `:href` |
| Events | `@click`, `@input`, `@change`, `@scroll`, `@wheel` |

Current notable limits:

- No `table` family yet
- No `video`, `audio`, or `canvas`
- `<img>` supports Bevy asset paths, not remote URLs
- `<a>` emits runtime/declarative events; it does not auto-navigate
- `br` and full inline-flow layout are intentionally deferred

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

## Runtime Examples

Low-level runnable examples live under `beuvy-runtime/examples`:

| Example | Purpose |
| --- | --- |
| `basic_controls` | minimal text, button, and input primitives |
| `button_states` | hover / press / disabled button visuals |
| `control_events` | runtime event flow for controls |
| `form_core` | checkbox, radio, password, and label-oriented form basics |
| `content_core` | image, link, separator, and content-page primitives |
| `utility_classes` | utility-class parsing and styling behavior |

## License

Licensed under the Apache License, Version 2.0.
