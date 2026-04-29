use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, parse_class_bindings, parse_conditional, parse_node_style,
    parse_node_style_binding, parse_ref_binding, parse_show_attr, parse_state_visual_styles,
    parse_visual_style, reject_legacy_attrs, reject_legacy_bind_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_hr_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;

    Ok(DeclarativeUiNode::Hr {
        node_id: String::new(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        ref_binding: parse_ref_binding(node)?,
        style_binding: parse_node_style_binding(node)?,
        node_override: Some(parse_node_style(node)?),
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeUiNode, parse_declarative_ui_asset};

    #[test]
    fn hr_node_parses() {
        let asset =
            parse_declarative_ui_asset(r#"<template><hr /></template>"#).expect("hr should parse");
        assert!(matches!(asset.root, DeclarativeUiNode::Hr { .. }));
    }
}
