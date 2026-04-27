use super::context::DeclarativeUiBuildContext;
use super::state::DeclarativeTextBinding;
use super::style::parse_hex_color;
use crate::ast::*;
use beuvy_runtime::text::{AddText, LocalizedTextFormat};
use bevy::prelude::default;
use bevy_localization::TextKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedTextContent {
    Plain(String),
    Localized(TextKey),
    LocalizedFormat(LocalizedTextFormat),
}

pub(crate) fn build_add_text(
    content: &DeclarativeUiTextContent,
    style: &DeclarativeTextStyle,
    context: &DeclarativeUiBuildContext,
) -> (AddText, Option<DeclarativeTextBinding>) {
    let color = style
        .color
        .as_deref()
        .and_then(parse_hex_color)
        .unwrap_or_else(crate::style::text_primary_color);
    let resolved = resolve_text_content(content, |path| context.string(path));
    (
        add_text_from_resolved(resolved, style.size, color),
        content_has_dynamic_bindings(content).then(|| DeclarativeTextBinding(content.clone())),
    )
}

fn localized_text_format(
    key: TextKey,
    args: &[DeclarativeLocalizedTextArg],
) -> LocalizedTextFormat {
    let mut format = LocalizedTextFormat::new(key);
    for arg in args {
        format = match arg.name.as_str() {
            "index" => format.with_arg("index", &arg.value),
            _ => format,
        };
    }
    format
}

pub(crate) fn button_text_content(
    content: &DeclarativeUiTextContent,
    context: &DeclarativeUiBuildContext,
) -> (String, Option<TextKey>, Option<LocalizedTextFormat>) {
    text_parts_from_resolved(resolve_text_content(content, |path| context.string(path)))
}

pub(crate) fn default_option_value(
    content: &DeclarativeUiTextContent,
    resolved_text: &str,
    context: &DeclarativeUiBuildContext,
) -> String {
    match content {
        DeclarativeUiTextContent::Bind { path } => context
            .string(path)
            .unwrap_or_else(|| resolved_text.to_string()),
        DeclarativeUiTextContent::I18n { key, .. } => match key {
            DeclarativeTextKeySource::Static(value) => value.clone(),
            DeclarativeTextKeySource::Binding(path) => {
                context.string(path).unwrap_or_else(|| path.clone())
            }
        },
        _ => resolved_text.to_string(),
    }
}

pub(crate) fn content_has_dynamic_bindings(content: &DeclarativeUiTextContent) -> bool {
    match content {
        DeclarativeUiTextContent::Static { .. } => false,
        DeclarativeUiTextContent::Bind { .. } => true,
        DeclarativeUiTextContent::Segments { segments } => segments
            .iter()
            .any(|segment| matches!(segment, DeclarativeUiTextSegment::Bind { .. })),
        DeclarativeUiTextContent::I18n { key, .. } => {
            matches!(key, DeclarativeTextKeySource::Binding(_))
        }
    }
}

pub(crate) fn resolve_text_content(
    content: &DeclarativeUiTextContent,
    mut resolve_string: impl FnMut(&str) -> Option<String>,
) -> ResolvedTextContent {
    match content {
        DeclarativeUiTextContent::Static { text } => ResolvedTextContent::Plain(text.clone()),
        DeclarativeUiTextContent::Bind { path } => {
            ResolvedTextContent::Plain(resolve_string(path).unwrap_or_default())
        }
        DeclarativeUiTextContent::Segments { segments } => {
            let mut text = String::new();
            for segment in segments {
                match segment {
                    DeclarativeUiTextSegment::Static { text: value } => text.push_str(value),
                    DeclarativeUiTextSegment::Bind { path } => {
                        text.push_str(&resolve_string(path).unwrap_or_default())
                    }
                }
            }
            ResolvedTextContent::Plain(text)
        }
        DeclarativeUiTextContent::I18n {
            key,
            localized_text_args,
        } => {
            let resolved_key = match key {
                DeclarativeTextKeySource::Static(value) => TextKey::from_id(value),
                DeclarativeTextKeySource::Binding(path) => {
                    resolve_string(path).as_deref().and_then(TextKey::from_id)
                }
            };
            match (resolved_key, localized_text_args.is_empty()) {
                (Some(key), true) => ResolvedTextContent::Localized(key),
                (Some(key), false) => ResolvedTextContent::LocalizedFormat(localized_text_format(
                    key,
                    localized_text_args,
                )),
                (None, _) => ResolvedTextContent::Plain(String::new()),
            }
        }
    }
}

fn add_text_from_resolved(
    resolved: ResolvedTextContent,
    size: f32,
    color: bevy::prelude::Color,
) -> AddText {
    match resolved {
        ResolvedTextContent::Plain(text) => AddText {
            text,
            localized_text: None,
            localized_text_format: None,
            size,
            color,
            ..default()
        },
        ResolvedTextContent::Localized(key) => AddText {
            text: String::new(),
            localized_text: Some(key),
            localized_text_format: None,
            size,
            color,
            ..default()
        },
        ResolvedTextContent::LocalizedFormat(localized_text_format) => AddText {
            text: String::new(),
            localized_text: None,
            localized_text_format: Some(localized_text_format),
            size,
            color,
            ..default()
        },
    }
}

fn text_parts_from_resolved(
    resolved: ResolvedTextContent,
) -> (String, Option<TextKey>, Option<LocalizedTextFormat>) {
    match resolved {
        ResolvedTextContent::Plain(text) => (text, None, None),
        ResolvedTextContent::Localized(key) => (String::new(), Some(key), None),
        ResolvedTextContent::LocalizedFormat(format) => (String::new(), None, Some(format)),
    }
}
