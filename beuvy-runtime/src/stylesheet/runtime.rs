use super::model::{RuntimeStyleSource, UiStyleSheet};
use super::parser::{compose_style_sheet, parse_style_sheet};
use std::fs;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::SystemTime;

const DEFAULT_UI_STYLESHEET: &str = concat!(
    include_str!("../styles/base.css"),
    "\n",
    include_str!("../styles/button.css"),
    "\n",
    include_str!("../styles/input.css"),
    "\n",
    include_str!("../styles/select.css"),
    "\n",
    include_str!("../styles/form_item.css"),
    "\n",
);
static RUNTIME_STYLE_SOURCE: OnceLock<RwLock<Arc<RuntimeStyleSource>>> = OnceLock::new();
static RUNTIME_STYLE_SNAPSHOT: OnceLock<RwLock<Option<RuntimeStyleSheetSnapshot>>> =
    OnceLock::new();

#[derive(Debug, Clone)]
struct RuntimeStyleSheetSnapshot {
    source: RuntimeStyleSource,
    modified: Option<SystemTime>,
    sheet: Arc<UiStyleSheet>,
}

pub fn default_style_sheet() -> &'static UiStyleSheet {
    static DEFAULT_STYLE_SHEET: OnceLock<UiStyleSheet> = OnceLock::new();
    DEFAULT_STYLE_SHEET.get_or_init(|| {
        parse_style_sheet(DEFAULT_UI_STYLESHEET).expect("default ui stylesheet should parse")
    })
}

pub fn replace_runtime_style_source(source: RuntimeStyleSource) {
    let lock = RUNTIME_STYLE_SOURCE.get_or_init(|| RwLock::new(Arc::new(source.clone())));
    *lock
        .write()
        .expect("runtime style source lock should not be poisoned") = Arc::new(source);
}

pub fn runtime_style_source() -> &'static RuntimeStyleSource {
    let lock =
        RUNTIME_STYLE_SOURCE.get_or_init(|| RwLock::new(Arc::new(RuntimeStyleSource::default())));
    let snapshot = lock
        .read()
        .expect("runtime style source lock should not be poisoned");
    unsafe { &*Arc::as_ptr(&snapshot) }
}

pub fn runtime_style_sheet() -> Arc<UiStyleSheet> {
    let source = runtime_style_source().clone();
    if matches!(source, RuntimeStyleSource::BuiltIn) {
        return Arc::new(default_style_sheet().clone());
    }
    let modified = source_file_modified(&source);

    let lock = RUNTIME_STYLE_SNAPSHOT.get_or_init(|| RwLock::new(None));
    if let Some(snapshot) = lock
        .read()
        .expect("runtime style sheet lock should not be poisoned")
        .as_ref()
        && snapshot.source == source
        && snapshot.modified == modified
    {
        return Arc::clone(&snapshot.sheet);
    }

    let sheet = Arc::new(load_runtime_style_sheet(&source));
    *lock
        .write()
        .expect("runtime style sheet lock should not be poisoned") =
        Some(RuntimeStyleSheetSnapshot {
            source,
            modified,
            sheet: Arc::clone(&sheet),
        });
    sheet
}

fn load_runtime_style_sheet(source: &RuntimeStyleSource) -> UiStyleSheet {
    let RuntimeStyleSource::File(path) = source else {
        return default_style_sheet().clone();
    };
    let Ok(raw) = fs::read_to_string(path) else {
        return default_style_sheet().clone();
    };

    match compose_style_sheet(default_style_sheet(), &raw) {
        Ok(sheet) => sheet,
        Err(error) => {
            bevy::log::warn!(
                "failed to compose runtime stylesheet {}: {}; falling back to built-in defaults",
                path,
                error.reason
            );
            default_style_sheet().clone()
        }
    }
}

fn source_file_modified(source: &RuntimeStyleSource) -> Option<SystemTime> {
    let RuntimeStyleSource::File(path) = source else {
        return None;
    };
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
}
