use crate::ast::{
    DeclarativeButtonType, DeclarativeEventBinding, DeclarativeEventKind, DeclarativeOnClick,
    DeclarativeUiNode,
};
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, attr_error, parse_bool_or_condition_attr, parse_class_bindings,
    parse_conditional, parse_event_bindings, parse_node_style, parse_node_style_binding,
    parse_onclick, parse_ref_binding, parse_show_attr, parse_state_visual_styles,
    parse_text_content, parse_utility_class_patch, parse_visual_style, reject_legacy_attrs,
    reject_legacy_bind_attrs, reject_legacy_event_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_button_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(
        node,
        &["text", "selected", "visible", "active", "v-bind-active"],
    )?;
    reject_legacy_bind_attrs(node)?;
    reject_legacy_event_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;
    let class_patch = parse_utility_class_patch(node)?;
    let onclick = attr(node, "v-on-click")
        .map(|value| parse_onclick(node, "@click", value, state_specs))
        .transpose()?;
    let mut event_bindings = parse_event_bindings(node)?;
    if let Some(DeclarativeOnClick::DispatchCall { action_id, params }) = &onclick {
        event_bindings.push(DeclarativeEventBinding {
            kind: DeclarativeEventKind::Activate,
            action_id: action_id.clone(),
            params: params.clone(),
        });
    }
    let (disabled, disabled_expr) = parse_bool_or_condition_attr(node, "disabled", state_specs)?;
    let show_expr = parse_show_attr(node, state_specs)?;
    Ok(DeclarativeUiNode::Button {
        node_id: String::new(),
        name: attr(node, "name").unwrap_or_default().to_string(),
        button_type: parse_button_type(node)?,
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        content: parse_text_content(node)?,
        conditional: parse_conditional(node, state_specs)?,
        onclick,
        event_bindings,
        ref_binding: parse_ref_binding(node)?,
        node_override: Some(parse_node_style(node)?),
        style_binding: parse_node_style_binding(node)?,
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
        disabled,
        disabled_expr,
        show_expr,
        label_size_override: class_patch.text_size,
    })
}

fn parse_button_type(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeButtonType, DeclarativeUiAssetLoadError> {
    match attr(node, "type").unwrap_or("button") {
        "" | "button" => Ok(DeclarativeButtonType::Button),
        "submit" => Ok(DeclarativeButtonType::Submit),
        "reset" => Ok(DeclarativeButtonType::Reset),
        raw => Err(attr_error(
            node,
            "type",
            raw,
            "expected button, submit, or reset",
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DeclarativeButtonType, DeclarativeClassBinding, DeclarativeOnClick, DeclarativeUiNode,
        parse_declarative_ui_asset,
    };

    #[test]
    fn click_shorthand_parses_button_action() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><button @click="tab = 'inventory'">Open</button></template>
<script>let tab = 'overview';</script>
"#,
        )
        .expect("button should parse");
        let DeclarativeUiNode::Button { onclick, .. } = asset.root else {
            panic!("expected button node");
        };
        assert!(matches!(
            onclick,
            Some(DeclarativeOnClick::Assign { ref name, .. }) if name == "tab"
        ));
    }

    #[test]
    fn class_object_shorthand_parses_button_binding() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><button class="button-root" :class="{ 'btn-selected': tab === 'save' }">Save</button></template>
<script>let tab: "save" | "load" = "save";</script>
"#,
        )
        .expect("button should parse");
        let DeclarativeUiNode::Button { class_bindings, .. } = asset.root else {
            panic!("expected button node");
        };
        assert_eq!(class_bindings.len(), 1);
        assert!(matches!(
            &class_bindings[0],
            DeclarativeClassBinding::RuntimeExpr { .. }
        ));
    }

    #[test]
    fn class_binding_accepts_ternary_string_syntax() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><button :class="tab === 'save' ? 'btn-selected' : ''">Save</button></template>
<script>let tab: "save" | "load" = "save";</script>
"#,
        )
        .expect("ternary class binding should parse");
        let DeclarativeUiNode::Button { class_bindings, .. } = asset.root else {
            panic!("expected button node");
        };
        assert!(matches!(
            &class_bindings[0],
            DeclarativeClassBinding::RuntimeExpr { .. }
        ));
    }

    #[test]
    fn class_binding_accepts_array_syntax() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><button :class="['btn', tab === 'save' ? 'btn-selected' : '']">Save</button></template>
<script>let tab: "save" | "load" = "save";</script>
"#,
        )
        .expect("array class binding should parse");
        let DeclarativeUiNode::Button { class_bindings, .. } = asset.root else {
            panic!("expected button node");
        };
        assert!(matches!(
            &class_bindings[0],
            DeclarativeClassBinding::RuntimeExpr { .. }
        ));
    }

    #[test]
    fn button_active_binding_is_not_supported() {
        let error = parse_declarative_ui_asset(
            r#"
<template><button :active="tab === 'save'">Save</button></template>
<script>let tab: "save" | "load" = "save";</script>
"#,
        )
        .expect_err("active binding should be rejected");
        assert!(error.to_string().contains("active"));
    }

    #[test]
    fn button_type_parses() {
        let asset = parse_declarative_ui_asset(
            r#"<template><button type="submit">Save</button></template>"#,
        )
        .expect("button should parse");
        let DeclarativeUiNode::Button { button_type, .. } = asset.root else {
            panic!("expected button node");
        };
        assert_eq!(button_type, DeclarativeButtonType::Submit);
    }
}
