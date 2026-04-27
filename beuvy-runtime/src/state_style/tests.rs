use super::transition::color_nearly_equal;
use super::*;
use crate::theme_config::resolve_theme_color_value;
use crate::utility::{UtilityTransitionProperty, UtilityVal, UtilityVisualStylePatch};
use bevy::time::TimeUpdateStrategy;
use bevy::ui::Val::Px;
use std::time::Duration;

fn sample_styles() -> UiStateVisualStyles {
    UiStateVisualStyles {
        base: UtilityVisualStylePatch {
            background_color: Some("var(--color-button-bg)".to_string()),
            text_color: Some("var(--color-button-text)".to_string()),
            border_color: Some("var(--color-panel-subtle-border)".to_string()),
            ..default()
        },
        hover: UtilityVisualStylePatch {
            background_color: Some("var(--color-button-bg-hover)".to_string()),
            ..default()
        },
        active: UtilityVisualStylePatch {
            background_color: Some("var(--color-button-bg-active)".to_string()),
            ..default()
        },
        focus: UtilityVisualStylePatch {
            outline_width: Some(UtilityVal::Px(2.0)),
            outline_color: Some("var(--color-primary)".to_string()),
            ..default()
        },
        disabled: UtilityVisualStylePatch::default(),
    }
}

fn sample_transition_styles() -> UiStateVisualStyles {
    let mut styles = sample_styles();
    styles.base.transition_property = Some(UtilityTransitionProperty::Colors);
    styles.base.transition_duration_ms = Some(150.0);
    styles
}

#[test]
fn active_overrides_hover_and_focus_keeps_outline() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(UiStateStylePlugin);

    let entity = app
        .world_mut()
        .spawn((
            sample_styles(),
            BackgroundColor(Color::NONE),
            BorderColor::all(Color::NONE),
            TextColor(Color::NONE),
            Outline::new(Px(0.0), Px(2.0), Color::NONE),
            crate::focus::UiHovered,
            crate::focus::UiFocused,
            crate::focus::UiPressed,
        ))
        .id();

    app.update();
    app.update();

    let entity_ref = app.world().entity(entity);
    let background = entity_ref.get::<BackgroundColor>().expect("background");
    let outline = entity_ref.get::<Outline>().expect("outline");
    assert_eq!(
        background.0,
        resolve_theme_color_value("var(--color-button-bg-active)")
            .expect("active color")
            .to_bevy()
    );
    assert_eq!(outline.width, Px(2.0));
    assert_eq!(
        outline.color,
        resolve_theme_color_value("var(--color-primary)")
            .expect("primary color")
            .to_bevy()
    );
}

#[test]
fn state_source_uses_parent_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(UiStateStylePlugin);

    let parent = app.world_mut().spawn(crate::focus::UiHovered).id();
    let child = app
        .world_mut()
        .spawn((
            sample_styles(),
            UiStateStyleSource(parent),
            TextColor(Color::NONE),
        ))
        .id();

    app.update();
    app.update();

    let text = app
        .world()
        .entity(child)
        .get::<TextColor>()
        .expect("text color");
    assert_eq!(
        text.0,
        resolve_theme_color_value("var(--color-button-text)")
            .expect("text color")
            .to_bevy()
    );
}

#[test]
fn color_transition_reaches_target() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(UiStateStylePlugin);

    let entity = app
        .world_mut()
        .spawn((
            sample_transition_styles(),
            BackgroundColor(
                resolve_theme_color_value("var(--color-button-bg)")
                    .expect("base color")
                    .to_bevy(),
            ),
        ))
        .id();

    app.update();
    app.world_mut()
        .entity_mut(entity)
        .insert(crate::focus::UiHovered);
    app.update();
    app.update();

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        75,
    )));
    app.update();
    let mid = app
        .world()
        .entity(entity)
        .get::<BackgroundColor>()
        .expect("background")
        .0;
    let target = resolve_theme_color_value("var(--color-button-bg-hover)")
        .expect("hover color")
        .to_bevy();
    assert!(!color_nearly_equal(mid, target));

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
        100,
    )));
    app.update();
    let final_color = app
        .world()
        .entity(entity)
        .get::<BackgroundColor>()
        .expect("background")
        .0;
    assert!(color_nearly_equal(final_color, target));
}
