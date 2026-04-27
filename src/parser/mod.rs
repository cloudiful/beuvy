use crate::ast::*;
use crate::basic::button::parse_declarative_button_node;
use crate::basic::div::parse_declarative_div_node;
use crate::basic::input::parse_declarative_input_node;
use crate::basic::select::parse_declarative_select_node;
use crate::error::DeclarativeUiAssetLoadError;
use beuvy_runtime::utility::{
    UtilityAlignContent, UtilityAlignItems, UtilityAlignSelf, UtilityDisplay, UtilityFlexDirection,
    UtilityFlexWrap, UtilityJustifyContent, UtilityOverflowAxis, UtilityPositionType,
    UtilityStylePatch, UtilityTransitionProperty, UtilityTransitionTiming, UtilityVal,
    UtilityVisualStylePatch,
};
use roxmltree::{Document, Node as XmlNode};
use std::collections::BTreeMap;
use std::sync::OnceLock;

mod action;
mod attr;
mod condition;
mod entry;
mod node;
mod onclick;
mod repeat;
mod runtime_expr;
mod script;
mod style;
mod style_binding;

pub(crate) use attr::{
    attr, attr_error, bound_attr, dsl_error, element_children, model_attr, parse_binding_path_expr,
    parse_bool_or_condition_attr, parse_event_bindings, parse_mustache_expr, parse_ref_binding,
    parse_show_attr, parse_usize, reject_hidden_attrs, reject_legacy_attrs,
    reject_legacy_bind_attrs, reject_legacy_event_attrs, reject_style_attrs,
    reject_style_attrs_except, required_attr,
};
pub(crate) use condition::{parse_condition_expr, parse_conditional};
pub use entry::parse_declarative_ui_asset;
pub(crate) use node::{parse_child_nodes, parse_text_content};
pub(crate) use onclick::parse_onclick;
#[allow(unused_imports)]
pub(crate) use style::{parse_class_bindings, parse_utility_class_patch};
pub(crate) use style::{parse_node_style, parse_state_visual_styles, parse_visual_style};
pub(crate) use style_binding::parse_node_style_binding;

use action::parse_literal;
use condition::validate_conditional_chain;
use node::{default_text_size_for_tag, parse_node};
pub(crate) use repeat::parse_v_for;
pub(crate) use runtime_expr::parse_runtime_expr;
pub(crate) use script::DeclarativeStateSpec;
use script::{is_identifier_path, parse_root_script, parse_script_source, parse_state_name};
use style::parse_text_style;

#[derive(Debug, Clone)]
pub struct DeclarativeActionSpec {
    pub action_id: &'static str,
    pub param_names: Vec<&'static str>,
}

pub type ResolveDeclarativeAction = fn(&str) -> Option<DeclarativeActionSpec>;

static ACTION_RESOLVER: OnceLock<ResolveDeclarativeAction> = OnceLock::new();

pub fn set_action_resolver(resolver: ResolveDeclarativeAction) {
    let _ = ACTION_RESOLVER.set(resolver);
}

pub fn resolve_action_spec(name: &str) -> Option<DeclarativeActionSpec> {
    ACTION_RESOLVER.get().and_then(|resolver| resolver(name))
}
