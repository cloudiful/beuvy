mod animate;

use crate::utility::UtilityTransitionTiming;
use bevy::prelude::*;

pub(crate) use animate::animate_state_visual_styles;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct UiAnimatedColor {
    pub(super) from: Color,
    pub(super) to: Color,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) struct UiStateVisualTransition {
    pub(super) background: Option<UiAnimatedColor>,
    pub(super) border: Option<UiAnimatedColor>,
    pub(super) text: Option<UiAnimatedColor>,
    pub(super) outline: Option<UiAnimatedColor>,
    pub(super) duration_secs: f32,
    pub(super) elapsed_secs: f32,
    pub(super) timing: UtilityTransitionTiming,
}

impl UiStateVisualTransition {
    pub(super) fn has_values(&self) -> bool {
        self.background.is_some()
            || self.border.is_some()
            || self.text.is_some()
            || self.outline.is_some()
    }
}

pub(super) fn transition_color(
    current: Option<Color>,
    target: Option<Color>,
) -> Option<UiAnimatedColor> {
    match (current, target) {
        (Some(from), Some(to)) if !color_nearly_equal(from, to) => {
            Some(UiAnimatedColor { from, to })
        }
        _ => None,
    }
}

pub(super) fn eased_progress(value: f32, timing: UtilityTransitionTiming) -> f32 {
    let value = value.clamp(0.0, 1.0);
    match timing {
        UtilityTransitionTiming::Linear => value,
        UtilityTransitionTiming::EaseIn => value * value,
        UtilityTransitionTiming::EaseOut => 1.0 - (1.0 - value) * (1.0 - value),
        UtilityTransitionTiming::EaseInOut => {
            if value < 0.5 {
                2.0 * value * value
            } else {
                1.0 - (-2.0 * value + 2.0).powi(2) / 2.0
            }
        }
    }
}

pub(super) fn color_nearly_equal(lhs: Color, rhs: Color) -> bool {
    let lhs = lhs.to_srgba().to_f32_array();
    let rhs = rhs.to_srgba().to_f32_array();
    lhs.into_iter()
        .zip(rhs)
        .all(|(lhs, rhs)| (lhs - rhs).abs() <= 0.002)
}
