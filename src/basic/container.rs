use crate::ast::{DeclarativeContainerKind, DeclarativeUiNode};
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, parse_bool_or_condition_attr, parse_child_nodes,
    parse_class_bindings, parse_conditional, parse_event_bindings, parse_node_style,
    parse_node_style_binding, parse_ref_binding, parse_show_attr, parse_state_visual_styles,
    parse_visual_style, reject_legacy_attrs, reject_legacy_bind_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_container_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;

    let kind = match node.tag_name().name() {
        "form" => DeclarativeContainerKind::Form,
        "fieldset" => DeclarativeContainerKind::Fieldset,
        "label" => DeclarativeContainerKind::Label,
        "ul" => DeclarativeContainerKind::UnorderedList,
        "ol" => DeclarativeContainerKind::OrderedList,
        "li" => DeclarativeContainerKind::ListItem,
        _ => DeclarativeContainerKind::Generic,
    };
    let (disabled, disabled_expr) = if matches!(kind, DeclarativeContainerKind::Fieldset) {
        parse_bool_or_condition_attr(node, "disabled", state_specs)?
    } else {
        (false, None)
    };

    Ok(DeclarativeUiNode::Container {
        node_id: String::new(),
        kind,
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        node: parse_node_style(node)?,
        style_binding: parse_node_style_binding(node)?,
        outlet: attr(node, "slot").map(str::to_string),
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        disabled,
        disabled_expr,
        label_for: matches!(kind, DeclarativeContainerKind::Label)
            .then(|| attr(node, "for").map(str::to_string))
            .flatten(),
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
        ref_binding: parse_ref_binding(node)?,
        event_bindings: parse_event_bindings(node)?,
        children: parse_child_nodes(node, state_specs)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeContainerKind, DeclarativeRuntimeExpr, DeclarativeUiNode, parse_declarative_ui_asset};

    #[test]
    fn style_binding_parses_for_container_nodes() {
        let asset = parse_declarative_ui_asset(
            r#"<template><section :style="{ left: popup.left, top: popup.top }" /></template>"#,
        )
        .expect("container with style binding should parse");

        let DeclarativeUiNode::Container { style_binding, .. } = asset.root else {
            panic!("expected container node");
        };
        let style_binding = style_binding.expect("style binding");
        assert!(matches!(
            style_binding.left,
            Some(DeclarativeRuntimeExpr::FieldAccess { .. })
        ));
        assert!(matches!(
            style_binding.top,
            Some(DeclarativeRuntimeExpr::FieldAccess { .. })
        ));
    }

    #[test]
    fn label_parses_as_container_kind() {
        let asset = parse_declarative_ui_asset(
            r#"<template><label for="pilot"><input name="pilot" /></label></template>"#,
        )
        .expect("label should parse");

        let DeclarativeUiNode::Container {
            kind,
            label_for,
            children,
            ..
        } = asset.root
        else {
            panic!("expected container node");
        };
        assert_eq!(kind, DeclarativeContainerKind::Label);
        assert_eq!(label_for.as_deref(), Some("pilot"));
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn fieldset_parses_disabled_attr() {
        let asset =
            parse_declarative_ui_asset(r#"<template><fieldset disabled="" /></template>"#)
                .expect("fieldset should parse");

        let DeclarativeUiNode::Container { kind, disabled, .. } = asset.root else {
            panic!("expected container node");
        };
        assert_eq!(kind, DeclarativeContainerKind::Fieldset);
        assert!(disabled);
    }
}
