use crate::ast::{DeclarativeSelectOption, DeclarativeUiNode};
use crate::error::DeclarativeUiAssetLoadError;
use crate::parser::{
    DeclarativeStateSpec, attr, attr_error, bound_attr, dsl_error, element_children, model_attr,
    parse_binding_path_expr, parse_bool_or_condition_attr, parse_class_bindings, parse_conditional,
    parse_event_bindings, parse_mustache_expr, parse_node_style, parse_node_style_binding,
    parse_ref_binding, parse_show_attr, parse_state_visual_styles, parse_text_content,
    parse_utility_class_patch, parse_v_for, parse_visual_style, reject_legacy_attrs,
    reject_legacy_bind_attrs, reject_style_attrs, reject_style_attrs_except,
};
use roxmltree::Node as XmlNode;
use std::collections::BTreeMap;

pub(crate) fn parse_declarative_select_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["text", "key", "bind-text", "bind-key", "visible"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs_except(node, &["style"])?;
    if attr(node, "multiple").is_some() {
        return Err(attr_error(
            node,
            "multiple",
            attr(node, "multiple").unwrap_or_default(),
            "multiple select is not supported",
        ));
    }

    let class_patch = parse_utility_class_patch(node)?;
    let (value, value_binding, model_binding) = parse_select_value(node)?;
    let options = parse_select_options(node, state_specs)?;
    let (disabled, disabled_expr) = parse_bool_or_condition_attr(node, "disabled", state_specs)?;
    let show_expr = parse_show_attr(node, state_specs)?;

    Ok(DeclarativeUiNode::Select {
        node_id: String::new(),
        name: attr(node, "name").unwrap_or_default().to_string(),
        class: attr(node, "class").unwrap_or_default().to_string(),
        class_bindings: parse_class_bindings(node, state_specs)?,
        conditional: parse_conditional(node, state_specs)?,
        value,
        value_binding,
        model_binding,
        ref_binding: parse_ref_binding(node)?,
        event_bindings: parse_event_bindings(node)?,
        style_binding: parse_node_style_binding(node)?,
        options,
        node_override: Some(parse_node_style(node)?),
        visual_style: parse_visual_style(node)?,
        state_visual_styles: parse_state_visual_styles(node)?,
        disabled,
        disabled_expr,
        show_expr,
        label_size_override: class_patch.text_size,
    })
}

fn parse_select_value(
    node: XmlNode<'_, '_>,
) -> Result<(String, Option<String>, Option<String>), DeclarativeUiAssetLoadError> {
    let value_attr = attr(node, "value").unwrap_or_default();
    let value_binding = bound_attr(node, "value")
        .map(|expr| parse_binding_path_expr(node, ":value", expr))
        .transpose()?;
    let model_binding = model_attr(node)
        .map(|expr| parse_binding_path_expr(node, "v-model", expr))
        .transpose()?;
    if value_binding.is_some() || model_binding.is_some() {
        return Ok((String::new(), value_binding, model_binding));
    }
    if let Some(expr) = parse_mustache_expr(value_attr) {
        return Err(attr_error(
            node,
            "value",
            expr,
            "use :value or v-model for bound values",
        ));
    }
    Ok((value_attr.to_string(), None, None))
}

fn parse_select_options(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<Vec<DeclarativeSelectOption>, DeclarativeUiAssetLoadError> {
    let mut options = Vec::new();
    let mut selected_count = 0usize;

    for child in element_children(node) {
        if !child.has_tag_name("option") {
            return Err(dsl_error(child, "<select> accepts only <option> children"));
        }
        let option = parse_select_option(child, state_specs)?;
        if option.selected && option.repeat.is_none() {
            selected_count += 1;
        }
        options.push(option);
    }

    if attr(node, "value").is_none() && selected_count > 1 {
        return Err(dsl_error(
            node,
            "<select> accepts at most one selected <option> when value is omitted",
        ));
    }

    Ok(options)
}

fn parse_select_option(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeSelectOption, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["visible", "text", "key", "bind-text", "bind-key"])?;
    reject_legacy_bind_attrs(node)?;
    reject_style_attrs(node)?;

    if attr(node, "v-on-click").is_some() {
        return Err(attr_error(
            node,
            "@click",
            attr(node, "v-on-click").unwrap_or_default(),
            "<option> does not support @click; use <select @change>",
        ));
    }
    if !parse_event_bindings(node)?.is_empty() {
        return Err(dsl_error(
            node,
            "<option> does not support event handlers; use <select @change>",
        ));
    }

    let selected = parse_native_bool_attr(node, "selected")?;
    let (disabled, disabled_expr) = parse_bool_or_condition_attr(node, "disabled", state_specs)?;
    let (value, value_binding) = parse_option_value(node)?;

    Ok(DeclarativeSelectOption {
        value,
        value_binding,
        content: parse_text_content(node)?,
        selected,
        disabled,
        disabled_expr,
        conditional: parse_conditional(node, state_specs)?,
        repeat: attr(node, "v-for").map(|_| parse_v_for(node)).transpose()?,
    })
}

fn parse_option_value(
    node: XmlNode<'_, '_>,
) -> Result<(Option<String>, Option<String>), DeclarativeUiAssetLoadError> {
    let value_attr = attr(node, "value").unwrap_or_default();
    if let Some(expr) = bound_attr(node, "value") {
        return Ok((None, Some(parse_binding_path_expr(node, ":value", expr)?)));
    }
    if let Some(expr) = parse_mustache_expr(value_attr) {
        return Ok((None, Some(parse_binding_path_expr(node, "value", expr)?)));
    }
    Ok((attr(node, "value").map(str::to_string), None))
}

#[cfg(test)]
mod tests {
    use crate::{
        DeclarativeActionSpec, DeclarativeUiNode, parse_declarative_ui_asset, set_action_resolver,
    };

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
    fn select_option_supports_v_for_and_bound_attrs() {
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <select name="language" :value="current">
    <option v-for="entry in options" :value="entry.value" :disabled="entry.disabled">{{ entry.text }}</option>
  </select>
</template>
"#,
        )
        .expect("select asset should parse");

        let DeclarativeUiNode::Select { options, .. } = asset.root else {
            panic!("expected select root");
        };
        let option = options.first().expect("expected option");
        assert_eq!(option.value_binding.as_deref(), Some("entry.value"));
        assert!(option.disabled_expr.is_some());
        assert!(option.repeat.is_some());
    }

    #[test]
    fn select_supports_v_model() {
        install_test_action_resolver();
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <select v-model="settings.language" @change="settingChange(settings.key)">
    <option value="en">English</option>
  </select>
</template>
"#,
        )
        .expect("select should parse");
        let DeclarativeUiNode::Select {
            model_binding,
            event_bindings,
            ..
        } = asset.root
        else {
            panic!("expected select root");
        };
        assert_eq!(model_binding.as_deref(), Some("settings.language"));
        assert_eq!(event_bindings.len(), 1);
    }
}

fn parse_native_bool_attr(
    node: XmlNode<'_, '_>,
    name: &str,
) -> Result<bool, DeclarativeUiAssetLoadError> {
    let Some(raw) = attr(node, name) else {
        return Ok(false);
    };
    match raw.trim() {
        "" | "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(attr_error(node, name, raw, "expected boolean attribute")),
    }
}
