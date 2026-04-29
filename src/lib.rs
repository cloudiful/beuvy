#![doc = r#"
`beuvy` is the facade crate for the Beuvy UI stack.

By default it exposes both layers:

- the reusable runtime controls and style system from [`beuvy-runtime`]
- the optional declarative authoring layer that parses and materializes UI
  assets on top of that runtime

Feature flags:

- `runtime` (default): re-export the low-level Bevy UI kit from
  [`beuvy-runtime`]
- `declarative` (default): enable the parser, asset loader, bindings, and shell
  materialization APIs
- `vue`: current alias for the high-level declarative authoring layer; reserved
  for future Vue-flavored surface expansion
"#]

#[cfg(feature = "runtime")]
pub use beuvy_runtime::{
    AddButton, AddInput, AddSelect, AddSelectOption, AddText, MouseWheelScroll, RuntimeStyleSource,
    Select, SelectPanel, SelectValueChangedMessage, StyleSheetError, UiKitPlugin, UiStyleSheet,
    button, compose_style_sheet, default_select_node, default_style_sheet, input,
    interaction_style as state_style, parse_style_classes_with_sheet, parse_style_sheet,
    parse_utility_classes, replace_runtime_style_source, runtime_style_sheet, runtime_style_source,
    scroll, scroll_container_node, select, selected_option, stylesheet,
    stylesheet_font_size_for_tag, sync_select_label, text, trigger_label_entity, utility,
};

#[cfg(feature = "declarative")]
mod ast;
#[cfg(feature = "declarative")]
mod basic;
#[cfg(feature = "declarative")]
mod error;
#[cfg(feature = "declarative")]
mod parser;
#[cfg(feature = "declarative")]
mod runtime;
#[cfg(feature = "declarative")]
mod style;
#[cfg(feature = "declarative")]
mod value;

#[cfg(feature = "declarative")]
pub use ast::{
    DeclarativeAlignContent, DeclarativeAlignItems, DeclarativeAlignSelf, DeclarativeBorderRadius,
    DeclarativeClassBinding, DeclarativeComputedLocal, DeclarativeConditionExpr,
    DeclarativeConditional, DeclarativeDisplay, DeclarativeEventBinding, DeclarativeEventKind,
    DeclarativeFlexDirection, DeclarativeFlexWrap, DeclarativeForEach, DeclarativeJustifyContent,
    DeclarativeLiteral, DeclarativeLocalizedTextArg, DeclarativeNodeStyle,
    DeclarativeNodeStyleBinding, DeclarativeNumber, DeclarativeOnClick, DeclarativeOverflowAxis,
    DeclarativePositionType, DeclarativeRefSource, DeclarativeRuntimeExpr, DeclarativeScriptType,
    DeclarativeSelectOption, DeclarativeStateAssignment, DeclarativeStateVisualStyles,
    DeclarativeTextKeySource, DeclarativeTextStyle, DeclarativeTransitionProperty,
    DeclarativeTransitionTiming, DeclarativeUiAsset, DeclarativeUiNode, DeclarativeUiRect,
    DeclarativeUiTextContent, DeclarativeUiTextSegment, DeclarativeVal, DeclarativeValueExpr,
    DeclarativeVisualStyle,
};
#[cfg(feature = "declarative")]
pub use error::DeclarativeUiAssetLoadError;
#[cfg(feature = "declarative")]
pub use parser::{
    DeclarativeActionSpec, parse_declarative_ui_asset, resolve_action_spec, set_action_resolver,
};
#[cfg(feature = "declarative")]
pub use runtime::{
    DeclarativeAppliedTemplateHotReload, DeclarativeClassBindings,
    DeclarativeCheckedBinding, DeclarativeConditionalChainState,
    DeclarativeConditionalSubtree, DeclarativeDisabledExpr, DeclarativeEventBindings,
    DeclarativeLabelForTarget, DeclarativeLocalState, DeclarativeModelBinding, DeclarativeNodeId,
    DeclarativeNodeStyleBindingComponent, DeclarativeOnClickAssignment, DeclarativeRefBinding,
    DeclarativeRefRects, DeclarativeResolvedRef, DeclarativeRootComputedLocals,
    DeclarativeRootUiAsset, DeclarativeRootViewModel, DeclarativeShowExpr, DeclarativeTextBinding,
    DeclarativeUiAssetLoader, DeclarativeUiBuildContext, DeclarativeUiPlugin,
    DeclarativeUiRuntimeValues, DeclarativeUiSlot, DeclarativeUiSlots, DeclarativeValueBinding,
    ResolvedDeclarativeEventBinding, apply_node_style, direct_conditional_chain_states,
    load_internal_declarative_ui_shell, materialize_declarative_ui_shell_on_entity_in_world,
    materialize_internal_declarative_ui_shell_on_entity_in_world, parse_hex_color,
    rematerialize_declarative_container_children_in_world, resolve_path, resolve_runtime_path,
    runtime_visual_styles, set_ref_resolver, spawn_declarative_ui_tree_collect_slots,
    spawn_declarative_ui_tree_collect_slots_in_world,
};
#[cfg(feature = "declarative")]
pub use style::{BeuvyStyleSource, replace_style_source};
#[cfg(feature = "declarative")]
pub use value::UiValue;
