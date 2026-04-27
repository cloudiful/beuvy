use super::*;

pub(crate) fn parse_v_for(
    node: XmlNode<'_, '_>,
) -> Result<DeclarativeForEach, DeclarativeUiAssetLoadError> {
    let raw = required_attr(node, "v-for")?.trim();
    let Some((aliases_raw, source_raw)) = raw.split_once(" in ") else {
        return Err(attr_error(
            node,
            "v-for",
            raw,
            "expected `item in items` or `(item, index) in items`",
        ));
    };

    let (item_alias, index_alias) = parse_v_for_aliases(node, aliases_raw.trim())?;
    let source = parse_binding_path_expr(node, "v-for", source_raw.trim())?;
    reject_legacy_bind_attrs(node)?;
    let key_expr = attr(node, "v-bind-key").map(str::trim).map(str::to_string);
    if let Some(key_expr) = &key_expr
        && key_expr.is_empty()
    {
        return Err(attr_error(node, ":key", "", "expected binding path"));
    }

    Ok(DeclarativeForEach {
        source,
        item_alias,
        index_alias,
        key_expr,
    })
}

#[cfg(test)]
mod tests {
    use crate::{DeclarativeUiNode, parse_declarative_ui_asset};

    #[test]
    fn v_for_reads_vue_key_binding() {
        let asset = parse_declarative_ui_asset(
            r#"
<template>
  <div>
    <template v-for="item in items" :key="item.id">
      <span>{{ item.name }}</span>
    </template>
  </div>
</template>
"#,
        )
        .expect("template should parse");
        let DeclarativeUiNode::Container { children, .. } = asset.root else {
            panic!("expected container");
        };
        let DeclarativeUiNode::Template { for_each, .. } = &children[0] else {
            panic!("expected template");
        };
        assert_eq!(for_each.key_expr.as_deref(), Some("item.id"));
    }
}

fn parse_v_for_aliases(
    node: XmlNode<'_, '_>,
    raw: &str,
) -> Result<(String, Option<String>), DeclarativeUiAssetLoadError> {
    if let Some(inner) = raw
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut parts = inner.split(',').map(str::trim);
        let item_alias = parse_state_name(node, "v-for", parts.next().unwrap_or_default())?;
        let index_alias = parts
            .next()
            .map(|value| parse_state_name(node, "v-for", value))
            .transpose()?;
        if parts.next().is_some() {
            return Err(attr_error(
                node,
                "v-for",
                raw,
                "expected `(item, index) in items`",
            ));
        }
        return Ok((item_alias, index_alias));
    }

    Ok((parse_state_name(node, "v-for", raw)?, None))
}
