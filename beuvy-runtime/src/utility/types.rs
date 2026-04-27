use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UtilityVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityJustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityAlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityAlignContent {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityAlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityFlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityOverflowAxis {
    Visible,
    Clip,
    Hidden,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityDisplay {
    Flex,
    Grid,
    None,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityPositionType {
    Relative,
    Absolute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityStateVariant {
    Hover,
    Active,
    Focus,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityTransitionProperty {
    All,
    Colors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UtilityTransitionTiming {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UtilityRect {
    pub left: Option<UtilityVal>,
    pub right: Option<UtilityVal>,
    pub top: Option<UtilityVal>,
    pub bottom: Option<UtilityVal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UtilityVisualStylePatch {
    pub background_color: Option<String>,
    pub text_color: Option<String>,
    pub border_color: Option<String>,
    pub outline_width: Option<UtilityVal>,
    pub outline_color: Option<String>,
    pub opacity: Option<f32>,
    pub transition_property: Option<UtilityTransitionProperty>,
    pub transition_duration_ms: Option<f32>,
    pub transition_timing: Option<UtilityTransitionTiming>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UtilityStylePatch {
    pub width: Option<UtilityVal>,
    pub height: Option<UtilityVal>,
    pub min_width: Option<UtilityVal>,
    pub min_height: Option<UtilityVal>,
    pub max_width: Option<UtilityVal>,
    pub max_height: Option<UtilityVal>,
    pub flex_direction: Option<UtilityFlexDirection>,
    pub justify_content: Option<UtilityJustifyContent>,
    pub align_items: Option<UtilityAlignItems>,
    pub align_content: Option<UtilityAlignContent>,
    pub align_self: Option<UtilityAlignSelf>,
    pub flex_wrap: Option<UtilityFlexWrap>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<UtilityVal>,
    pub row_gap: Option<UtilityVal>,
    pub column_gap: Option<UtilityVal>,
    pub padding: Option<UtilityRect>,
    pub margin: Option<UtilityRect>,
    pub border: Option<UtilityRect>,
    pub border_radius: Option<UtilityVal>,
    pub overflow_x: Option<UtilityOverflowAxis>,
    pub overflow_y: Option<UtilityOverflowAxis>,
    pub display: Option<UtilityDisplay>,
    pub position_type: Option<UtilityPositionType>,
    pub left: Option<UtilityVal>,
    pub right: Option<UtilityVal>,
    pub top: Option<UtilityVal>,
    pub bottom: Option<UtilityVal>,
    pub text_size: Option<f32>,
    pub visual: UtilityVisualStylePatch,
    pub hover: UtilityVisualStylePatch,
    pub active: UtilityVisualStylePatch,
    pub focus: UtilityVisualStylePatch,
    pub disabled: UtilityVisualStylePatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseUtilityError {
    pub token: String,
    pub reason: String,
}

impl ParseUtilityError {
    pub(crate) fn new(token: &str, reason: impl Into<String>) -> Self {
        Self {
            token: token.to_string(),
            reason: reason.into(),
        }
    }
}
