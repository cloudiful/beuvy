#![doc = r#"
`beuvy` is a declarative UI layer for Bevy.

It parses reusable UI assets into typed Bevy runtime structures, then
materializes those structures with the controls and style utilities provided by
[`beuvy-runtime`].

The main entry points are:

- [`DeclarativeUiPlugin`] to register asset loading and runtime systems.
- [`parse_declarative_ui_asset`] to parse a declarative UI asset from text.
- [`materialize_declarative_ui_shell_on_entity_in_world`] to attach a parsed UI
  shell to an existing entity.

Use `beuvy-runtime` directly when you only need controls, utility classes, and
state-driven UI styling.
"#]

mod ast;
mod basic;
mod error;
mod parser;
mod runtime;
mod style;
mod value;

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
pub use error::DeclarativeUiAssetLoadError;
pub use parser::{
    DeclarativeActionSpec, parse_declarative_ui_asset, resolve_action_spec, set_action_resolver,
};
pub use runtime::{
    DeclarativeAppliedTemplateHotReload, DeclarativeClassBindings,
    DeclarativeConditionalChainState, DeclarativeConditionalSubtree, DeclarativeDisabledExpr,
    DeclarativeEventBindings, DeclarativeLocalState, DeclarativeModelBinding, DeclarativeNodeId,
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
pub use style::{BeuvyStyleSource, replace_style_source};
pub use value::UiValue;
