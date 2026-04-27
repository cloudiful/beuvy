use super::*;

pub(crate) fn parse_event_bindings(
    node: XmlNode<'_, '_>,
) -> Result<Vec<DeclarativeEventBinding>, DeclarativeUiAssetLoadError> {
    reject_legacy_event_attrs(node)?;
    let mut bindings = Vec::new();
    for attribute in node.attributes() {
        let name = attribute.name();
        let Some(kind) = name.strip_prefix("v-on-") else {
            continue;
        };
        if kind.contains('.') {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "event modifiers are not supported in declarative UI runtime",
            ));
        }
        if kind == "click" {
            continue;
        }
        let (action_id, params) =
            super::onclick::parse_dispatch_call(node, name, attribute.value().trim(), name)?;
        bindings.push(DeclarativeEventBinding {
            kind: parse_event_kind(node, name, kind)?,
            action_id,
            params,
        });
    }
    Ok(bindings)
}

pub(crate) fn reject_legacy_event_attrs(
    node: XmlNode<'_, '_>,
) -> Result<(), DeclarativeUiAssetLoadError> {
    for attribute in node.attributes() {
        let name = attribute.name();
        if matches!(
            name,
            "onclick" | "oninput" | "onchange" | "onscroll" | "onwheel"
        ) || name.starts_with("on:")
            || name.starts_with("on-")
        {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "legacy event attributes are not supported; use @click/@input/@change/@scroll/@wheel",
            ));
        }
    }
    Ok(())
}

pub(super) fn parse_event_kind(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<DeclarativeEventKind, DeclarativeUiAssetLoadError> {
    match raw {
        "activate" => Ok(DeclarativeEventKind::Activate),
        "input" => Ok(DeclarativeEventKind::Input),
        "change" => Ok(DeclarativeEventKind::Change),
        "scroll" => Ok(DeclarativeEventKind::Scroll),
        "wheel" => Ok(DeclarativeEventKind::Wheel),
        _ => Err(attr_error(node, name, raw, "unknown event kind")),
    }
}

pub(crate) fn parse_usize(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<usize, DeclarativeUiAssetLoadError> {
    raw.parse::<usize>()
        .map_err(|_| attr_error(node, name, raw, "expected unsigned integer"))
}

pub(crate) fn element_children<'a>(node: XmlNode<'a, 'a>) -> impl Iterator<Item = XmlNode<'a, 'a>> {
    node.children().filter(|child| child.is_element())
}

pub(crate) fn attr<'a>(node: XmlNode<'a, 'a>, name: &str) -> Option<&'a str> {
    node.attribute(name)
}

pub(crate) fn parse_mustache_expr(raw: &str) -> Option<&str> {
    let raw = raw.trim();
    raw.strip_prefix("{{")
        .and_then(|value| value.strip_suffix("}}"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

pub(crate) fn bound_attr<'a>(node: XmlNode<'a, 'a>, name: &str) -> Option<&'a str> {
    let bound_name = format!("v-bind-{name}");
    node.attribute(bound_name.as_str())
}

pub(crate) fn model_attr<'a>(node: XmlNode<'a, 'a>) -> Option<&'a str> {
    node.attribute("v-model")
}

pub(crate) fn parse_ref_binding(
    node: XmlNode<'_, '_>,
) -> Result<Option<DeclarativeRefSource>, DeclarativeUiAssetLoadError> {
    if let Some(raw) = bound_attr(node, "ref") {
        return Ok(Some(DeclarativeRefSource::Binding(
            parse_binding_path_expr(node, ":ref", raw)?,
        )));
    }
    let Some(raw) = attr(node, "ref") else {
        return Ok(None);
    };
    if let Some(expr) = parse_mustache_expr(raw) {
        return Err(attr_error(node, "ref", expr, "use :ref for bound refs"));
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(attr_error(node, "ref", raw, "ref cannot be empty"));
    }
    Ok(Some(DeclarativeRefSource::Static(trimmed.to_string())))
}

pub(crate) fn reject_legacy_bind_attrs(
    node: XmlNode<'_, '_>,
) -> Result<(), DeclarativeUiAssetLoadError> {
    for attribute in node.attributes() {
        let name = attribute.name();
        if name == "v-bind-host" {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "`:host` is not supported; use `ref` or `:ref` for node refs",
            ));
        }
        if matches!(name, "bevy-ref" | "v-bind-bevy-ref") {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "legacy bevy-ref syntax is not supported; use `ref` or `:ref`",
            ));
        }
        if name.starts_with("bind-") || name == "key-expr" {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "legacy binding attributes are not supported; use :prop, v-model, or :key",
            ));
        }
    }
    Ok(())
}

pub(crate) fn parse_binding_path_expr(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    if is_identifier_path(raw) {
        Ok(raw.to_string())
    } else {
        Err(attr_error(node, name, raw, "expected binding path"))
    }
}

