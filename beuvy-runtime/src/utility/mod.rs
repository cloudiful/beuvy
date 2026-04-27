mod builtin;
pub(crate) mod parser;
mod tests;
mod types;

/// Tailwind-like utility-class parsing and style patch types.
pub use parser::parse_utility_classes;
#[doc(hidden)]
pub use parser::parse_utility_classes_with_config;
pub use types::{
    ParseUtilityError, UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay,
    UtilityFlexDirection, UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis,
    UtilityPositionType, UtilityRect, UtilityStateVariant, UtilityStylePatch,
    UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal, UtilityVisualStylePatch,
};
