use crate::interaction_style::UiStateVisualStyles;
use crate::utility::{UtilityStylePatch, UtilityVisualStylePatch};

pub fn token_border_style(token: &str) -> UiStateVisualStyles {
    UiStateVisualStyles {
        base: UtilityVisualStylePatch {
            border_color: Some(format!("var(--color-{token})")),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn root_visual_styles_from_patch(patch: &UtilityStylePatch) -> Option<UiStateVisualStyles> {
    let styles = UiStateVisualStyles {
        base: visual_patch_without_text(&patch.visual),
        hover: visual_patch_without_text(&patch.hover),
        active: visual_patch_without_text(&patch.active),
        focus: visual_patch_without_text(&patch.focus),
        disabled: visual_patch_without_text(&patch.disabled),
    };
    (!styles.is_empty()).then_some(styles)
}

pub fn text_visual_styles_from_patch(patch: &UtilityStylePatch) -> Option<UiStateVisualStyles> {
    let styles = UiStateVisualStyles {
        base: text_visual_patch_only(&patch.visual),
        hover: text_visual_patch_only(&patch.hover),
        active: text_visual_patch_only(&patch.active),
        focus: text_visual_patch_only(&patch.focus),
        disabled: text_visual_patch_only(&patch.disabled),
    };
    (!styles.is_empty()).then_some(styles)
}

fn visual_patch_without_text(patch: &UtilityVisualStylePatch) -> UtilityVisualStylePatch {
    UtilityVisualStylePatch {
        background_color: patch.background_color.clone(),
        border_color: patch.border_color.clone(),
        outline_width: patch.outline_width,
        outline_color: patch.outline_color.clone(),
        opacity: patch.opacity,
        transition_property: patch.transition_property,
        transition_duration_ms: patch.transition_duration_ms,
        transition_timing: patch.transition_timing,
        ..Default::default()
    }
}

fn text_visual_patch_only(patch: &UtilityVisualStylePatch) -> UtilityVisualStylePatch {
    UtilityVisualStylePatch {
        text_color: patch.text_color.clone(),
        opacity: patch.opacity,
        transition_property: patch.transition_property,
        transition_duration_ms: patch.transition_duration_ms,
        transition_timing: patch.transition_timing,
        ..Default::default()
    }
}
