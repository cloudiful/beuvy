use crate::DeclarativeUiAssetLoadError;
use beuvy_runtime::stylesheet::{
    RuntimeStyleSource, UiStyleSheet, compose_style_sheet, default_style_sheet,
    font_size_for_tag as stylesheet_font_size_for_tag, parse_style_classes_with_sheet,
    replace_runtime_style_source,
};
use beuvy_runtime::utility::UtilityStylePatch;
use bevy::prelude::*;
use std::fs;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::SystemTime;

static STYLE_SOURCE_SNAPSHOT: OnceLock<RwLock<Arc<BeuvyStyleSource>>> = OnceLock::new();
static STYLE_SHEET_SNAPSHOT: OnceLock<RwLock<Option<BeuvyStyleSheetSnapshot>>> = OnceLock::new();

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub enum BeuvyStyleSource {
    #[default]
    BuiltIn,
    File(String),
}

impl BeuvyStyleSource {
    pub fn built_in() -> Self {
        Self::BuiltIn
    }

    pub fn file(path: impl Into<String>) -> Self {
        Self::File(path.into())
    }
}

#[derive(Debug, Clone)]
struct BeuvyStyleSheetSnapshot {
    source: BeuvyStyleSource,
    modified: Option<SystemTime>,
    sheet: Arc<UiStyleSheet>,
}

pub fn replace_style_source(source: BeuvyStyleSource) {
    let runtime_source = match &source {
        BeuvyStyleSource::BuiltIn => RuntimeStyleSource::built_in(),
        BeuvyStyleSource::File(path) => RuntimeStyleSource::file(path.clone()),
    };
    replace_runtime_style_source(runtime_source);
    let lock = STYLE_SOURCE_SNAPSHOT.get_or_init(|| RwLock::new(Arc::new(source.clone())));
    *lock
        .write()
        .expect("beuvy style source lock should not be poisoned") = Arc::new(source);
}

pub fn style_source() -> &'static BeuvyStyleSource {
    let lock =
        STYLE_SOURCE_SNAPSHOT.get_or_init(|| RwLock::new(Arc::new(BeuvyStyleSource::default())));
    let snapshot = lock
        .read()
        .expect("beuvy style source lock should not be poisoned");
    unsafe { &*Arc::as_ptr(&snapshot) }
}

pub(crate) fn parse_style_classes(
    input: &str,
) -> Result<UtilityStylePatch, DeclarativeUiAssetLoadError> {
    let sheet = current_style_sheet()?;
    parse_style_classes_with_sheet(&sheet, input).map_err(|error| style_error(error.reason))
}

pub(crate) fn font_size_for_tag(tag: &str) -> f32 {
    current_style_sheet()
        .ok()
        .map(|sheet| stylesheet_font_size_for_tag(sheet.config(), tag))
        .unwrap_or_else(|| stylesheet_font_size_for_tag(default_style_sheet().config(), tag))
}

pub(crate) fn text_primary_color() -> Color {
    current_style_sheet()
        .ok()
        .and_then(|sheet| {
            beuvy_runtime::style::resolve_color_value_with_config(
                sheet.config(),
                "var(--color-primary)",
            )
        })
        .unwrap_or_else(beuvy_runtime::style::text_primary_color)
}

pub(crate) fn resolve_color_value(raw: &str) -> Option<Color> {
    current_style_sheet()
        .ok()
        .and_then(|sheet| {
            beuvy_runtime::style::resolve_color_value_with_config(sheet.config(), raw)
        })
        .or_else(|| beuvy_runtime::style::resolve_color_value(raw))
}

pub(crate) fn scrollbar_width() -> f32 {
    current_style_sheet()
        .ok()
        .map(|sheet| sheet.config().spacing.scrollbar_width)
        .unwrap_or_else(beuvy_runtime::style::scrollbar_width)
}

fn current_style_sheet() -> Result<Arc<UiStyleSheet>, DeclarativeUiAssetLoadError> {
    let source = style_source().clone();
    if matches!(source, BeuvyStyleSource::BuiltIn) {
        return Ok(Arc::new(default_style_sheet().clone()));
    }
    let modified = source_file_modified(&source);

    let lock = STYLE_SHEET_SNAPSHOT.get_or_init(|| RwLock::new(None));
    if let Some(snapshot) = lock
        .read()
        .expect("beuvy style sheet lock should not be poisoned")
        .as_ref()
        && snapshot.source == source
        && snapshot.modified == modified
    {
        return Ok(Arc::clone(&snapshot.sheet));
    }

    let raw = read_style_source(&source).map_err(|error| {
        std::io::Error::new(
            error.kind(),
            format!(
                "failed to read beuvy styles file {}: {error}",
                display_style_source(&source)
            ),
        )
    })?;
    let sheet = Arc::new(
        compose_style_sheet(default_style_sheet(), &raw)
            .map_err(|error| style_error(error.reason))?,
    );
    *lock
        .write()
        .expect("beuvy style sheet lock should not be poisoned") = Some(BeuvyStyleSheetSnapshot {
        source,
        modified,
        sheet: Arc::clone(&sheet),
    });
    Ok(sheet)
}

fn read_style_source(source: &BeuvyStyleSource) -> std::io::Result<String> {
    match source {
        BeuvyStyleSource::BuiltIn => Ok(String::new()),
        BeuvyStyleSource::File(path) => fs::read_to_string(path),
    }
}

fn source_file_modified(source: &BeuvyStyleSource) -> Option<SystemTime> {
    let BeuvyStyleSource::File(path) = source else {
        return None;
    };
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}

fn display_style_source(source: &BeuvyStyleSource) -> &str {
    match source {
        BeuvyStyleSource::BuiltIn => "<built-in>",
        BeuvyStyleSource::File(path) => path.as_str(),
    }
}

fn style_error(message: impl Into<String>) -> DeclarativeUiAssetLoadError {
    DeclarativeUiAssetLoadError::InvalidDsl(format!("invalid styles.css: {}", message.into()))
}
