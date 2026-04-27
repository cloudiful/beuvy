use crate::ast::DeclarativeUiNode;
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, attr_error, bound_attr, model_attr, parse_binding_path_expr,
    parse_bool_or_condition_attr, parse_class_bindings, parse_conditional, parse_event_bindings,
    parse_mustache_expr, parse_node_style, parse_node_style_binding, parse_ref_binding,
    parse_show_attr, parse_state_visual_styles, parse_usize, parse_visual_style,
    reject_legacy_attrs, reject_legacy_bind_attrs, reject_style_attrs_except,
};
use beuvy_runtime::input::InputType;
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_input_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["size", "style"])?;
    let input_type = parse_input_type(node)?;
    let value_attr = node.attribute("value").unwrap_or_default();
    let value_binding = bound_attr(node, "value")
        .map(|expr| parse_binding_path_expr(node, ":value", expr))
        .transpose()?;
    let model_binding = model_attr(node)
        .map(|expr| parse_binding_path_expr(node, "v-model", expr))
        .transpose()?;
    let value = if value_binding.is_some() || model_binding.is_some() {
        String::new()
    } else if let Some(expr) = parse_mustache_expr(value_attr) {
        return Err(attr_error(
            node,
            "value",
            expr,
            "use :value or v-model for bound values",
        ));
    } else {
        value_attr.to_string()
    };
    let size_chars = attr(node, "size")
        .map(|raw| parse_usize(node, "size", raw))
        .transpose()?;
    let rows = attr(node, "rows")
        .map(|raw| parse_usize(node, "rows", raw))
        .transpose()?;
    let (disabled, disabled_expr) = parse_bool_or_condition_attr(node, "disabled", state_specs)?;
    let show_expr = parse_show_attr(node, state_specs)?;
    Ok(DeclarativeUiNode::Input {
        node_id: String::new(),
        name: attr(node, "name").unwrap_or_default().to_string(),
        input_type,
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        conditional: parse_conditional(node, state_specs)?,
        value,
        value_binding,
        model_binding,
        ref_binding: parse_ref_binding(node)?,
        event_bindings: parse_event_bindings(node)?,
        style_binding: parse_node_style_binding(node)?,
        placeholder: node
            .attribute("placeholder")
            .unwrap_or_default()
            .to_string(),
        size_chars,
        rows,
        min: parse_f32_attr(node, "min")?,
        max: parse_f32_attr(node, "max")?,
        step: parse_f32_attr(node, "step")?,
        node_override: Some(parse_node_style(node)?),
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
        disabled,
        disabled_expr,
        show_expr,
    })
}

pub(crate) fn parse_declarative_textarea_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    let parsed = parse_declarative_input_node(node, state_specs)?;
    let DeclarativeUiNode::Input {
        node_id,
        name,
        class,
        class_bindings,
        conditional,
        value,
        value_binding,
        model_binding,
        ref_binding,
        event_bindings,
        style_binding,
        placeholder,
        size_chars,
        rows,
        node_override,
        visual_style,
        state_visual_styles,
        disabled,
        disabled_expr,
        show_expr,
        ..
    } = parsed
    else {
        unreachable!();
    };

    Ok(DeclarativeUiNode::Input {
        node_id,
        name,
        input_type: InputType::Textarea,
        class,
        class_bindings,
        conditional,
        value,
        value_binding,
        model_binding,
        ref_binding,
        event_bindings,
        style_binding,
        placeholder,
        size_chars,
        rows,
        min: None,
        max: None,
        step: None,
        node_override,
        visual_style,
        state_visual_styles,
        disabled,
        disabled_expr,
        show_expr,
    })
}

fn parse_input_type(node: XmlNode<'_, '_>) -> Result<InputType, DeclarativeUiAssetLoadError> {
    match attr(node, "type").unwrap_or("text") {
        "" | "text" => Ok(InputType::Text),
        "textarea" => Ok(InputType::Textarea),
        "number" => Ok(InputType::Number),
        "range" => Ok(InputType::Range),
        raw => Err(attr_error(
            node,
            "type",
            raw,
            "expected text, textarea, number, or range",
        )),
    }
}

fn parse_f32_attr(
    node: XmlNode<'_, '_>,
    name: &str,
) -> Result<Option<f32>, DeclarativeUiAssetLoadError> {
    let raw = attr(node, name).unwrap_or_default();
    if raw.trim().is_empty() {
        return Ok(None);
    }
    raw.parse::<f32>()
        .map(Some)
        .map_err(|_| attr_error(node, name, raw, "expected number"))
}

#[cfg(test)]
mod tests {
    use crate::ast::DeclarativeRefSource;
    use crate::{
        DeclarativeActionSpec, DeclarativeConditionExpr, DeclarativeEventKind, DeclarativeUiNode,
        parse_declarative_ui_asset, set_action_resolver,
    };
    use beuvy_runtime::input::InputType;

    fn install_test_action_resolver() {
        set_action_resolver(|name| match name {
            "settingInput" => Some(DeclarativeActionSpec {
                action_id: "setting.input",
                param_names: vec!["key"],
            }),
            "settingChange" => Some(DeclarativeActionSpec {
                action_id: "setting.change",
                param_names: vec!["key"],
            }),
            "uiScroll" => Some(DeclarativeActionSpec {
                action_id: "ui.scroll",
                param_names: vec!["key"],
            }),
            "uiWheel" => Some(DeclarativeActionSpec {
                action_id: "ui.wheel",
                param_names: vec!["key"],
            }),
            _ => None,
        });
    }

