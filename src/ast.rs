use beuvy_runtime::input::InputType;
use beuvy_runtime::utility::{UtilityTransitionProperty, UtilityTransitionTiming};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Asset, TypePath)]
pub struct DeclarativeUiAsset {
    pub root_state: Vec<DeclarativeStateAssignment>,
    pub root_computed: Vec<DeclarativeComputedLocal>,
    pub root: DeclarativeUiNode,
}

#[derive(Debug, Clone)]
pub enum DeclarativeUiNode {
    Container {
        node_id: String,
        semantic_tag: Option<DeclarativeContainerTag>,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        node: DeclarativeNodeStyle,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        outlet: Option<String>,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
        ref_binding: Option<DeclarativeRefSource>,
        event_bindings: Vec<DeclarativeEventBinding>,
        children: Vec<DeclarativeUiNode>,
    },
    Text {
        node_id: String,
        semantic_tag: Option<DeclarativeTextTag>,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        content: DeclarativeUiTextContent,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        ref_binding: Option<DeclarativeRefSource>,
        style: DeclarativeTextStyle,
    },
    Image {
        node_id: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        ref_binding: Option<DeclarativeRefSource>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        src: String,
        src_binding: Option<String>,
        alt: String,
        alt_binding: Option<String>,
        node_override: Option<DeclarativeNodeStyle>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
    },
    Link {
        node_id: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        ref_binding: Option<DeclarativeRefSource>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        event_bindings: Vec<DeclarativeEventBinding>,
        href: String,
        href_binding: Option<String>,
        content: DeclarativeUiTextContent,
        text_style: DeclarativeTextStyle,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
    },
    Hr {
        node_id: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        ref_binding: Option<DeclarativeRefSource>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        node_override: Option<DeclarativeNodeStyle>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
    },
    Label {
        node_id: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        content: DeclarativeUiTextContent,
        conditional: DeclarativeConditional,
        show_expr: Option<DeclarativeConditionExpr>,
        ref_binding: Option<DeclarativeRefSource>,
        style: DeclarativeTextStyle,
        for_target: Option<String>,
        children: Vec<DeclarativeUiNode>,
    },
    Button {
        node_id: String,
        name: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        content: DeclarativeUiTextContent,
        conditional: DeclarativeConditional,
        onclick: Option<DeclarativeOnClick>,
        event_bindings: Vec<DeclarativeEventBinding>,
        ref_binding: Option<DeclarativeRefSource>,
        node_override: Option<DeclarativeNodeStyle>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
        disabled: bool,
        disabled_expr: Option<DeclarativeConditionExpr>,
        show_expr: Option<DeclarativeConditionExpr>,
        label_size_override: Option<f32>,
    },
    Input {
        node_id: String,
        name: String,
        input_type: InputType,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        conditional: DeclarativeConditional,
        value: String,
        value_binding: Option<String>,
        model_binding: Option<String>,
        checked: bool,
        checked_binding: Option<String>,
        ref_binding: Option<DeclarativeRefSource>,
        event_bindings: Vec<DeclarativeEventBinding>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        placeholder: String,
        size_chars: Option<usize>,
        rows: Option<usize>,
        min: Option<f32>,
        max: Option<f32>,
        step: Option<f32>,
        node_override: Option<DeclarativeNodeStyle>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
        disabled: bool,
        disabled_expr: Option<DeclarativeConditionExpr>,
        show_expr: Option<DeclarativeConditionExpr>,
    },
    Select {
        node_id: String,
        name: String,
        class: String,
        class_bindings: Vec<DeclarativeClassBinding>,
        conditional: DeclarativeConditional,
        value: String,
        value_binding: Option<String>,
        model_binding: Option<String>,
        ref_binding: Option<DeclarativeRefSource>,
        event_bindings: Vec<DeclarativeEventBinding>,
        style_binding: Option<DeclarativeNodeStyleBinding>,
        options: Vec<DeclarativeSelectOption>,
        node_override: Option<DeclarativeNodeStyle>,
        visual_style: DeclarativeVisualStyle,
        state_visual_styles: DeclarativeStateVisualStyles,
        disabled: bool,
        disabled_expr: Option<DeclarativeConditionExpr>,
        show_expr: Option<DeclarativeConditionExpr>,
        label_size_override: Option<f32>,
    },
    Template {
        node_id: String,
        for_each: DeclarativeForEach,
        children: Vec<DeclarativeUiNode>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum DeclarativeContainerTag {
    Div,
    Section,
    Header,
    Footer,
    Main,
    Nav,
    Aside,
    Article,
    Form,
    Fieldset,
    Ul,
    Ol,
    Li,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum DeclarativeTextTag {
    Span,
    P,
    Legend,
    Small,
    Strong,
    Em,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum DeclarativeRefSource {
    Static(String),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeclarativeClassBinding {
    Conditional {
        class_name: String,
        condition: DeclarativeConditionExpr,
    },
    RuntimeExpr {
        expr: DeclarativeRuntimeExpr,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeSelectOption {
    pub value: Option<String>,
    pub value_binding: Option<String>,
    pub content: DeclarativeUiTextContent,
    pub selected: bool,
    pub disabled: bool,
    pub disabled_expr: Option<DeclarativeConditionExpr>,
    pub conditional: DeclarativeConditional,
    pub repeat: Option<DeclarativeForEach>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DeclarativeUiTextContent {
    Static {
        #[serde(default)]
        text: String,
    },
    Bind {
        path: String,
    },
    Segments {
        segments: Vec<DeclarativeUiTextSegment>,
    },
    I18n {
        key: DeclarativeTextKeySource,
        #[serde(default)]
        localized_text_args: Vec<DeclarativeLocalizedTextArg>,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum DeclarativeUiTextSegment {
    Static { text: String },
    Bind { path: String },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeStateAssignment {
    pub name: String,
    pub mutable: bool,
    pub type_hint: DeclarativeScriptType,
    pub value: DeclarativeLiteral,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeComputedLocal {
    pub name: String,
    pub expr: DeclarativeRuntimeExpr,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeRuntimeStmt {
    Const {
        name: String,
        expr: DeclarativeRuntimeExpr,
    },
    If {
        condition: DeclarativeRuntimeExpr,
        then_branch: Vec<DeclarativeRuntimeStmt>,
        else_branch: Vec<DeclarativeRuntimeStmt>,
    },
    Return(DeclarativeRuntimeExpr),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum DeclarativeScriptType {
    String,
    Bool,
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub enum DeclarativeConditional {
    #[default]
    Always,
    If(DeclarativeConditionExpr),
    ElseIf(DeclarativeConditionExpr),
    Else,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeConditionExpr {
    Binding(String),
    Equals {
        name: String,
        value: DeclarativeLiteral,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeclarativeForEach {
    pub source: String,
    pub item_alias: String,
    pub index_alias: Option<String>,
    pub key_expr: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeOnClick {
    Assign {
        name: String,
        value: DeclarativeLiteral,
    },
    DispatchCall {
        action_id: String,
        params: BTreeMap<String, DeclarativeValueExpr>,
    },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeLiteral {
    String(String),
    Bool(bool),
    Number(DeclarativeNumber),
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
pub enum DeclarativeNumber {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeValueExpr {
    Literal(DeclarativeLiteral),
    Binding(String),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DeclarativeLocalizedTextArg {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeTextKeySource {
    Static(String),
    Binding(String),
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct DeclarativeNodeStyle {
    #[serde(default)]
    pub width: Option<DeclarativeVal>,
    #[serde(default)]
    pub height: Option<DeclarativeVal>,
    #[serde(default)]
    pub min_width: Option<DeclarativeVal>,
    #[serde(default)]
    pub min_height: Option<DeclarativeVal>,
    #[serde(default)]
    pub max_width: Option<DeclarativeVal>,
    #[serde(default)]
    pub max_height: Option<DeclarativeVal>,
    #[serde(default)]
    pub flex_direction: Option<DeclarativeFlexDirection>,
    #[serde(default)]
    pub justify_content: Option<DeclarativeJustifyContent>,
    #[serde(default)]
    pub align_items: Option<DeclarativeAlignItems>,
    #[serde(default)]
    pub align_content: Option<DeclarativeAlignContent>,
    #[serde(default)]
    pub align_self: Option<DeclarativeAlignSelf>,
    #[serde(default)]
    pub flex_wrap: Option<DeclarativeFlexWrap>,
    #[serde(default)]
    pub flex_grow: Option<f32>,
    #[serde(default)]
    pub flex_shrink: Option<f32>,
    #[serde(default)]
    pub flex_basis: Option<DeclarativeVal>,
    #[serde(default)]
    pub row_gap: Option<DeclarativeVal>,
    #[serde(default)]
    pub column_gap: Option<DeclarativeVal>,
    #[serde(default)]
    pub padding: Option<DeclarativeUiRect>,
    #[serde(default)]
    pub margin: Option<DeclarativeUiRect>,
    #[serde(default)]
    pub border: Option<DeclarativeUiRect>,
    #[serde(default)]
    pub border_radius: Option<DeclarativeBorderRadius>,
    #[serde(default)]
    pub overflow_x: Option<DeclarativeOverflowAxis>,
    #[serde(default)]
    pub overflow_y: Option<DeclarativeOverflowAxis>,
    #[serde(default)]
    pub display: Option<DeclarativeDisplay>,
    #[serde(default)]
    pub position_type: Option<DeclarativePositionType>,
    #[serde(default)]
    pub left: Option<DeclarativeVal>,
    #[serde(default)]
    pub right: Option<DeclarativeVal>,
    #[serde(default)]
    pub top: Option<DeclarativeVal>,
    #[serde(default)]
    pub bottom: Option<DeclarativeVal>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct DeclarativeNodeStyleBinding {
    #[serde(default)]
    pub left: Option<DeclarativeRuntimeExpr>,
    #[serde(default)]
    pub top: Option<DeclarativeRuntimeExpr>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DeclarativeRuntimeExpr {
    BindingPath(String),
    Literal(DeclarativeLiteral),
    NumberLiteral(DeclarativeNumber),
    ArrayLiteral(Vec<DeclarativeRuntimeExpr>),
    GetBoundingClientRect {
        target_path: String,
    },
    FieldAccess {
        base: Box<DeclarativeRuntimeExpr>,
        field: String,
    },
    UnaryNot {
        expr: Box<DeclarativeRuntimeExpr>,
    },
    Binary {
        left: Box<DeclarativeRuntimeExpr>,
        op: DeclarativeBinaryOp,
        right: Box<DeclarativeRuntimeExpr>,
    },
    ObjectLiteral(Vec<(String, DeclarativeRuntimeExpr)>),
    MathMin {
        args: Vec<DeclarativeRuntimeExpr>,
    },
    MathMax {
        args: Vec<DeclarativeRuntimeExpr>,
    },
    Conditional {
        condition: Box<DeclarativeRuntimeExpr>,
        then_expr: Box<DeclarativeRuntimeExpr>,
        else_expr: Box<DeclarativeRuntimeExpr>,
    },
    Block(Vec<DeclarativeRuntimeStmt>),
    AnchorPopup {
        anchor_rect: Box<DeclarativeRuntimeExpr>,
        shell_rect: Box<DeclarativeRuntimeExpr>,
        popup_width: Box<DeclarativeRuntimeExpr>,
        popup_min_height: Box<DeclarativeRuntimeExpr>,
        gap: Box<DeclarativeRuntimeExpr>,
        margin: Box<DeclarativeRuntimeExpr>,
    },
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub enum DeclarativeBinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeTextStyle {
    pub size: f32,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub visual_style: DeclarativeVisualStyle,
    #[serde(default)]
    pub state_visual_styles: DeclarativeStateVisualStyles,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct DeclarativeVisualStyle {
    #[serde(default)]
    pub background_color: Option<String>,
    #[serde(default)]
    pub text_color: Option<String>,
    #[serde(default)]
    pub border_color: Option<String>,
    #[serde(default)]
    pub outline_width: Option<DeclarativeVal>,
    #[serde(default)]
    pub outline_color: Option<String>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub transition_property: Option<DeclarativeTransitionProperty>,
    #[serde(default)]
    pub transition_duration_ms: Option<f32>,
    #[serde(default)]
    pub transition_timing: Option<DeclarativeTransitionTiming>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct DeclarativeStateVisualStyles {
    #[serde(default)]
    pub hover: DeclarativeVisualStyle,
    #[serde(default)]
    pub active: DeclarativeVisualStyle,
    #[serde(default)]
    pub focus: DeclarativeVisualStyle,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeTransitionProperty {
    All,
    Colors,
}

impl From<UtilityTransitionProperty> for DeclarativeTransitionProperty {
    fn from(value: UtilityTransitionProperty) -> Self {
        match value {
            UtilityTransitionProperty::All => Self::All,
            UtilityTransitionProperty::Colors => Self::Colors,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeTransitionTiming {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl From<UtilityTransitionTiming> for DeclarativeTransitionTiming {
    fn from(value: UtilityTransitionTiming) -> Self {
        match value {
            UtilityTransitionTiming::Linear => Self::Linear,
            UtilityTransitionTiming::EaseIn => Self::EaseIn,
            UtilityTransitionTiming::EaseOut => Self::EaseOut,
            UtilityTransitionTiming::EaseInOut => Self::EaseInOut,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(tag = "unit", content = "value", rename_all = "snake_case")]
pub enum DeclarativeVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeUiRect {
    #[serde(default)]
    pub left: Option<DeclarativeVal>,
    #[serde(default)]
    pub right: Option<DeclarativeVal>,
    #[serde(default)]
    pub top: Option<DeclarativeVal>,
    #[serde(default)]
    pub bottom: Option<DeclarativeVal>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeclarativeBorderRadius {
    All { radius: DeclarativeVal },
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeFlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeJustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeAlignItems {
    Default,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeAlignContent {
    Default,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeAlignSelf {
    Auto,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeFlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeOverflowAxis {
    Visible,
    Clip,
    Hidden,
    Scroll,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeEventKind {
    Activate,
    Input,
    Change,
    Scroll,
    Wheel,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct DeclarativeEventBinding {
    pub kind: DeclarativeEventKind,
    pub action_id: String,
    #[serde(default)]
    pub params: BTreeMap<String, DeclarativeValueExpr>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativeDisplay {
    Flex,
    Grid,
    Block,
    None,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeclarativePositionType {
    Relative,
    Absolute,
}

impl DeclarativeUiNode {
    pub fn node_id(&self) -> &str {
        match self {
            Self::Container { node_id, .. }
            | Self::Text { node_id, .. }
            | Self::Image { node_id, .. }
            | Self::Link { node_id, .. }
            | Self::Hr { node_id, .. }
            | Self::Label { node_id, .. }
            | Self::Button { node_id, .. }
            | Self::Input { node_id, .. }
            | Self::Select { node_id, .. }
            | Self::Template { node_id, .. } => node_id,
        }
    }

    pub fn set_node_id(&mut self, value: impl Into<String>) {
        let value = value.into();
        match self {
            Self::Container { node_id, .. }
            | Self::Text { node_id, .. }
            | Self::Image { node_id, .. }
            | Self::Link { node_id, .. }
            | Self::Hr { node_id, .. }
            | Self::Label { node_id, .. }
            | Self::Button { node_id, .. }
            | Self::Input { node_id, .. }
            | Self::Select { node_id, .. }
            | Self::Template { node_id, .. } => *node_id = value,
        }
    }

    pub fn children(&self) -> Option<&[DeclarativeUiNode]> {
        match self {
            Self::Container { children, .. }
            | Self::Label { children, .. }
            | Self::Template { children, .. } => Some(children),
            _ => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<DeclarativeUiNode>> {
        match self {
            Self::Container { children, .. }
            | Self::Label { children, .. }
            | Self::Template { children, .. } => Some(children),
            _ => None,
        }
    }
}
