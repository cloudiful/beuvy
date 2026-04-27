use super::*;
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static RUNTIME_STYLE_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn stylesheet_expands_apply_and_variants() {
    let sheet = parse_style_sheet(
        r#"
            @theme {
                --color-brand: #3366FF;
            }

            @utility button-colors {
                @apply bg-brand text-brand;
            }
            "#,
    )
    .expect("stylesheet should parse");

    let patch = parse_style_classes_with_sheet(&sheet, "hover:button-colors rounded-control")
        .expect("classes should expand");

    assert_eq!(
        patch.hover.background_color.as_deref(),
        Some("var(--color-brand)")
    );
    assert_eq!(
        patch.hover.text_color.as_deref(),
        Some("var(--color-brand)")
    );
    assert_eq!(patch.hover.border_color, None);
}

#[test]
fn runtime_stylesheet_applies_project_overlay_utilities() {
    let _guard = RUNTIME_STYLE_LOCK.lock().expect("runtime style test lock");
    let previous_source = runtime_style_source().clone();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("unix time")
        .as_nanos();
    let temp_path = std::env::temp_dir().join(format!("beuvy-runtime-runtime-style-{unique}.css"));

    fs::write(
        &temp_path,
        r#"
            @theme {
                --color-brand: #3366FF;
            }

            @utility btn-bordered {
                @apply bg-brand border-brand;
            }
            "#,
    )
    .expect("temporary runtime stylesheet should write");

    replace_runtime_style_source(RuntimeStyleSource::file(
        temp_path.to_string_lossy().into_owned(),
    ));

    let sheet = runtime_style_sheet();
    let patch = parse_style_classes_with_sheet(&sheet, "button-root btn-bordered")
        .expect("runtime stylesheet should resolve project utility");
    assert_eq!(
        patch.visual.background_color.as_deref(),
        Some("var(--color-brand)")
    );
    assert_eq!(
        patch.visual.border_color.as_deref(),
        Some("var(--color-brand)")
    );

    replace_runtime_style_source(previous_source);
    let _ = fs::remove_file(temp_path);
}