pub(crate) fn parse_show_attr(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<Option<DeclarativeConditionExpr>, DeclarativeUiAssetLoadError> {
    reject_hidden_attrs(node)?;
    let Some(raw) = attr(node, "v-show") else {
        return Ok(None);
    };
    parse_condition_expr(node, "v-show", raw, true, state_specs).map(Some)
}

pub(crate) fn reject_hidden_attrs(
    node: XmlNode<'_, '_>,
) -> Result<(), DeclarativeUiAssetLoadError> {
    for attribute in node.attributes() {
        let name = attribute.name();
        if matches!(name, "hidden" | "v-bind-hidden") || name == "bind-hidden" {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "hidden attributes are not supported; use v-if or v-show",
            ));
        }
    }
    Ok(())
}

pub(crate) fn parse_bool_or_condition_attr(
    node: XmlNode<'_, '_>,
    name: &str,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<(bool, Option<DeclarativeConditionExpr>), DeclarativeUiAssetLoadError> {
    if let Some(raw) = bound_attr(node, name) {
        return Ok((
            false,
            Some(parse_condition_expr(
                node,
                &format!("v-bind-{name}"),
                raw,
                true,
                state_specs,
            )?),
        ));
    }
    let Some(raw) = attr(node, name) else {
        return Ok((false, None));
    };
    if let Some(expr) = parse_mustache_expr(raw) {
        return Ok((
            false,
            Some(parse_condition_expr(node, name, expr, true, state_specs)?),
        ));
    }
    match raw.trim() {
        "" => Ok((true, None)),
        "true" => Ok((true, None)),
        "false" => Ok((false, None)),
        _ => Err(attr_error(
            node,
            name,
            raw,
            "expected true, false, or {{ expr }}",
        )),
    }
}

pub(crate) fn reject_legacy_attrs(
    node: XmlNode<'_, '_>,
    names: &[&str],
) -> Result<(), DeclarativeUiAssetLoadError> {
    for name in names {
        if let Some(value) = attr(node, name) {
            return Err(attr_error(
                node,
                name,
                value,
                "legacy DSL attribute is not supported",
            ));
        }
    }
    for attribute in node.attributes() {
        let name = attribute.name();
        if name.starts_with("bind:") || name.starts_with("on:") || name.starts_with("on-") {
            return Err(attr_error(
                node,
                name,
                attribute.value(),
                "legacy DSL attribute syntax is not supported",
            ));
        }
    }
    Ok(())
}

pub(crate) fn reject_style_attrs(node: XmlNode<'_, '_>) -> Result<(), DeclarativeUiAssetLoadError> {
    reject_style_attrs_except(node, &[])
}

pub(crate) fn reject_style_attrs_except(
    node: XmlNode<'_, '_>,
    allowed_names: &[&str],
) -> Result<(), DeclarativeUiAssetLoadError> {
    if let Some(value) = attr(node, "style") {
        return Err(attr_error(
            node,
            "style",
            value,
            "style attributes are not supported; use :style for supported bindings or Tailwind utility classes via class",
        ));
    }

    if !allowed_names.contains(&"style") {
        if let Some(value) = bound_attr(node, "style") {
            return Err(attr_error(
                node,
                "v-bind-style",
                value,
                "dynamic style bindings are not supported on this element",
            ));
        }
    }

    const STYLE_ATTRS: &[&str] = &[
        "width",
        "height",
        "min-width",
        "min-height",
        "max-width",
        "max-height",
        "direction",
        "justify",
        "align",
        "align-content",
        "align-self",
        "wrap",
        "grow",
        "shrink",
        "basis",
        "gap",
        "row-gap",
        "column-gap",
        "padding",
        "margin",
        "border",
        "radius",
        "overflow",
        "overflow-x",
        "overflow-y",
        "display",
        "position",
        "left",
        "right",
        "top",
        "bottom",
        "background",
        "border-color",
        "color",
        "size",
    ];

    for name in STYLE_ATTRS {
        if allowed_names.contains(name) {
            continue;
        }
        if let Some(value) = attr(node, name) {
            return Err(attr_error(
                node,
                name,
                value,
                "style attributes are not supported; use Tailwind utility classes via class",
            ));
        }
    }

    Ok(())
}

pub(crate) fn required_attr<'a>(
    node: XmlNode<'a, 'a>,
    name: &str,
) -> Result<&'a str, DeclarativeUiAssetLoadError> {
    attr(node, name).ok_or_else(|| {
        dsl_error(
            node,
            format!("<{}> requires {name:?} attribute", node.tag_name().name()),
        )
    })
}

pub(crate) fn dsl_error(
    node: XmlNode<'_, '_>,
    message: impl Into<String>,
) -> DeclarativeUiAssetLoadError {
    let pos = node.document().text_pos_at(node.range().start);
    DeclarativeUiAssetLoadError::InvalidDsl(format!(
        "{} at {}:{}",
        message.into(),
        pos.row,
        pos.col
    ))
}

pub(crate) fn attr_error(
    node: XmlNode<'_, '_>,
    name: &str,
    value: &str,
    message: &str,
) -> DeclarativeUiAssetLoadError {
    dsl_error(
        node,
        format!(
            "invalid {}=\"{}\" on <{}>: {}",
            name,
            value,
            node.tag_name().name(),
            message
        ),
    )
}
