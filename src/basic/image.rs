use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, attr_error, bound_attr, parse_binding_path_expr,
    parse_class_bindings, parse_conditional, parse_node_style, parse_node_style_binding,
    parse_ref_binding, parse_show_attr, parse_state_visual_styles, parse_visual_style,
    reject_legacy_attrs, reject_legacy_bind_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_image_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;

    let src = attr(node, "src").unwrap_or_default().to_string();
    let src_binding = bound_attr(node, "src")
        .map(|expr| parse_binding_path_expr(node, ":src", expr))
        .transpose()?;
    if src_binding.is_none() && src.trim().is_empty() {
        return Err(attr_error(node, "src", "", "<img> requires src or :src"));
    }
    let alt = attr(node, "alt").unwrap_or_default().to_string();
    let alt_binding = bound_attr(node, "alt")
        .map(|expr| parse_binding_path_expr(node, ":alt", expr))
        .transpose()?;

    Ok(DeclarativeUiNode::Image {
        node_id: String::new(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        ref_binding: parse_ref_binding(node)?,
        style_binding: parse_node_style_binding(node)?,
        src,
        src_binding,
        alt,
        alt_binding,
        node_override: Some(parse_node_style(node)?),
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeUiNode, parse_declarative_ui_asset};

    #[test]
    fn image_node_parses_static_and_bound_attrs() {
        let asset = parse_declarative_ui_asset(
            r#"<template><img src="icons/help.png" :alt="hero.name" /></template>"#,
        )
        .expect("image should parse");

        let DeclarativeUiNode::Image {
            src,
            src_binding,
            alt,
            alt_binding,
            ..
        } = asset.root
        else {
            panic!("expected image node");
        };

        assert_eq!(src, "icons/help.png");
        assert_eq!(src_binding, None);
        assert_eq!(alt, "");
        assert_eq!(alt_binding.as_deref(), Some("hero.name"));
    }
}
