use super::runtime_expr::expr_binding_path;
use super::*;
use crate::basic::hr::parse_declarative_hr_node;
use crate::basic::image::parse_declarative_image_node;
use crate::basic::input::parse_declarative_textarea_node;
use crate::basic::label::parse_declarative_label_node;
use crate::basic::link::parse_declarative_link_node;

pub(super) fn parse_node(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<DeclarativeUiNode, DeclarativeUiAssetLoadError> {
    reject_legacy_event_attrs(node)?;
    reject_legacy_bind_attrs(node)?;
    reject_hidden_attrs(node)?;
    let tag_name = node.tag_name().name();
    if attr(node, "v-model").is_some() && !matches!(tag_name, "input" | "textarea" | "select") {
        return Err(attr_error(
            node,
            "v-model",
            attr(node, "v-model").unwrap_or_default(),
            "v-model is only supported on <input>, <textarea>, and <select>",
        ));
    }
    let parsed = match node.tag_name().name() {
        "slot" => {
            reject_style_attrs_except(node, &["style"])?;
            Ok(DeclarativeUiNode::Container {
                node_id: String::new(),
                semantic_tag: None,
                class: attr(node, "class").unwrap_or_default().to_string(),
                class_bindings: parse_class_bindings(node, state_specs)?,
                node: parse_node_style(node)?,
                style_binding: parse_node_style_binding(node)?,
                outlet: Some(required_attr(node, "name")?.to_string()),
                conditional: parse_conditional(node, state_specs)?,
                show_expr: parse_show_attr(node, state_specs)?,
                visual_style: parse_visual_style(node)?,
                state_visual_styles: parse_state_visual_styles(node)?,
                ref_binding: parse_ref_binding(node)?,
                event_bindings: parse_event_bindings(node)?,
                children: parse_child_nodes(node, state_specs)?,
            })
        }
        tag if is_block_tag(tag) => parse_declarative_div_node(node, state_specs),
        tag if is_text_tag(tag) => {
            reject_style_attrs(node)?;
            let content = parse_text_content(node)?;
            let style = parse_text_style(node, tag)?;
            Ok(DeclarativeUiNode::Text {
                node_id: String::new(),
                semantic_tag: text_tag_for(tag),
                class: attr(node, "class").unwrap_or_default().to_string(),
                class_bindings: parse_class_bindings(node, state_specs)?,
                content,
                conditional: parse_conditional(node, state_specs)?,
                show_expr: parse_show_attr(node, state_specs)?,
                ref_binding: parse_ref_binding(node)?,
                style,
            })
        }
        "label" => parse_declarative_label_node(node, state_specs),
        "button" => parse_declarative_button_node(node, state_specs),
        "img" => parse_declarative_image_node(node, state_specs),
        "a" => parse_declarative_link_node(node, state_specs),
        "hr" => parse_declarative_hr_node(node, state_specs),
        "input" => parse_declarative_input_node(node, state_specs),
        "textarea" => parse_declarative_textarea_node(node, state_specs),
        "select" => parse_declarative_select_node(node, state_specs),
        "option" => Err(dsl_error(node, "<option> is only valid inside <select>")),
        "template" => Ok(DeclarativeUiNode::Template {
            node_id: String::new(),
            for_each: parse_v_for(node)?,
            children: parse_child_nodes(node, state_specs)?,
        }),
        other => Err(dsl_error(
            node,
            format!("unknown declarative ui tag <{other}>"),
        )),
    }?;

    Ok(parsed)
}

pub(crate) fn parse_child_nodes(
    node: XmlNode<'_, '_>,
    state_specs: &BTreeMap<String, DeclarativeStateSpec>,
) -> Result<Vec<DeclarativeUiNode>, DeclarativeUiAssetLoadError> {
    let mut children = Vec::new();
    let allows_root_script = node == node.document().root_element();
    for child in node.children() {
        if child.is_element() {
            if child.has_tag_name("script") {
                if allows_root_script {
                    continue;
                }
                return Err(dsl_error(
                    child,
                    "<script> is only allowed as a direct child of the root node",
                ));
            }
            children.push(parse_node(child, state_specs)?);
            continue;
        }

        if let Some(text_node) = parse_raw_text_child(node, child)? {
            children.push(text_node);
        }
    }
    validate_conditional_chain(node, &children)?;
    Ok(children)
}

fn parse_raw_text_child(
    parent: XmlNode<'_, '_>,
    child: XmlNode<'_, '_>,
) -> Result<Option<DeclarativeUiNode>, DeclarativeUiAssetLoadError> {
    let Some(raw) = child.text() else {
        return Ok(None);
    };
    let Some(content) = parse_text_content_from_raw(parent, raw)? else {
        return Ok(None);
    };

    Ok(Some(DeclarativeUiNode::Text {
        node_id: String::new(),
        semantic_tag: Some(DeclarativeTextTag::Span),
        class: String::new(),
        class_bindings: Vec::new(),
        content,
        conditional: DeclarativeConditional::Always,
        show_expr: None,
        ref_binding: None,
        style: DeclarativeTextStyle {
            size: default_text_size_for_tag("span"),
            color: None,
            visual_style: DeclarativeVisualStyle::default(),
            state_visual_styles: DeclarativeStateVisualStyles::default(),
        },
    }))
}

pub(crate) fn parse_text_content(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeUiTextContent, DeclarativeUiAssetLoadError> {
    reject_legacy_attrs(node, &["text", "key", "bind-text", "bind-key"])?;
    if let Some(value) = attr(node, "i18n") {
        return Err(attr_error(
            node,
            "i18n",
            value,
            "`i18n` is no longer supported; use `{{ $t('key') }}` or `{{ $t(binding.key) }}`",
        ));
    }
    for attribute in node.attributes() {
        if attribute.name().starts_with("format-") {
            return Err(attr_error(
                node,
                attribute.name(),
                attribute.value(),
                "localized format attrs are no longer supported; use `{{ $t(...) }}` text content instead",
            ));
        }
    }
    let inline_text = parse_inline_text(node)?;
    if let Some(content) = inline_text {
        return Ok(content);
    }

    Ok(DeclarativeUiTextContent::Static {
        text: String::new(),
    })
}

pub(super) fn parse_inline_text(
    node: XmlNode<'_, '_>,
) -> Result<Option<DeclarativeUiTextContent>, DeclarativeUiAssetLoadError> {
    if element_children(node).next().is_some() {
        return Err(dsl_error(
            node,
            "text-like elements and <button> do not support child elements yet",
        ));
    }
    let mut parts = Vec::new();
    for child in node.children() {
        let Some(raw) = child.text() else {
            continue;
        };
        if let Some(text) = normalize_text_node(raw) {
            parts.push(text);
        }
    }
    if parts.is_empty() {
        return Ok(None);
    }
    let combined = parts.concat();
    if let Some(expr) = parse_mustache_expr(&combined) {
        return parse_text_expr_content(node, expr).map(Some);
    }
    let mut segments = Vec::new();
    for text in parts {
        append_text_segments(&mut segments, parse_text_segments(node, &text)?);
    }
    Ok(content_from_segments(segments))
}

fn parse_text_content_from_raw(
    parent: XmlNode<'_, '_>,
    raw: &str,
) -> Result<Option<DeclarativeUiTextContent>, DeclarativeUiAssetLoadError> {
    let Some(text) = normalize_text_node(raw) else {
        return Ok(None);
    };
    if let Some(expr) = parse_mustache_expr(&text) {
        return parse_text_expr_content(parent, expr).map(Some);
    }
    Ok(content_from_segments(parse_text_segments(parent, &text)?))
}

fn parse_text_expr_content(
    node: XmlNode<'_, '_>,
    expr: &str,
) -> Result<DeclarativeUiTextContent, DeclarativeUiAssetLoadError> {
    if let Some(translated) = parse_dollar_t_call(node, expr)? {
        return Ok(translated);
    }
    Ok(DeclarativeUiTextContent::Bind {
        path: parse_binding_path_expr(node, "text", expr)?,
    })
}

fn parse_dollar_t_call(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<Option<DeclarativeUiTextContent>, DeclarativeUiAssetLoadError> {
    let raw = raw.trim();
    let Some(args) = raw
        .strip_prefix("$t(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let expr = parse_runtime_expr(node, "text", args)?;
    let key = match expr {
        DeclarativeRuntimeExpr::Literal(DeclarativeLiteral::String(value)) => {
            DeclarativeTextKeySource::Static(value)
        }
        other => {
            let Some(path) = expr_binding_path(&other) else {
                return Err(attr_error(
                    node,
                    "text",
                    raw,
                    "$t() expects a text key literal or binding path",
                ));
            };
            DeclarativeTextKeySource::Binding(path.into_owned())
        }
    };
    Ok(Some(DeclarativeUiTextContent::I18n {
        key,
        localized_text_args: Vec::new(),
    }))
}

fn parse_text_segments(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<Vec<DeclarativeUiTextSegment>, DeclarativeUiAssetLoadError> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    while let Some(start) = raw[cursor..].find("{{") {
        let start = cursor + start;
        if start > cursor {
            push_static_segment(&mut segments, &raw[cursor..start]);
        }
        let Some(end_offset) = raw[start + 2..].find("}}") else {
            push_static_segment(&mut segments, &raw[start..]);
            cursor = raw.len();
            break;
        };
        let end = start + 2 + end_offset;
        let expr = raw[start + 2..end].trim();
        if expr.is_empty() {
            push_static_segment(&mut segments, &raw[start..end + 2]);
        } else {
            segments.push(DeclarativeUiTextSegment::Bind {
                path: parse_binding_path_expr(node, "text", expr)?,
            });
        }
        cursor = end + 2;
    }
    if cursor < raw.len() {
        push_static_segment(&mut segments, &raw[cursor..]);
    }
    Ok(segments)
}

fn push_static_segment(segments: &mut Vec<DeclarativeUiTextSegment>, raw: &str) {
    if raw.is_empty() {
        return;
    }
    match segments.last_mut() {
        Some(DeclarativeUiTextSegment::Static { text }) => text.push_str(raw),
        _ => segments.push(DeclarativeUiTextSegment::Static {
            text: raw.to_string(),
        }),
    }
}

fn append_text_segments(
    target: &mut Vec<DeclarativeUiTextSegment>,
    segments: Vec<DeclarativeUiTextSegment>,
) {
    for segment in segments {
        match segment {
            DeclarativeUiTextSegment::Static { text } => push_static_segment(target, &text),
            DeclarativeUiTextSegment::Bind { path } => {
                target.push(DeclarativeUiTextSegment::Bind { path })
            }
        }
    }
}

fn content_from_segments(
    segments: Vec<DeclarativeUiTextSegment>,
) -> Option<DeclarativeUiTextContent> {
    let mut segments = segments;
    while matches!(
        segments.first(),
        Some(DeclarativeUiTextSegment::Static { text }) if text.is_empty()
    ) {
        segments.remove(0);
    }
    while matches!(
        segments.last(),
        Some(DeclarativeUiTextSegment::Static { text }) if text.is_empty()
    ) {
        segments.pop();
    }
    match segments.len() {
        0 => None,
        1 => match segments.pop().expect("one segment") {
            DeclarativeUiTextSegment::Static { text } => {
                Some(DeclarativeUiTextContent::Static { text })
            }
            DeclarativeUiTextSegment::Bind { path } => {
                Some(DeclarativeUiTextContent::Bind { path })
            }
        },
        _ => Some(DeclarativeUiTextContent::Segments { segments }),
    }
}

fn normalize_text_node(raw: &str) -> Option<String> {
    if raw.trim().is_empty() {
        return None;
    }
    let preserve_leading = raw
        .chars()
        .next()
        .is_some_and(|ch| ch.is_whitespace() && !matches!(ch, '\n' | '\r'));
    let preserve_trailing = raw
        .chars()
        .next_back()
        .is_some_and(|ch| ch.is_whitespace() && !matches!(ch, '\n' | '\r'));
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    let mut normalized = String::new();
    if preserve_leading {
        normalized.push(' ');
    }
    normalized.push_str(&collapsed);
    if preserve_trailing {
        normalized.push(' ');
    }
    Some(normalized)
}

pub(super) fn is_block_tag(tag: &str) -> bool {
    matches!(
        tag,
        "div"
            | "section"
            | "header"
            | "footer"
            | "main"
            | "nav"
            | "aside"
            | "article"
            | "form"
            | "fieldset"
            | "ul"
            | "ol"
            | "li"
    )
}

pub(super) fn is_text_tag(tag: &str) -> bool {
    matches!(
        tag,
        "span"
            | "p"
            | "legend"
            | "small"
            | "strong"
            | "em"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
    )
}

fn text_tag_for(tag: &str) -> Option<DeclarativeTextTag> {
    Some(match tag {
        "span" => DeclarativeTextTag::Span,
        "p" => DeclarativeTextTag::P,
        "legend" => DeclarativeTextTag::Legend,
        "small" => DeclarativeTextTag::Small,
        "strong" => DeclarativeTextTag::Strong,
        "em" => DeclarativeTextTag::Em,
        "h1" => DeclarativeTextTag::H1,
        "h2" => DeclarativeTextTag::H2,
        "h3" => DeclarativeTextTag::H3,
        "h4" => DeclarativeTextTag::H4,
        "h5" => DeclarativeTextTag::H5,
        "h6" => DeclarativeTextTag::H6,
        _ => return None,
    })
}

pub(super) fn default_text_size_for_tag(tag: &str) -> f32 {
    crate::style::font_size_for_tag(tag)
}

#[cfg(test)]
mod tests {
    use crate::ast::{DeclarativeContainerTag, DeclarativeTextTag};
    use crate::{
        DeclarativeTextKeySource, DeclarativeUiNode, DeclarativeUiTextContent,
        DeclarativeUiTextSegment, parse_declarative_ui_asset,
    };

    #[test]
    fn block_nodes_preserve_inline_text_children() {
        let asset = parse_declarative_ui_asset(r#"<template><div>Hello world</div></template>"#)
            .expect("container text should parse");

        let DeclarativeUiNode::Container { children, .. } = asset.root else {
            panic!("expected container node");
        };
        assert_eq!(children.len(), 1);
        let DeclarativeUiNode::Text { content, .. } = &children[0] else {
            panic!("expected text child");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::Static { text } if text == "Hello world"
        ));
    }

    #[test]
    fn block_nodes_preserve_binding_text_children() {
        let asset = parse_declarative_ui_asset(
            r#"
<template><div>{{ status.message }}</div></template>
"#,
        )
        .expect("container binding text should parse");

        let DeclarativeUiNode::Container { children, .. } = asset.root else {
            panic!("expected container node");
        };
        assert_eq!(children.len(), 1);
        let DeclarativeUiNode::Text { content, .. } = &children[0] else {
            panic!("expected text child");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::Bind { path } if path == "status.message"
        ));
    }

    #[test]
    fn block_nodes_parse_mixed_text_segments() {
        let asset =
            parse_declarative_ui_asset(r#"<template><div>Hello {{ name }} !</div></template>"#)
                .expect("mixed content should parse");

        let DeclarativeUiNode::Container { children, .. } = asset.root else {
            panic!("expected container node");
        };
        let DeclarativeUiNode::Text { content, .. } = &children[0] else {
            panic!("expected text child");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::Segments { segments }
                if segments == &vec![
                    DeclarativeUiTextSegment::Static {
                        text: "Hello ".to_string()
                    },
                    DeclarativeUiTextSegment::Bind {
                        path: "name".to_string()
                    },
                    DeclarativeUiTextSegment::Static {
                        text: " !".to_string()
                    }
                ]
        ));
    }

    #[test]
    fn inline_text_ignores_formatting_whitespace() {
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <button>
    Open {{ count }}
  </button>
</template>
"#,
        )
        .expect("button mixed text should parse");

        let crate::DeclarativeUiNode::Button { content, .. } = asset.root else {
            panic!("expected button node");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::Segments { segments }
                if segments == vec![
                    DeclarativeUiTextSegment::Static {
                        text: "Open ".to_string()
                    },
                    DeclarativeUiTextSegment::Bind {
                        path: "count".to_string()
                    },
                    DeclarativeUiTextSegment::Static {
                        text: " ".to_string()
                    }
                ]
        ));
    }

    #[test]
    fn inline_text_parses_static_dollar_t_call() {
        let asset =
            parse_declarative_ui_asset(r#"<template><h1>{{ $t('pad.title') }}</h1></template>"#)
                .expect("$t static key should parse");

        let DeclarativeUiNode::Text { content, .. } = asset.root else {
            panic!("expected text node");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::I18n {
                key: DeclarativeTextKeySource::Static(key),
                localized_text_args,
            } if key == "pad.title" && localized_text_args.is_empty()
        ));
    }

    #[test]
    fn inline_text_parses_dynamic_dollar_t_call() {
        let asset =
            parse_declarative_ui_asset(r#"<template><p>{{ $t(entry.label_key) }}</p></template>"#)
                .expect("$t binding key should parse");

        let DeclarativeUiNode::Text { content, .. } = asset.root else {
            panic!("expected text node");
        };
        assert!(matches!(
            content,
            DeclarativeUiTextContent::I18n {
                key: DeclarativeTextKeySource::Binding(path),
                localized_text_args,
            } if path == "entry.label_key" && localized_text_args.is_empty()
        ));
    }

    #[test]
    fn i18n_attribute_is_rejected() {
        let error = parse_declarative_ui_asset(r#"<template><p i18n="pad.title" /></template>"#)
            .expect_err("legacy i18n attr should fail");

        let message = error.to_string();
        assert!(message.contains("`i18n` is no longer supported"));
        assert!(message.contains("{{ $t('key') }}"));
    }

    #[test]
    fn semantic_tags_parse_for_content_nodes() {
        let asset = parse_declarative_ui_asset(
            r#"
            <template>
              <fieldset>
                <legend>Profile</legend>
                <strong>Bold</strong>
                <em>Hint</em>
                <small>Meta</small>
                <ul><li>One</li><li>Two</li></ul>
              </fieldset>
            </template>
            "#,
        )
        .expect("content semantic nodes should parse");

        let DeclarativeUiNode::Container {
            semantic_tag,
            children,
            ..
        } = asset.root
        else {
            panic!("expected fieldset container");
        };
        assert_eq!(semantic_tag, Some(DeclarativeContainerTag::Fieldset));
        assert!(matches!(
            children.first(),
            Some(DeclarativeUiNode::Text {
                semantic_tag: Some(DeclarativeTextTag::Legend),
                ..
            })
        ));
        assert!(children.iter().any(|child| matches!(
            child,
            DeclarativeUiNode::Container {
                semantic_tag: Some(DeclarativeContainerTag::Ul),
                ..
            }
        )));
    }
}
