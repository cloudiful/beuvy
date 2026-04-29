use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, parse_child_nodes, parse_class_bindings, parse_conditional,
    parse_ref_binding, parse_show_attr, parse_text_content, reject_legacy_attrs,
    reject_legacy_bind_attrs, reject_style_attrs,
};
use crate::style::font_size_for_tag;
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_label_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs(node)?;
    Ok(DeclarativeUiNode::Label {
        node_id: String::new(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        content: parse_text_content(node)?,
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        ref_binding: parse_ref_binding(node)?,
        style: crate::DeclarativeTextStyle {
            size: font_size_for_tag("label"),
            color: None,
            visual_style: crate::DeclarativeVisualStyle::default(),
            state_visual_styles: crate::DeclarativeStateVisualStyles::default(),
        },
        for_target: attr(node, "for").map(str::to_string),
        children: parse_child_nodes(node, state_specs)?,
    })
}
