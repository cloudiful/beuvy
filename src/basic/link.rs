use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, attr_error, bound_attr, parse_binding_path_expr,
    parse_class_bindings, parse_conditional, parse_event_bindings, parse_node_style_binding,
    parse_ref_binding, parse_show_attr, parse_state_visual_styles, parse_text_content,
    parse_text_style, parse_visual_style, reject_legacy_attrs, reject_legacy_bind_attrs,
    reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_link_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;

    let href = attr(node, "href").unwrap_or_default().to_string();
    let href_binding = bound_attr(node, "href")
        .map(|expr| parse_binding_path_expr(node, ":href", expr))
        .transpose()?;
    if href_binding.is_none() && href.trim().is_empty() {
        return Err(attr_error(node, "href", "", "<a> requires href or :href"));
    }

    Ok(DeclarativeUiNode::Link {
        node_id: String::new(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        ref_binding: parse_ref_binding(node)?,
        style_binding: parse_node_style_binding(node)?,
        event_bindings: parse_event_bindings(node)?,
        href,
        href_binding,
        content: parse_text_content(node)?,
        text_style: parse_text_style(node, "a")?,
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeUiNode, DeclarativeUiTextContent, parse_declarative_ui_asset};

    #[test]
    fn link_node_parses_href_and_text() {
        let asset = parse_declarative_ui_asset(
            r#"<template><a href="/docs/getting-started">Docs</a></template>"#,
        )
        .expect("link should parse");

        let DeclarativeUiNode::Link { href, content, .. } = asset.root else {
            panic!("expected link node");
        };

        assert_eq!(href, "/docs/getting-started");
        assert!(matches!(
            content,
            DeclarativeUiTextContent::Static { text } if text == "Docs"
        ));
    }
}
