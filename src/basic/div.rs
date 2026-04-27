use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, parse_child_nodes, parse_class_bindings, parse_conditional,
    parse_event_bindings, parse_node_style, parse_node_style_binding, parse_ref_binding,
    parse_show_attr, parse_state_visual_styles, parse_visual_style, reject_legacy_attrs,
    reject_legacy_bind_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_div_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;
    Ok(DeclarativeUiNode::Container {
        node_id: String::new(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        node: parse_node_style(node)?,
        style_binding: parse_node_style_binding(node)?,
        outlet: attr(node, "slot").map(str::to_string),
        conditional: parse_conditional(node, state_specs)?,
        show_expr: parse_show_attr(node, state_specs)?,
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
        ref_binding: parse_ref_binding(node)?,
        event_bindings: parse_event_bindings(node)?,
        children: parse_child_nodes(node, state_specs)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeRuntimeExpr, DeclarativeUiNode, parse_declarative_ui_asset};

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
    fn style_binding_rejects_non_object_syntax() {
        let error =
            parse_declarative_ui_asset(r#"<template><section :style="popup.left" /></template>"#)
                .expect_err("non-object style binding should fail");
        assert!(error.to_string().contains("expected object syntax"));
    }

    #[test]
    fn style_binding_rejects_unsupported_keys_and_values() {
        let key_error = parse_declarative_ui_asset(
            r#"<template><section :style="{ right: popup.right }" /></template>"#,
        )
        .expect_err("unsupported key should fail");
        assert!(key_error.to_string().contains("only `left` and `top`"));

        let value_error = parse_declarative_ui_asset(
            r#"<template><section :style="{ left: '12px' }" /></template>"#,
        )
        .expect_err("literal style value should fail");
        assert!(
            value_error
                .to_string()
                .contains("numeric runtime expressions")
        );
    }

    #[test]
    fn style_literal_still_rejects_when_style_binding_is_allowed() {
        let error =
            parse_declarative_ui_asset(r#"<template><section style="left: 12px" /></template>"#)
                .expect_err("literal style attribute should fail");
        assert!(
            error
                .to_string()
                .contains("style attributes are not supported")
        );
    }

    #[test]
    fn style_binding_parses_runtime_rect_expressions() {
        let asset = parse_declarative_ui_asset(
            r#"<template><section :style="{ left: grid_shell_ref.getBoundingClientRect().left, top: anchorPopup(anchor.getBoundingClientRect(), shell.getBoundingClientRect(), 196, 148, 10, 12).top }" /></template>"#,
        )
        .expect("runtime expression style binding should parse");

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
    fn style_binding_rejects_unknown_runtime_methods() {
        let error = parse_declarative_ui_asset(
            r#"<template><section :style="{ left: anchor.openPopup().left }" /></template>"#,
        )
        .expect_err("unknown runtime method should fail");
        assert!(
            error
                .to_string()
                .contains("unsupported runtime method or function")
        );
    }

    #[test]
    fn container_supports_margin_auto_and_directional_border_utilities() {
        let asset = parse_declarative_ui_asset(
            r#"<template><section class="mt-auto border-t border-panel-subtle-border" /></template>"#,
        )
        .expect("container with supported utility subset should parse");

        let DeclarativeUiNode::Container { node, .. } = asset.root else {
            panic!("expected container node");
        };

        let margin = node.margin.expect("margin");
        assert!(matches!(margin.top, Some(crate::DeclarativeVal::Auto)));
        assert_eq!(margin.left, None);
        assert_eq!(margin.right, None);
        assert_eq!(margin.bottom, None);

        let border = node.border.expect("border");
        assert!(matches!(border.top, Some(crate::DeclarativeVal::Px(1.0))));
        assert_eq!(border.left, None);
        assert_eq!(border.right, None);
        assert_eq!(border.bottom, None);
    }
}
