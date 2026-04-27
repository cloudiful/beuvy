use super::*;

pub(crate) fn parse_node_style_binding(
    node: XmlNode<'_, '_>,
) -> Result<Option<DeclarativeNodeStyleBinding>, DeclarativeUiAssetLoadError> {
    let Some(raw) = bound_attr(node, "style") else {
        return Ok(None);
    };
    let raw = raw.trim();
    if !raw.starts_with('{') || !raw.ends_with('}') {
        return Err(attr_error(
            node,
            "v-bind-style",
            raw,
            "expected object syntax like `{ left: popup.left, top: popup.top }`",
        ));
    }
    let body = raw[1..raw.len() - 1].trim();
    if body.is_empty() {
        return Ok(Some(DeclarativeNodeStyleBinding::default()));
    }

    let mut binding = DeclarativeNodeStyleBinding::default();
    for entry in split_style_binding_entries(body) {
        let Some((raw_name, raw_value)) = split_style_binding_entry(entry) else {
            return Err(attr_error(
                node,
                "v-bind-style",
                entry,
                "expected `left: binding.path` or `top: binding.path`",
            ));
        };
        let name = parse_style_binding_name(node, raw_name.trim())?;
        let value = parse_runtime_expr(node, "v-bind-style", raw_value.trim())?;
        reject_invalid_style_runtime_expr(node, raw_value.trim(), &value)?;
        match name {
            "left" => binding.left = Some(value),
            "top" => binding.top = Some(value),
            _ => {
                return Err(attr_error(
                    node,
                    "v-bind-style",
                    raw_name.trim(),
                    "only `left` and `top` are supported in :style",
                ));
            }
        }
    }

    Ok(Some(binding))
}

fn split_style_binding_entries(raw: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut start = 0usize;
    let mut quote = None;
    let mut paren_depth = 0usize;
    for (index, ch) in raw.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            ',' if paren_depth == 0 => {
                entries.push(raw[start..index].trim());
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    entries.push(raw[start..].trim());
    entries
        .into_iter()
        .filter(|entry| !entry.is_empty())
        .collect()
}

fn split_style_binding_entry(raw: &str) -> Option<(&str, &str)> {
    let mut quote = None;
    for (index, ch) in raw.char_indices() {
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            ':' => return Some((&raw[..index], &raw[index + ch.len_utf8()..])),
            _ => {}
        }
    }
    None
}

fn parse_style_binding_name(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<&'static str, DeclarativeUiAssetLoadError> {
    let unquoted = if (raw.starts_with('"') && raw.ends_with('"'))
        || (raw.starts_with('\'') && raw.ends_with('\''))
    {
        &raw[1..raw.len() - 1]
    } else {
        raw
    };

    match unquoted.trim() {
        "left" => Ok("left"),
        "top" => Ok("top"),
        _ => Err(attr_error(
            node,
            "v-bind-style",
            raw,
            "only `left` and `top` are supported in :style",
        )),
    }
}

fn reject_invalid_style_runtime_expr(
    node: XmlNode<'_, '_>,
    raw: &str,
    expr: &DeclarativeRuntimeExpr,
) -> Result<(), DeclarativeUiAssetLoadError> {
    if matches!(expr, DeclarativeRuntimeExpr::Literal(_)) {
        return Err(attr_error(
            node,
            "v-bind-style",
            raw,
            "style binding values must resolve to numeric runtime expressions",
        ));
    }
    Ok(())
}
