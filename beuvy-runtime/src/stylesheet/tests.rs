use super::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

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

    let sheet = runtime::load_runtime_style_sheet(&RuntimeStyleSource::file(
        temp_path.to_string_lossy().into_owned(),
    ));
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

    let _ = fs::remove_file(temp_path);
}