    #[test]
    fn input_defaults_to_text() {
        let asset = parse_declarative_ui_asset(r#"<template><input name="label" /></template>"#)
            .expect("input should parse");
        let DeclarativeUiNode::Input { input_type, .. } = asset.root else {
            panic!("expected input node");
        };
        assert_eq!(input_type, InputType::Text);
    }

    #[test]
    fn range_input_parses_numeric_attrs_and_binding() {
        install_test_action_resolver();
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <input type="range" name="volume" :value="settings.volume" min="0" max="100" step="5" @input="settingInput(settings.key)" />
</template>
"#,
        )
        .expect("range input should parse");
        let DeclarativeUiNode::Input {
            input_type,
            value_binding,
            min,
            max,
            step,
            ..
        } = asset.root
        else {
            panic!("expected input node");
        };
        assert_eq!(input_type, InputType::Range);
        assert_eq!(value_binding.as_deref(), Some("settings.volume"));
        assert_eq!(min, Some(0.0));
        assert_eq!(max, Some(100.0));
        assert_eq!(step, Some(5.0));
    }

    #[test]
    fn vue_event_attrs_parse_to_event_bindings() {
        install_test_action_resolver();
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <input @input="settingInput(settings.key)" @change="settingChange(settings.key)" @scroll="uiScroll(settings.key)" @wheel="uiWheel(settings.key)" />
</template>
"#,
        )
        .expect("events should parse");
        let DeclarativeUiNode::Input { event_bindings, .. } = asset.root else {
            panic!("expected input node");
        };
        let kinds = event_bindings
            .iter()
            .map(|binding| binding.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            kinds,
            vec![
                DeclarativeEventKind::Input,
                DeclarativeEventKind::Change,
                DeclarativeEventKind::Scroll,
                DeclarativeEventKind::Wheel,
            ]
        );
    }

    #[test]
    fn event_binding_rejects_bare_action_id() {
        let error =
            parse_declarative_ui_asset(r#"<template><input @input="ui.input" /></template>"#)
                .expect_err("bare action id should be rejected");
        assert!(
            error
                .to_string()
                .contains("expected whitelisted function call")
        );
    }

    #[test]
    fn host_binding_is_rejected_and_ref_is_supported() {
        let error =
            parse_declarative_ui_asset(r#"<template><input :host="entry.host" /></template>"#)
                .expect_err(":host should be rejected");
        assert!(error.to_string().contains(":host"));

        let asset =
            parse_declarative_ui_asset(r#"<template><input :ref="entry.ref" /></template>"#)
                .expect(":ref should parse");
        let DeclarativeUiNode::Input { ref_binding, .. } = asset.root else {
            panic!("expected input node");
        };
        assert_eq!(
            ref_binding,
            Some(DeclarativeRefSource::Binding("entry.ref".to_string()))
        );
    }

    #[test]
    fn static_ref_parses_and_legacy_ref_is_rejected() {
        let asset = parse_declarative_ui_asset(
            r#"<template><input ref="inventory.grid_shell" /></template>"#,
        )
        .expect("static ref should parse");
        let DeclarativeUiNode::Input { ref_binding, .. } = asset.root else {
            panic!("expected input node");
        };
        assert_eq!(
            ref_binding,
            Some(DeclarativeRefSource::Static(
                "inventory.grid_shell".to_string()
            ))
        );

        let error =
            parse_declarative_ui_asset(r#"<template><input :bevy-ref="entry.ref" /></template>"#)
                .expect_err("legacy bevy-ref should be rejected");
        assert!(error.to_string().contains("legacy bevy-ref syntax"));

        let error =
            parse_declarative_ui_asset(r#"<template><input :ref="entry.id + '.x'" /></template>"#)
                .expect_err("ref expression should be rejected");
        assert!(error.to_string().contains("expected binding path"));
    }

    #[test]
    fn input_supports_value_binding_and_v_model() {
        let asset = parse_declarative_ui_asset(
            r#"<template><input :value="settings.label" v-model="draft.label" /></template>"#,
        )
        .expect("bindings should parse");
        let DeclarativeUiNode::Input {
            value_binding,
            model_binding,
            ..
        } = asset.root
        else {
            panic!("expected input node");
        };
        assert_eq!(value_binding.as_deref(), Some("settings.label"));
        assert_eq!(model_binding.as_deref(), Some("draft.label"));
    }

    #[test]
    fn input_supports_v_show() {
        let asset = parse_declarative_ui_asset(
            r#"<template><input v-show="settings.visible" /></template>"#,
        )
        .expect("v-show should parse");
        let DeclarativeUiNode::Input { show_expr, .. } = asset.root else {
            panic!("expected input node");
        };
        assert_eq!(
            show_expr,
            Some(DeclarativeConditionExpr::Binding(
                "settings.visible".to_string()
            ))
        );
    }

    #[test]
    fn textarea_tag_parses_as_multiline_text_input() {
        let asset = parse_declarative_ui_asset(
            r#"<template><textarea rows="4" size="32" v-model="draft.body" /></template>"#,
        )
        .expect("textarea should parse");
        let DeclarativeUiNode::Input {
            input_type,
            rows,
            size_chars,
            model_binding,
            ..
        } = asset.root
        else {
            panic!("expected input-like textarea node");
        };
        assert_eq!(input_type, InputType::Textarea);
        assert_eq!(rows, Some(4));
        assert_eq!(size_chars, Some(32));
        assert_eq!(model_binding.as_deref(), Some("draft.body"));
    }
}
