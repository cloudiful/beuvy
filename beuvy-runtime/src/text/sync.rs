use super::{LocalizedArg, LocalizedText, LocalizedTextFormat};
use bevy::prelude::*;
use bevy_localization::Localization;

pub(super) fn sync_localized_text_on_binding_change(
    mut query: Query<(&mut Text, &LocalizedText), Changed<LocalizedText>>,
    localization: Option<Res<Localization>>,
) {
    let Some(localization) = localization else {
        return;
    };
    for (mut text, localized_text) in &mut query {
        *text = Text::new(localization.text(localized_text.key));
    }
}

pub(super) fn sync_localized_text_on_locale_change(
    mut query: Query<(&mut Text, &LocalizedText)>,
    localization: Option<Res<Localization>>,
) {
    let Some(localization) = localization else {
        return;
    };
    if !localization.is_changed() {
        return;
    }

    for (mut text, localized_text) in &mut query {
        *text = Text::new(localization.text(localized_text.key));
    }
}

pub(super) fn sync_localized_text_format_on_binding_change(
    mut query: Query<(&mut Text, &LocalizedTextFormat), Changed<LocalizedTextFormat>>,
    localization: Option<Res<Localization>>,
) {
    let Some(localization) = localization else {
        return;
    };
    for (mut text, localized_text) in &mut query {
        *text = Text::new(format_localized_text(localized_text, &localization));
    }
}

pub(super) fn sync_localized_text_format_on_locale_change(
    mut query: Query<(&mut Text, &LocalizedTextFormat)>,
    localization: Option<Res<Localization>>,
) {
    let Some(localization) = localization else {
        return;
    };
    if !localization.is_changed() {
        return;
    }

    for (mut text, localized_text) in &mut query {
        *text = Text::new(format_localized_text(localized_text, &localization));
    }
}

fn format_localized_text(
    localized_text: &LocalizedTextFormat,
    localization: &Localization,
) -> String {
    localization.format_text(
        localized_text.key,
        localized_text
            .args
            .iter()
            .map(|arg: &LocalizedArg| (arg.name, arg.value.as_str())),
    )
}
