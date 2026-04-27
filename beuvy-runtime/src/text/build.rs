use super::{AddText, FontResource, LocalizedText, LocalizedTextFormat};
use crate::build_pending::UiBuildPending;
use crate::style::font_asset_path;
use bevy::prelude::*;
use bevy_localization::Localization;

pub(super) fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let primary_font = asset_server.load(font_asset_path());
    commands.insert_resource(FontResource { primary_font });
}

pub(super) fn add_text(
    mut commands: Commands,
    query: Query<(Entity, &AddText)>,
    font_resource: Res<FontResource>,
    localization: Option<Res<Localization>>,
) {
    for (entity, add_text) in query {
        let Ok(mut entity_commands) = commands.get_entity(entity) else {
            continue;
        };

        debug_assert!(
            !(add_text.localized_text.is_some() && add_text.localized_text_format.is_some()),
            "text entity cannot bind both LocalizedText and LocalizedTextFormat"
        );

        let initial_text = match (
            localization.as_deref(),
            add_text.localized_text,
            add_text.localized_text_format.clone(),
        ) {
            (Some(localization), Some(key), _) => localization.text(key).to_string(),
            (Some(localization), _, Some(localized_text_format)) => localization.format_text(
                localized_text_format.key,
                localized_text_format
                    .args
                    .iter()
                    .map(|arg| (arg.name, arg.value.as_str())),
            ),
            _ => add_text.text.clone(),
        };

        entity_commands.try_insert((
            Text::new(initial_text),
            TextFont {
                font: font_resource.primary_font.clone(),
                font_size: add_text.size,
                ..default()
            },
            add_text.line_height,
            TextColor(add_text.color),
        ));

        if let Some(localized_text_format) = add_text.localized_text_format.clone() {
            entity_commands.try_remove::<LocalizedText>();
            entity_commands.try_insert(localized_text_format);
        } else if let Some(key) = add_text.localized_text {
            entity_commands.try_remove::<LocalizedTextFormat>();
            entity_commands.try_insert(LocalizedText { key });
        } else {
            entity_commands.try_remove::<LocalizedTextFormat>();
            entity_commands.try_remove::<LocalizedText>();
        }

        entity_commands.try_remove::<AddText>();
        entity_commands.try_remove::<UiBuildPending>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;

    #[test]
    fn add_text_ignores_entities_despawned_before_apply() {
        let mut app = App::new();
        app.insert_resource(FontResource {
            primary_font: Handle::<Font>::default(),
        })
        .register_required_components::<AddText, UiBuildPending>();

        let entity = app.world_mut().spawn(AddText::default()).id();

        let mut system_state: SystemState<(
            Commands,
            Query<(Entity, &AddText)>,
            Res<FontResource>,
            Option<Res<Localization>>,
        )> = SystemState::new(app.world_mut());
        let (commands, query, font_resource, localization) = system_state.get_mut(app.world_mut());
        add_text(commands, query, font_resource, localization);

        app.world_mut().despawn(entity);
        system_state.apply(app.world_mut());
    }
}
