use crate::ast::{DeclarativeTextKind, DeclarativeUiNode};
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, parse_class_bindings, parse_conditional, parse_ref_binding,
    parse_show_attr, parse_state_visual_styles, parse_text_content, parse_text_style,
    parse_visual_style, reject_style_attrs,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_text_node(
    node: XmlNode<'_, '_>,
    tag: &str,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_style_attrs(node)?;
    let mut style = parse_text_style(node, tag)?;
    style.visual_style = parse_visual_style(node)?;
    style.state_visual_styles = parse_state_visual_styles(node)?;
    Ok(DeclarativeUiNode::Text {
        node_id: String::new(),
        kind: text_kind(tag),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        content: parse_text_content(node)?,
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        ref_binding: parse_ref_binding(node)?,
        style,
    })
}

fn text_kind(tag: &str) -> DeclarativeTextKind {
    match tag {
        "p" => DeclarativeTextKind::Paragraph,
        "small" => DeclarativeTextKind::Small,
        "legend" => DeclarativeTextKind::Legend,
        "strong" => DeclarativeTextKind::Strong,
        "em" => DeclarativeTextKind::Emphasis,
        "h1" => DeclarativeTextKind::Heading { level: 1 },
        "h2" => DeclarativeTextKind::Heading { level: 2 },
        "h3" => DeclarativeTextKind::Heading { level: 3 },
        "h4" => DeclarativeTextKind::Heading { level: 4 },
        "h5" => DeclarativeTextKind::Heading { level: 5 },
        "h6" => DeclarativeTextKind::Heading { level: 6 },
        _ => DeclarativeTextKind::Generic,
    }
}
