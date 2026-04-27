use super::*;
use crate::parser::action::parse_literal_for_type;

pub(crate) fn parse_conditional(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeConditional, DeclarativeUiAssetLoadError> {
    if let Some(value) = attr(node, "v-if") {
        return Ok(DeclarativeConditional::If(parse_condition_expr(
            node,
            "v-if",
            value,
            true,
            state_specs,
        )?));
    }
    if let Some(value) = attr(node, "v-else-if") {
        return Ok(DeclarativeConditional::ElseIf(parse_condition_expr(
            node,
            "v-else-if",
            value,
            false,
            state_specs,
        )?));
    }
    if let Some(value) = attr(node, "v-else") {
        if !value.trim().is_empty() {
            return Err(attr_error(
                node,
                "v-else",
                value,
                "v-else does not accept any value",
            ));
        }
        return Ok(DeclarativeConditional::Else);
    }
    Ok(DeclarativeConditional::Always)
}

pub(crate) fn parse_condition_expr(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
    allow_binding_fallback: bool,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeConditionExpr, DeclarativeUiAssetLoadError> {
    if let Some((left, right)) = raw.split_once("===").or_else(|| raw.split_once("==")) {
        let state_name = parse_condition_path(node, name, left.trim())?;
        let value = if let Some(state_spec) = state_specs.get(&state_name).copied() {
            parse_literal_for_type(node, name, right.trim(), state_spec.type_hint)?
        } else {
            parse_literal(node, name, right.trim())?
        };
        return Ok(DeclarativeConditionExpr::Equals {
            name: state_name,
            value,
        });
    }
    if allow_binding_fallback && is_identifier_path(raw.trim()) {
        return Ok(DeclarativeConditionExpr::Binding(raw.trim().to_string()));
    }
    Err(attr_error(node, name, raw, "expected `name === literal`"))
}

fn parse_condition_path(
    node: XmlNode<'_, '_>,
    attr_name: &str,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    if is_identifier_path(raw) {
        Ok(raw.to_string())
    } else {
        Err(attr_error(node, attr_name, raw, "expected binding path"))
    }
}

pub(super) fn validate_conditional_chain(
    parent: XmlNode<'_, '_>,
    children: &[DeclarativeUiNode],
) -> Result<(), DeclarativeUiAssetLoadError> {
    for (index, child) in children.iter().enumerate() {
        let Some(conditional) = node_conditional(child) else {
            continue;
        };
        if !matches!(
            conditional,
            DeclarativeConditional::Else | DeclarativeConditional::ElseIf(_)
        ) {
            continue;
        }
        if index == 0 {
            return Err(dsl_error(
                parent,
                "else-if/else must follow a sibling with if/else-if",
            ));
        }
        let previous = node_conditional(&children[index - 1]);
        if !matches!(
            previous,
            Some(DeclarativeConditional::If(_)) | Some(DeclarativeConditional::ElseIf(_))
        ) {
            return Err(dsl_error(
                parent,
                "else-if/else must follow a sibling with if/else-if",
            ));
        }
    }
    Ok(())
}

pub(super) fn node_conditional(node: &DeclarativeUiNode) -> Option<&DeclarativeConditional> {
    match node {
        DeclarativeUiNode::Container { conditional, .. }
        | DeclarativeUiNode::Text { conditional, .. }
        | DeclarativeUiNode::Button { conditional, .. }
        | DeclarativeUiNode::Input { conditional, .. }
        | DeclarativeUiNode::Select { conditional, .. } => Some(conditional),
        DeclarativeUiNode::Template { .. } => None,
    }
}
