use super::*;

pub(super) fn parse_literal(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<DeclarativeLiteral, DeclarativeUiAssetLoadError> {
    if let Some(value) = parse_string_literal_optional(raw) {
        return Ok(DeclarativeLiteral::String(value));
    }
    match raw {
        "true" => return Ok(DeclarativeLiteral::Bool(true)),
        "false" => return Ok(DeclarativeLiteral::Bool(false)),
        _ => {}
    }
    if raw.contains('.') {
        if let Ok(number) = raw.parse::<f64>() {
            return Ok(DeclarativeLiteral::Number(DeclarativeNumber::F64(number)));
        }
    } else if let Ok(number) = raw.parse::<i32>() {
        return Ok(DeclarativeLiteral::Number(DeclarativeNumber::I32(number)));
    } else if let Ok(number) = raw.parse::<i64>() {
        return Ok(DeclarativeLiteral::Number(DeclarativeNumber::I64(number)));
    }
    Err(attr_error(
        node,
        name,
        raw,
        "expected string, bool, integer, or float literal",
    ))
}

pub(super) fn parse_literal_for_type(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
    expected: DeclarativeScriptType,
) -> Result<DeclarativeLiteral, DeclarativeUiAssetLoadError> {
    match expected {
        DeclarativeScriptType::String => {
            parse_string_literal(node, name, raw).map(DeclarativeLiteral::String)
        }
        DeclarativeScriptType::Bool => match raw {
            "true" => Ok(DeclarativeLiteral::Bool(true)),
            "false" => Ok(DeclarativeLiteral::Bool(false)),
            _ => Err(attr_error(node, name, raw, "expected bool literal")),
        },
        DeclarativeScriptType::I32 => raw
            .parse::<i32>()
            .map(|value| DeclarativeLiteral::Number(DeclarativeNumber::I32(value)))
            .map_err(|_| attr_error(node, name, raw, "expected i32 literal")),
        DeclarativeScriptType::I64 => raw
            .parse::<i64>()
            .map(|value| DeclarativeLiteral::Number(DeclarativeNumber::I64(value)))
            .map_err(|_| attr_error(node, name, raw, "expected i64 literal")),
        DeclarativeScriptType::F32 => raw
            .parse::<f32>()
            .map(|value| DeclarativeLiteral::Number(DeclarativeNumber::F32(value)))
            .map_err(|_| attr_error(node, name, raw, "expected f32 literal")),
        DeclarativeScriptType::F64 => raw
            .parse::<f64>()
            .map(|value| DeclarativeLiteral::Number(DeclarativeNumber::F64(value)))
            .map_err(|_| attr_error(node, name, raw, "expected f64 literal")),
    }
}

pub(super) fn parse_string_literal(
    node: XmlNode<'_, '_>,
    name: &str,
    raw: &str,
) -> Result<String, DeclarativeUiAssetLoadError> {
    parse_string_literal_optional(raw)
        .ok_or_else(|| attr_error(node, name, raw, "expected string literal"))
}

pub(super) fn parse_string_literal_optional(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        return Some(raw[1..raw.len() - 1].to_string());
    }
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return Some(raw[1..raw.len() - 1].to_string());
    }
    None
}
